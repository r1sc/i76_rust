use glam::{Mat4, Vec3, Vec4};

use super::binary_reader::{BinaryReader, Readable};

pub struct BWD2Tag {
    pub name: String,
    pub size: u32,
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
impl ColorRGB {
    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new((self.0 as f32) / 255.0, (self.1 as f32) / 255.0, (self.2 as f32) / 255.0)
    }
}

#[derive(Clone, Copy)]
pub struct RotationAxis {
    pub right: Vec3,
    pub up: Vec3,
    pub forward: Vec3,
    pub matrix: Mat4,
}
impl Readable for RotationAxis {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let right = Vec3::consume(reader)?;
        let up = Vec3::consume(reader)?;
        let forward = Vec3::consume(reader)?;
        let matrix = Mat4::from_cols(
            (right).extend(0.0),
            (up).extend(0.0),
            (forward).extend(0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );
        Ok(Self { right, up, forward, matrix })
    }
}
