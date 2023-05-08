use std::collections::HashMap;

use super::binary_reader::Readable;

pub struct ZFSFileInfo {
    pub offset: u32,
    pub length: u32,
    pub compression: u8,
    pub decompressed_length: u32,
}

pub struct ZFSPakEntry {
    pub containing_pak_filename: String,
    pub offset: u32,
    pub length: u32,
}

pub enum ZFSEntry {
    Normal(ZFSFileInfo),
    PakEntry(ZFSPakEntry),
}

pub struct ZFS {
    pub files: HashMap<String, ZFSEntry>,
    pub pix_files: Vec<String>,
}

impl Readable for ZFS {
    fn consume(reader: &mut super::binary_reader::BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut files = HashMap::new();

        let magic = reader.read_fixed(4)?;
        assert_eq!(&magic, "ZFSF");
        let version = reader.read_u32()?;
        assert_eq!(version, 1);
        let _unk1 = reader.read_u32()?;
        let num_files_in_each_directory = reader.read_u32()?;
        let num_files_total = reader.read_u32()?;
        let _unk2 = reader.read_u32();
        let _unk3 = reader.read_u32();

        let mut pix_files = Vec::new();

        loop {
            let next_dir_offset = reader.read_u32()?;
            for _file_index in 0..num_files_in_each_directory {
                if files.len() == num_files_total as usize {
                    break;
                }

                let filename = reader.read_fixed(16)?.to_lowercase();
                let offset = reader.read_u32()?;
                let _id = reader.read_u32()?;
                let length = reader.read_u32()?;
                let _unk = reader.read_u32()?;
                let compression = reader.read_u8()?;
                let mut decompressed_length = reader.read_u16()? as u32;
                decompressed_length = ((reader.read_u8()? as u32) << 16) | decompressed_length;

                if filename.ends_with(".pix") {
                    pix_files.push(filename.clone());
                }

                files.insert(
                    filename,
                    ZFSEntry::Normal(ZFSFileInfo {
                        offset,
                        length,
                        compression,
                        decompressed_length,
                    }),
                );
            }
            if files.len() == num_files_total as usize {
                break;
            }

            reader.seek_from_start(next_dir_offset as u64)?;
        }

        Ok(Self { files, pix_files })
    }
}
