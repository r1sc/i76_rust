use std::{
    fs::File,
    io::{BufReader, Cursor},
    path::Path,
    ptr::null_mut,
};

use anyhow::bail;
use lib76::{fileparsers::{
    binary_reader::{BinaryReader, Readable},
    zfs::ZFSEntry,
}, zfs_archive::ZFSArchive};

pub struct VirtualFS {
    zfs_archive: ZFSArchive,
    pub game_path: String,
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

        Ok(Self {
            game_path,
            zfs_archive: ZFSArchive::new(zfs_path)?
        })
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

        self.zfs_archive.exists(name) || exists_on_disk
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
            let data = self.zfs_archive.get_archive_data(name)?;
            let mut br = BinaryReader {
                reader: BufReader::new(Box::new(Cursor::new(data))),
            };
            T::consume(&mut br)?
        };

        Ok(result)
    }
}
