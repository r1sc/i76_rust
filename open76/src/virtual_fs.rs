use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

use crate::fileparsers::binary_reader::{BinaryReader, Readable};

pub fn load<T>(path: &str) -> Result<T, std::io::Error>
where
    T: Readable,
{
    if !Path::new(path).exists() {
        panic!("Path does not exist: {}", path);
    }
    let f = File::open(path)?;
    let mut reader = BinaryReader {
        reader: BufReader::new(f),
    };

    let result = T::consume(&mut reader);
    result
}
