use crate::math::Vec3;

pub struct BWD2Tag {
    pub name: String,
    pub size: u32
}

pub trait BinaryReader {
    fn read_u8(&mut self) -> Result<u8, std::io::Error>;
    fn read_u16(&mut self) -> Result<u16, std::io::Error>;
    fn read_u32(&mut self) -> Result<u32, std::io::Error>;
    fn read_i16(&mut self) -> Result<i16, std::io::Error>;
    fn read_i32(&mut self) -> Result<i32, std::io::Error>;

    fn read_f32(&mut self) -> Result<f32, std::io::Error>;
    fn read_fixed(&mut self, count: usize) -> Result<String, std::io::Error>;

    fn bytes(&mut self, count: usize) -> Result<Vec<u8>, std::io::Error>;
    fn rest_bytes(&mut self) -> Result<Vec<u8>, std::io::Error>;

    fn bwd2_tag(&mut self) -> Result<BWD2Tag, std::io::Error>;

    fn seek(&mut self, offset: i64) -> Result<u64, std::io::Error>;
}
pub trait Readable {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error> where 
        R : BinaryReader,
        Self: Sized;
}


#[derive(Debug, Clone, Copy)]
pub struct ColorRGB(u8, u8, u8);
impl Readable for ColorRGB {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error> where R : BinaryReader {
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
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error> where R : BinaryReader {        
        let right = Vec3::consume(reader)?;
        let up = Vec3::consume(reader)?;
        let forward = Vec3::consume(reader)?;
        Ok(Self{right, up, forward})
    }
}