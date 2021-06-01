use super::common::{BinaryReader, Readable, RotationAxis};
use crate::math::Vec3;

/*
********************************* Common Tags
*/
#[derive(Clone)]
pub struct GEOPart {
    pub name: String,
    pub axis: RotationAxis,
    pub position: Vec3,
    pub relative_to: String,
    pub v_unk1: Vec3,
    pub bbox_size: Vec3,
    pub u1: f32,
    pub flag: u32,
    pub u3: u32,
}
impl Readable for GEOPart {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let name = reader.read_fixed(8)?;
        let axis = RotationAxis::consume(reader)?;
        let position = Vec3::consume(reader)?;
        let relative_to = reader.read_fixed(8)?;
        let v_unk1 = Vec3::consume(reader)?;
        let bbox_size = Vec3::consume(reader)?;
        let u1 = reader.read_f32()?;
        let flag = reader.read_u32()?;
        let u3 = reader.read_u32()?;

        Ok(Self {
            name,
            axis,
            position,
            relative_to,
            v_unk1,
            bbox_size,
            u1,
            flag,
            u3,
        })
    }
}

/*
****************************** SDF Tags
*/
#[derive(Clone)]
pub struct SDFC {
    pub name: String,
    pub unk: u32,
    pub lods: Vec<f32>,
    pub health: u32,
    pub xdf_name: String,
    pub death_sound_name: String,
}
impl Readable for SDFC {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let name = reader.read_fixed(16)?;
        let unk = reader.read_u32()?;
        let lods = (0..5)
            .map(|_| reader.read_f32())
            .collect::<Result<Vec<f32>, std::io::Error>>()?;
        let health = reader.read_u32()?;
        let xdf_name = reader.read_fixed(13)?;
        let death_sound_name = reader.read_fixed(13)?;

        Ok(Self {
            name,
            unk,
            lods,
            health,
            xdf_name,
            death_sound_name,
        })
    }
}

#[derive(Clone)]
pub struct SGEOPart {
    pub geo_part: GEOPart,
    pub u4: u32,
    pub v_unk2: Vec3,
    pub u5: f32,
}
impl Readable for SGEOPart {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let geo_part = GEOPart::consume(reader)?;
        let u4 = reader.read_u32()?;
        let v_unk2 = Vec3::consume(reader)?;
        let u5 = reader.read_f32()?;

        Ok(Self {
            geo_part,
            u4,
            v_unk2,
            u5,
        })
    }
}

pub struct LodLevel {
    pub lod_parts: Vec<SGEOPart>,
    pub destroyed_parts: Vec<SGEOPart>,
}
impl LodLevel {
    fn consume<R>(reader: &mut R, num_parts: u32) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let lod_parts = (0..num_parts)
            .map(|_| SGEOPart::consume(reader))
            .collect::<Result<Vec<SGEOPart>, std::io::Error>>()?;
        let destroyed_parts = (0..num_parts)
            .map(|_| SGEOPart::consume(reader))
            .collect::<Result<Vec<SGEOPart>, std::io::Error>>()?;

        Ok(Self {
            lod_parts,
            destroyed_parts,
        })
    }
}

pub struct SGEO {
    pub num_parts: u32,
    pub lod_levels: Vec<LodLevel>,
}
impl Readable for SGEO {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let num_parts = reader.read_u32()?;
        let lods = (0..3)
            .map(|_| LodLevel::consume(reader, num_parts))
            .collect::<Result<Vec<LodLevel>, std::io::Error>>()?;

        Ok(Self {
            num_parts,
            lod_levels: lods,
        })
    }
}

pub struct SCHK {
    pub target_part: String,
    pub num_parts: u32,
    pub parts: Vec<SGEOPart>,
}
impl Readable for SCHK {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let target_part = reader.read_fixed(8)?;
        let num_parts = reader.read_u32()?;
        let parts = (0..num_parts)
            .map(|_| SGEOPart::consume(reader))
            .collect::<Result<Vec<SGEOPart>, std::io::Error>>()?;

        Ok(Self {
            target_part,
            num_parts,
            parts,
        })
    }
}

pub struct SOBJ {
    pub label: String,
    pub mat: Vec<f32>,
    pub position_root: String,
    pub unk: Vec<u8>,
}
impl Readable for SOBJ {
    fn consume<R>(reader: &mut R) -> Result<Self, std::io::Error>
    where
        R: BinaryReader,
    {
        let label = reader.read_fixed(8)?;
        let mat = (0..12)
            .map(|_| reader.read_f32())
            .collect::<Result<Vec<f32>, std::io::Error>>()?;
        let position_root = reader.read_fixed(8)?;
        let unk = reader.bytes(36)?;

        Ok(Self {
            label,
            mat,
            position_root,
            unk,
        })
    }
}
