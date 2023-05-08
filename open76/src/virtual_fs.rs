use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::Path,
    ptr::null_mut,
};

use anyhow::bail;
use lib76::fileparsers::{
    binary_reader::{BinaryReader, Readable},
    pix::PIX,
    zfs::{ZFSEntry, ZFSFileInfo, ZFSPakEntry, ZFS},
};

pub struct VirtualFS {
    zfs: ZFS,
    pub game_path: String,
    pub zfs_path: String,
}

fn consume_file<T>(path: &str) -> Result<T, std::io::Error>
where
    T: Readable,
{
    let f = File::open(path)?;
    let mut reader = BinaryReader {
        reader: BufReader::new(Box::new(f)),
    };

    T::consume(&mut reader)
}

impl VirtualFS {
    pub fn new(game_path: String) -> anyhow::Result<Self> {
        let zfs_path = format!("{}/i76.zfs", &game_path);
        let zfs: ZFS = consume_file(&zfs_path)?;

        let mut this = Self {
            game_path,
            zfs,
            zfs_path,
        };

        for pix_file in &this.zfs.pix_files {
            let pak_filename = pix_file.replace(".pix", ".pak");

            let pix: PIX = this.load(&pix_file)?;

            for pix_info in pix.files {
                this.zfs.files.insert(
                    pix_info.filename,
                    ZFSEntry::PakEntry(ZFSPakEntry {
                        offset: pix_info.offset,
                        length: pix_info.length,
                        containing_pak_filename: pak_filename.clone(),
                    }),
                );
            }
        }

        Ok(this)
    }
    
    pub fn get_file_list(&self) -> Vec<String> {
        self.zfs.files.keys().cloned().collect()
    }
    
    fn get_archive_data(&self, name: &str) -> anyhow::Result<Vec<u8>> {
        if let Some(entry) = self.zfs.files.get(&name.to_lowercase()) {
            match entry {
                ZFSEntry::Normal(file) => {
                    let f = File::open(&self.zfs_path)?;
                    let mut reader = BinaryReader {
                        reader: BufReader::new(Box::new(f)),
                    };
                    reader.seek_from_start(file.offset as u64)?;
                    let bytes = reader.bytes(file.length as usize)?;
                    match file.compression {
                        0 => Ok(bytes),
                        2 => {
                            let mut decompressed: Vec<u8> =
                                vec![0; file.decompressed_length as usize];
                            let mut decompressed_length: u64 = 0;
                            unsafe {
                                lzo_sys::lzo1x_decompress(
                                    bytes.as_ptr() as _,
                                    bytes.len() as u64,
                                    decompressed.as_mut_ptr() as _,
                                    &mut decompressed_length,
                                    null_mut(),
                                );
                            };
                            assert_eq!(decompressed_length, file.decompressed_length as u64);
                            Ok(decompressed)
                        }
                        4 => {
                            let mut decompressed: Vec<u8> =
                                vec![0; file.decompressed_length as usize];
                            let mut decompressed_length: u64 = 0;
                            unsafe {
                                lzo_sys::lzo1y_decompress(
                                    bytes.as_ptr() as _,
                                    bytes.len() as u64,
                                    decompressed.as_mut_ptr() as _,
                                    &mut decompressed_length,
                                    null_mut(),
                                );
                            };
                            assert_eq!(decompressed_length, file.decompressed_length as u64);
                            Ok(decompressed)
                        }
                        _ => bail!("Unknown compression format"),
                    }
                }
                ZFSEntry::PakEntry(pak_entry) => {
                    let pak_data = self.get_archive_data(&pak_entry.containing_pak_filename)?;
                    let filedata = &pak_data
                        [pak_entry.offset as usize..(pak_entry.offset + pak_entry.length) as usize];
                    let part: Vec<u8> = filedata.into();
                    Ok(part)
                }
            }
        } else {
            bail!("Failed to find file")
        }
    }

    fn get_disk_path(&self, name: &str) -> String {
        let extension = Path::new(name)
            .extension()
            .and_then(|o| o.to_str())
            .unwrap_or("");

        match extension.to_uppercase().as_str() {
            "TER" | "MSN" | "PCF" => format!("{}/miss16/{}", &self.game_path, name),
            _ => format!("{}/ADDON/{}", &self.game_path, name),
        }
    }

    pub fn exists(&self, name: &str) -> bool {
        let disk_path = self.get_disk_path(name);
        let exists_on_disk = std::fs::metadata(&disk_path).is_ok();

        self.zfs.files.contains_key(name) || exists_on_disk
    }

    pub fn load<T>(&self, name: &str) -> anyhow::Result<T>
    where
        T: Readable,
    {
        let disk_path = self.get_disk_path(name);
        let exists_on_disk = std::fs::metadata(&disk_path).is_ok();

        let result = if exists_on_disk {
            consume_file(&disk_path)?
        } else {
            let data = self.get_archive_data(name)?;
            let mut br = BinaryReader {
                reader: BufReader::new(Box::new(Cursor::new(data))),
            };
            T::consume(&mut br)?
        };

        Ok(result)
    }
}
