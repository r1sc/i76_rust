use std::{
    fs::File,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

use crate::{
    fileparsers::binary_reader::{BinaryReader, Readable},
    zfs_archive::ZFSArchive,
};

pub struct VirtualFS {
    pub disk_folders: Vec<PathBuf>,
    pub zfs_archive: Option<ZFSArchive>,
}

pub enum VirtualPath {
    ZFS,
    Disk(PathBuf),
}

impl VirtualFS {
    pub fn get_path(&self, name: &str) -> Option<VirtualPath> {
        // 1. Check disk folders
        for folder_path in &self.disk_folders {
            let file_path = folder_path.join(name);
            if let Ok(true) = file_path.try_exists() {
                return Some(VirtualPath::Disk(file_path));
            }
        }

        // 2. Check ZFS
        if let Some(zfs) = self.zfs_archive.as_ref() {
            if zfs.exists(&name.to_lowercase()) {
                return Some(VirtualPath::ZFS);
            }
        }
        
        None
    }

    pub fn exists(&self, name: &str) -> bool {
        self.get_path(name).is_some()
    }

    pub fn load<T>(&self, name: &str) -> Result<T, std::io::Error>
    where
        T: Readable,
    {
        let file_path = self.get_path(name).ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "File does not exist",
        ))?;
        match file_path {
            VirtualPath::ZFS => {
                let data = self.zfs_archive.as_ref().unwrap().get_archive_data(name)?;
                let mut br = BinaryReader {
                    reader: BufReader::new(Box::new(Cursor::new(data))),
                };
                T::consume(&mut br)
            }
            VirtualPath::Disk(path) => {
                let f = File::open(path)?;
                let mut reader = BinaryReader {
                    reader: BufReader::new(Box::new(f)),
                };

                T::consume(&mut reader)
            }
        }
    }
}
