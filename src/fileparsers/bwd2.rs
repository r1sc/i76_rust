use nom::{
    call, count, do_parse, many1, map, named, named_args,
    number::complete::{le_f32, le_u32},
    switch, take, take_str, value,
};

use crate::{fileparsers::common::Parsable, math::Vec3};

use super::common::RotationAxis;

pub enum BWD2Chunk {
    ChunkUnk(String, u32),
    ChunkBWD2,
    ChunkREV,
    ChunkSCHK(SCHK),
    ChunkSOBJ(SOBJ),
    ChunkSGEO(SGEO),
    ChunkSDFC(SDFC),
    ChunkEXIT
}

named!(
    tag_parser<(&[u8], u32)>,
    do_parse!(
        name: take!(4) 
        >> len: le_u32 
        >> ((name, len))
    )
);

named!(
    chunk_parser<BWD2Chunk>,
    switch!(call!(tag_parser),
        (b"SGEO", _) => map!(SGEO::parse, BWD2Chunk::ChunkSGEO) |
        (b"BWD2", l) => map!(take!(l-8), |_| BWD2Chunk::ChunkBWD2) |
        (b"REV\0", l) => map!(take!(l-8), |_| BWD2Chunk::ChunkREV) |
        (b"SCHK", _) => map!(SCHK::parse, BWD2Chunk::ChunkSCHK) |
        (b"SOBJ", _) => map!(SOBJ::parse, BWD2Chunk::ChunkSOBJ) |
        (b"SDFC", _) => map!(SDFC::parse, BWD2Chunk::ChunkSDFC) |
        (b"EXIT", _) => value!(BWD2Chunk::ChunkEXIT) |
        (t, l) => map!(take!(l-8), |_| BWD2Chunk::ChunkUnk(String::from_utf8(t.to_owned()).unwrap(), l))
    )
);

named!(
    pub bwd2_parser<Vec<BWD2Chunk>>,
    many1!(chunk_parser)
);

/* Common Tags */
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
impl super::common::Parsable<Self> for GEOPart {
    named!(
        parse<Self>,
        do_parse!(
            name: take_str!(8)
                >> axis: call!(RotationAxis::parse)
                >> position: call!(Vec3::parse)
                >> relative_to: take_str!(8)
                >> v_unk1: call!(Vec3::parse)
                >> bbox_size: call!(Vec3::parse)
                >> u1: le_f32
                >> flag: le_u32
                >> u3: le_u32
                >> u4: le_u32
                >> v_unk2: call!(Vec3::parse)
                >> u5: le_f32
                >> (Self {
                    name: name.into(),
                    axis,
                    position,
                    relative_to: relative_to.into(),
                    v_unk1,
                    bbox_size,
                    u1,
                    flag,
                    u3
                })
        )
    );
}

/* SDF Tags */
#[derive(Clone)]
pub struct SDFC {
    pub name: String,
    pub unk: u32,
    pub lods: Vec<f32>,
    pub health: u32,
    pub xdf_name: String,
    pub death_sound_name: String,
}
impl super::common::Parsable<Self> for SDFC {
    named!(
        parse<Self>,
        do_parse!(
            name: take_str!(16)
                >> unk: le_u32
                >> lods: count!(le_f32, 5)
                >> health: le_u32
                >> xdf_name: take_str!(13)
                >> death_sound_name: take_str!(13)
                >> (Self {
                    name: name.into(),
                    unk,
                    lods,
                    health,
                    xdf_name: xdf_name.into(),
                    death_sound_name: death_sound_name.into()
                })
        )
    );
}
#[derive(Clone)]
pub struct SGEOPart {
    pub geo_part: GEOPart,
    pub u4: u32,
    pub v_unk2: Vec3,
    pub u5: f32,
}
impl super::common::Parsable<Self> for SGEOPart {
    named!(
        parse<Self>,
        do_parse!(
            geo_part: call!(GEOPart::parse)
                >> u4: le_u32
                >> v_unk2: call!(Vec3::parse)
                >> u5: le_f32
                >> (Self {
                    geo_part,
                    u4,
                    v_unk2,
                    u5
                })
        )
    );
}

pub struct LodLevel {
    pub lod_parts: Vec<SGEOPart>,
    pub destroyed_parts: Vec<SGEOPart>,
}
impl LodLevel {
    named_args!(
        parse(num_parts: u32)<Self>,
        do_parse!(
            lod_parts: count!(SGEOPart::parse, num_parts as usize)
            >> destroyed_parts: count!(SGEOPart::parse, num_parts as usize)
            >> (Self {
                lod_parts,
                destroyed_parts
            })
        )
    );
}

pub struct SGEO {
    pub num_parts: u32,
    pub lod_levels: Vec<LodLevel>,
}
impl super::common::Parsable<Self> for SGEO {
    named!(
        parse<Self>,
        do_parse!(
            num_parts: le_u32
                >> lod_levels_1: call!(LodLevel::parse, num_parts)
                >> lod_levels_2: call!(LodLevel::parse, num_parts)
                >> lod_levels_3: call!(LodLevel::parse, num_parts)
                >> (Self {
                    num_parts: num_parts,
                    lod_levels: vec![lod_levels_1, lod_levels_2, lod_levels_3]
                })
        )
    );
}

pub struct SCHK {
    pub target_part: String,
    pub num_parts: u32,
    pub parts: Vec<SGEOPart>,
}
impl super::common::Parsable<Self> for SCHK {
    named!(
        parse<Self>,
        do_parse!(
            target_part: take_str!(8)
                >> num_parts: le_u32
                >> parts: count!(SGEOPart::parse, num_parts as usize)
                >> (Self {
                    target_part: target_part.into(),
                    num_parts,
                    parts
                })
        )
    );
}

pub struct SOBJ {
    pub label: String,
    pub mat: Vec<f32>,
    pub position_root: String,
    pub unk: Vec<u8>,
}
impl super::common::Parsable<Self> for SOBJ {
    named!(
        parse<Self>,
        do_parse!(
            label: take_str!(8)
                >> fv: count!(le_f32, 12)
                >> position_root: take_str!(8)
                >> unk: take!(36)
                >> (SOBJ {
                    label: label.into(),
                    mat: fv,
                    position_root: position_root.into(),
                    unk: unk.into()
                })
        )
    );
}
