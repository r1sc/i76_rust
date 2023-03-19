use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
};

use byteorder::{LittleEndian, ReadBytesExt};

use super::common::BWD2Tag;

pub trait Readable {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized;
}

pub struct BinaryReader {
    pub reader: BufReader<File>,
}

impl BinaryReader {
    pub fn read_u8(&mut self) -> Result<u8, std::io::Error> {
        self.reader.read_u8()
    }

    pub fn read_u16(&mut self) -> Result<u16, std::io::Error> {
        self.reader.read_u16::<LittleEndian>()
    }

    pub fn read_u32(&mut self) -> Result<u32, std::io::Error> {
        self.reader.read_u32::<LittleEndian>()
    }

    pub fn read_i16(&mut self) -> Result<i16, std::io::Error> {
        self.reader.read_i16::<LittleEndian>()
    }

    pub fn read_i32(&mut self) -> Result<i32, std::io::Error> {
        self.reader.read_i32::<LittleEndian>()
    }

    pub fn read_f32(&mut self) -> Result<f32, std::io::Error> {
        self.reader.read_f32::<LittleEndian>()
    }

    pub fn read_fixed(&mut self, count: usize) -> Result<String, std::io::Error> {
        let mut buffer = self.bytes(count)?;

        buffer = buffer.iter_mut().map(|c| *c & 0x7F).collect();

        let index = buffer.iter().position(|c| *c == 0).unwrap_or(count);

        let s = std::str::from_utf8(&buffer[0..index])
            .expect("Failed to read bytes to string! Not valid utf-8")
            .to_string();

        Ok(s)
    }

    pub fn bytes(&mut self, count: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = vec![0; count];
        self.reader.read_exact(&mut buffer)?;
        Ok(buffer)
    }

    pub fn len(&mut self) -> Result<u64, std::io::Error> {
        let pos = self.reader.stream_position()?;
        let end_pos = self.reader.seek(SeekFrom::End(0))?;
        let len = end_pos - pos;
        self.reader.seek(SeekFrom::Start(pos))?;

        Ok(len)
    }

    pub fn rest_bytes_u8(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let len = self.len()?;

        let mut result: Vec<u8> = Vec::with_capacity(len as usize);
        self.reader.read(&mut result)?;

        Ok(result)
    }

    pub fn rest_bytes_u16(&mut self) -> Result<Vec<u16>, std::io::Error> {
        let len = self.len()?;

        let mut result: Vec<u16> = vec![0; (len / 2) as usize];
        self.reader.read_u16_into::<LittleEndian>(&mut result)?;

        Ok(result)
    }

    pub fn bwd2_tag(&mut self) -> Result<BWD2Tag, std::io::Error> {
        let name = self.read_fixed(4)?;
        let size = self.read_u32()? -8;
        Ok(BWD2Tag { name, size })
    }

    pub fn seek(&mut self, offset: i64) -> Result<u64, std::io::Error> {
        self.reader.seek(SeekFrom::Current(offset))
    }
}
