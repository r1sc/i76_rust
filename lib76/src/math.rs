use crate::fileparsers::binary_reader::{BinaryReader, Readable};


#[derive(Debug, Clone, Copy)]
pub struct Vec3(pub f32, pub f32, pub f32);

impl Readable for Vec3 {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_f32()?;
        Ok(Self(x, y, z))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec4(pub f32, pub f32, pub f32, pub f32);

impl Readable for Vec4 {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    {
        let x = reader.read_f32()?;
        let y = reader.read_f32()?;
        let z = reader.read_f32()?;
        let w = reader.read_f32()?;
        Ok(Self(x, y, z, w))
    }
}
