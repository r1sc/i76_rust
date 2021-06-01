#![feature(bufreader_seek_relative)]

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{BufReader, Seek, SeekFrom};
use std::{fs::File, io::Read};

use crate::fileparsers::common::{BWD2Tag, BinaryReader, Readable};

pub struct VirtualFS {
    // geo_cache: HashMap<String, Geo>,
}

fn read_to_buffer(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}

struct MyBinaryReader {
    reader: BufReader<File>,
}

impl BinaryReader for MyBinaryReader {
    fn read_u8(&mut self) -> Result<u8, std::io::Error> {
        self.reader.read_u8()
    }

    fn read_u16(&mut self) -> Result<u16, std::io::Error> {
        self.reader.read_u16::<LittleEndian>()
    }

    fn read_u32(&mut self) -> Result<u32, std::io::Error> {
        self.reader.read_u32::<LittleEndian>()
    }

    fn read_i16(&mut self) -> Result<i16, std::io::Error> {
        self.reader.read_i16::<LittleEndian>()
    }

    fn read_i32(&mut self) -> Result<i32, std::io::Error> {
        self.reader.read_i32::<LittleEndian>()
    }

    fn read_f32(&mut self) -> Result<f32, std::io::Error> {
        self.reader.read_f32::<LittleEndian>()
    }

    fn read_fixed(&mut self, count: usize) -> Result<String, std::io::Error> {
        let mut buffer = vec![0; count];
        self.reader.read(&mut buffer)?;
        let index = buffer.iter().position(|c| *c == 0).unwrap_or(count);

        let s = std::str::from_utf8(&buffer[0..index])
            .expect("Failed to read bytes to string! Not valid utf-8")
            .to_owned();
        Ok(s)
    }

    fn bytes(&mut self, count: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = vec![0; count];
        self.reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    fn rest_bytes(&mut self) -> Result<Vec<u8>, std::io::Error> {
        todo!()
    }

    fn bwd2_tag(&mut self) -> Result<BWD2Tag, std::io::Error> {
        let name = self.read_fixed(4)?;
        let size = self.read_u32()?;
        Ok(BWD2Tag { name, size })
    }

    fn seek(&mut self, offset: i64) -> Result<u64, std::io::Error> {        
        self.reader.seek(SeekFrom::Current(offset))
    }
}

pub fn load<T>(path: &str) -> Result<T, std::io::Error>
where
    T: Readable,
{
    // let buffer = read_to_buffer(path).expect("Failed to read file");
    let f = File::open(path)?;
    let mut reader = MyBinaryReader {
        reader: BufReader::new(f),
    };

    T::consume(&mut reader)    
}