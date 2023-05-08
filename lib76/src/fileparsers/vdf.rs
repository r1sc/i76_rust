use glam::Vec3;

use super::{binary_reader::{BinaryReader, Readable}, bwd2::GEOPart, common::RotationAxis};

pub struct VDF {
    pub vdfc: VDFC,
    pub vlocs: Vec<VLOC>,
    pub vgeo: VGEO
}
impl Readable for VDF {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let mut vdfc: Option<VDFC> = None;
        let mut vlocs: Vec<VLOC> = Vec::new();
        let mut vgeo: Option<VGEO> = None;

        while let Ok(tag) = reader.bwd2_tag() {
            match &tag.name[..] {
                "VDFC" => vdfc = Some(VDFC::consume(reader)?),
                "VLOC" => vlocs.push(VLOC::consume(reader)?),
                "VGEO" => vgeo = Some(VGEO::consume(reader)?),
                _ => {
                    reader.seek_relative(tag.size as i64)?;
                }
            }
        }

        Ok(VDF {
            vdfc: vdfc.expect("Expected VDFC to be found in VDF"),
            vgeo: vgeo.expect("Expected VGEO to be found in VDF"),
            vlocs
        })
    }
}

pub struct VDFC {
    pub name: String, // 20
    pub vehicle_type: u32,
    pub vehicle_size: u32,
    pub lod_distances: [f32; 5],
    pub mass: f32,
    pub collision_multiplier: f32,
    pub drag_coefficient: f32,
    pub unk: u32,
}
impl Readable for VDFC {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        let name = reader.read_fixed(20)?;
        let vehicle_type = reader.read_u32()?;
        let vehicle_size = reader.read_u32()?;
        let lod_distance_1 = reader.read_f32()?;
        let lod_distance_2 = reader.read_f32()?;
        let lod_distance_3 = reader.read_f32()?;
        let lod_distance_4 = reader.read_f32()?;
        let lod_distance_5 = reader.read_f32()?;
        let mass = reader.read_f32()?;
        let collision_multiplier = reader.read_f32()?;
        let drag_coefficient = reader.read_f32()?;
        let unk = reader.read_u32()?;
        let vpit = reader.read_fixed(13)?;

        Ok(Self {
            name,
            vehicle_type,
            vehicle_size,
            lod_distances: [
                lod_distance_1,
                lod_distance_2,
                lod_distance_3,
                lod_distance_4,
                lod_distance_5,
            ],
            mass,
            collision_multiplier,
            drag_coefficient,
            unk,
        })
    }
}


pub struct VLOC {
    pub index: u32,
    pub rotation: RotationAxis,
    pub position: Vec3,
}
impl Readable for VLOC {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    {
        let index = reader.read_u32()?;
        let rotation = RotationAxis::consume(reader)?;
        let position = Vec3::consume(reader)?;

        Ok(Self {
            index,
            rotation,
            position,
        })
    }
}

pub struct VGEO {
    pub num_parts: u32,
    pub third_person_parts: Vec<Vec<Vec<GEOPart>>>, // 4 lods * 4 damage states * num_parts
    pub first_person_parts: Vec<Vec<Vec<GEOPart>>>, // 4 damage states * num_parts
    pub other_parts: Vec<Vec<Vec<GEOPart>>>,        // 2 lods * 4 damage states * num_parts
}
impl Readable for VGEO {
    fn consume(reader: &mut BinaryReader) -> Result<Self, std::io::Error>
    {
        let num_parts = reader.read_u32()?;

        let mut read_parts = |num_lods| {
            (0..num_lods).map(|_lod| {
                (0..4).map(|_damage_state| {
                    (0..num_parts)
                        .map(|_part_no| GEOPart::consume(reader))
                        .collect::<Result<Vec<GEOPart>, std::io::Error>>()
                }).collect::<Result<Vec<Vec<GEOPart>>, std::io::Error>>()
            }).collect::<Result<Vec<Vec<Vec<GEOPart>>>, std::io::Error>>()
        };

        let third_person_parts = read_parts(4)?;
        let first_person_parts = read_parts(1)?;
        let other_parts = read_parts(2)?;

        Ok(Self {
            num_parts,
            third_person_parts,
            first_person_parts,
            other_parts
        })
    }
}
