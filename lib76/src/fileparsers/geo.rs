use super::common::*;
use crate::math::*;

#[derive(Debug)]
pub struct Geo {
    pub name: String,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<GeoFace>,
}

impl Readable for Geo {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let _tag = reader.read_fixed(4)?;
        let _b = reader.read_u32()?;
        let name = reader.read_fixed(16)?;
        let vertex_count = reader.read_u32()?;
        let face_count = reader.read_u32()?;
        let _unk = reader.read_u32()?;
        let vertices = (0..vertex_count)
            .map(|_| Vec3::consume(reader))
            .collect::<Result<Vec<Vec3>, std::io::Error>>()?;
        let normals = (0..vertex_count)
            .map(|_| Vec3::consume(reader))
            .collect::<Result<Vec<Vec3>, std::io::Error>>()?;
        let faces = (0..face_count)
            .map(|_| GeoFace::consume(reader))
            .collect::<Result<Vec<GeoFace>, std::io::Error>>()?;      

        Ok(Self {
            name,
            vertices,
            normals,
            faces,
        })
    }
}

#[derive(Debug)]
pub struct GeoFace {
    pub index: u32,
    pub color: ColorRGB,
    pub normal: Vec4,
    pub flags: (u8, u8, u8),
    pub texture_name: String,
    pub vertex_refs: Vec<GeoVertexRef>,
}
impl Readable for GeoFace {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let index = reader.read_u32()?;
        let num_vertices = reader.read_u32()?;
        let color = ColorRGB::consume(reader)?;
        let normal = Vec4::consume(reader)?;

        let _unk = reader.read_u32()?;

        let flag_1 = reader.read_u8()?;
        let flag_2 = reader.read_u8()?;
        let flag_3 = reader.read_u8()?;

        let texture_name = reader.read_fixed(13)?;
        reader.read_u32()?;
        reader.read_u32()?;
        let vertex_refs = (0..num_vertices)
            .map(|_| GeoVertexRef::consume(reader))
            .collect::<Result<Vec<GeoVertexRef>, std::io::Error>>()?;

        Ok(Self {
            index,
            color,
            flags: (flag_1, flag_2, flag_3),
            texture_name,
            normal,
            vertex_refs,
        })
    }
}

#[derive(Debug)]
pub struct GeoVertexRef {
    pub vertex_index: u32,
    pub normal_index: u32,
    pub uv: (f32, f32),
}
impl Readable for GeoVertexRef {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let vertex_index = reader.read_u32()?;
        let normal_index = reader.read_u32()?;
        let u = reader.read_f32()?;
        let v = reader.read_f32()?;

        Ok(Self {
            vertex_index,
            normal_index,
            uv: (u, v)
        })
    }
}
