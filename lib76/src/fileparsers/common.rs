use crate::math::Vec3;

use super::binary_reader::{BinaryReader, Readable};

pub struct BWD2Tag {
    pub name: String,
    pub size: u32
}


#[derive(Debug, Clone, Copy)]
pub struct ColorRGB(pub u8, pub u8, pub u8);
impl Readable for ColorRGB {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let r = reader.read_u8()?;
        let g = reader.read_u8()?;
        let b = reader.read_u8()?;                
        Ok(Self(r, g, b))
    }
}

#[derive(Clone, Copy)]
pub struct RotationAxis {
    pub right: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
}
impl Readable for RotationAxis {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {        
        let right = Vec3::consume(reader)?;
        let up = Vec3::consume(reader)?;
        let forward = Vec3::consume(reader)?;
        Ok(Self{right, up, forward})
    }
}