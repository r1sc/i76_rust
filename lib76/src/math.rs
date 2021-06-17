use glam::{Vec3, Vec4};

use crate::fileparsers::binary_reader::{BinaryReader, Readable};

impl Readable for Vec3 {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_f32()?;
        Ok(Vec3::new(x, y, z))
    }
}

impl Readable for Vec4 {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_f32()?;
        let w = reader.read_f32()?;
        Ok(Vec4::new(x, y, z, w))
    }
}
