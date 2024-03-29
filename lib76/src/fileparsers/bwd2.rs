use glam::Vec3;

use super::{
    binary_reader::{BinaryReader, Readable},
    common::RotationAxis,
};

/*
********************************* Common Tags
*/
#[derive(Debug, Clone)]
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
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
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
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let name = reader.read_fixed(16)?;
        let unk = reader.read_u32()?;
        let lods = (0..5)
            .map(|_| reader.read_f32())
            .collect::<Result<_, _>>()?;
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

#[derive(Debug, Clone)]
pub struct SGEOPart {
    pub geo_part: GEOPart,
    pub u4: u32,
    pub v_unk2: Vec3,
    pub u5: f32,
}
impl Readable for SGEOPart {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
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

#[derive(Debug)]
pub struct LodLevel {
    pub lod_parts: Vec<SGEOPart>,
    pub destroyed_parts: Vec<SGEOPart>,
}
impl LodLevel {
    fn consume(reader: &mut BinaryReader, num_parts: u32) -> Result<Self, std::io::Error> {
        let lod_parts = (0..num_parts)
            .map(|_| SGEOPart::consume(reader))
            .collect::<Result<_, _>>()?;
        let destroyed_parts = (0..num_parts)
            .map(|_| SGEOPart::consume(reader))
            .collect::<Result<_, _>>()?;

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
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
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

#[derive(Debug)]
pub struct WGEOLodLevel {
    pub lod_parts: Vec<GEOPart>,
    pub destroyed_parts: Vec<GEOPart>,
}
impl WGEOLodLevel {
    fn consume(reader: &mut BinaryReader, num_parts: u32) -> Result<Self, std::io::Error> {
        let lod_parts = (0..num_parts)
            .map(|_| GEOPart::consume(reader))
            .collect::<Result<_, _>>()?;
        let destroyed_parts = (0..num_parts)
            .map(|_| GEOPart::consume(reader))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            lod_parts,
            destroyed_parts,
        })
    }
}

#[derive(Debug)]
pub struct WGEO {
    pub num_parts: u32,
    pub left: Vec<WGEOLodLevel>,
    pub right: Vec<WGEOLodLevel>,
}
impl Readable for WGEO {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let num_parts = reader.read_u32()?;
        let right = (0..4)
            .map(|_| WGEOLodLevel::consume(reader, num_parts))
            .collect::<Result<_, _>>()?;

        let left = (0..4)
            .map(|_| WGEOLodLevel::consume(reader, num_parts))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            num_parts,
            left,
            right,
        })
    }
}

pub struct SCHK {
    pub target_part: String,
    pub num_parts: u32,
    pub parts: Vec<SGEOPart>,
}
impl Readable for SCHK {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
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
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
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

#[derive(Debug)]
pub struct WDFC {
    pub name: String,
    pub datas: Vec<f32>,
    pub something: String,
}

impl Readable for WDFC {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let name = reader.read_fixed(20)?;
        let datas = (0..8)
            .map(|_| reader.read_f32())
            .collect::<Result<_, _>>()?;
        let something = reader.read_fixed(14)?;

        Ok(Self {
            name,
            datas,
            something,
        })
    }
}


#[derive(Debug)]
pub struct WLOC {
    pub rotation_axis: RotationAxis,
    pub position: Vec3,
}

impl Readable for WLOC {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error> {
        let _unk1 = reader.read_u32()?;
        let rotation_axis = RotationAxis::consume(reader)?;
        let position = Vec3::consume(reader)?;
        let _unk = reader.read_f32()?;

        Ok(Self {
            rotation_axis,
            position,
        })
    }
}