use std::{
    fs::File,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
    ptr::null_mut,
};

use crate::fileparsers::{
    binary_reader::{BinaryReader, Readable},
    pix::PIX,
    zfs::{ZFSEntry, ZFSPakEntry, ZFS},
};

pub struct ZFSArchive {
    zfs: ZFS,
    pub path: PathBuf,
}

impl ZFSArchive {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        let f = File::open(path)?;
        let mut reader = BinaryReader {
            reader: BufReader::new(Box::new(f)),
        };

        let mut this = Self {
            path: path.to_path_buf(),
            zfs: ZFS::consume(&mut reader)?,
        };

        for pix_file in &this.zfs.pix_files {
            let pak_filename = pix_file.replace(".pix", ".pak");

            let data = this.get_archive_data(pix_file)?;
            let mut br = BinaryReader {
                reader: BufReader::new(Box::new(Cursor::new(data))),
            };
            let pix = PIX::consume(&mut br)?;

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

    pub fn load<T>(&self, name: &str) -> Result<T, std::io::Error>
    where
        T: Readable,
    {
        let data = self.get_archive_data(name)?;
        let mut br = BinaryReader {
            reader: BufReader::new(Box::new(Cursor::new(data))),
        };
        T::consume(&mut br)
    }

    pub fn get_archive_data(&self, name: &str) -> Result<Vec<u8>, std::io::Error> {
        if let Some(entry) = self.zfs.files.get(&name.to_lowercase()) {
            match entry {
                ZFSEntry::Normal(file) => {
                    let f = File::open(&self.path)?;
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
                        _ => Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Unknown compression format",
                        )),
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
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to find file",
            ))
        }
    }

    pub fn exists(&self, name: &str) -> bool {
        self.zfs.files.contains_key(name)
    }
}
