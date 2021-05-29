use std::usize;

use nom::number::complete::{le_f32, le_u32, le_u8};
use nom::{call, count, do_parse, named, pair, tag};

use super::common::*;
use crate::math::*;

#[derive(Debug)]
pub struct Geo {
    pub name: String,
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<GeoFace>,
}

impl super::common::Parsable<Geo> for Geo {
    named!(
        parse<Geo>,
        do_parse!(
            tag!("OEG")
                >> le_u8
                >> le_u32
                >> name: call!(super::common::cstring, 16)
                >> vertex_count: le_u32
                >> face_count: le_u32
                >> le_u32
                >> vertices: count!(Vec3::parse, vertex_count as usize)
                >> normals: count!(Vec3::parse, vertex_count as usize)
                >> faces: count!(GeoFace::parse, face_count as usize)
                >> (Geo {
                    name,
                    vertices,
                    normals,
                    faces
                })
        )
    );
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
impl GeoFace {
    named!(
        parse<GeoFace>,
        do_parse!(
            index: le_u32
                >> num_vertices: le_u32
                >> color: call!(ColorRGB::parse)
                >> normal: call!(Vec4::parse)
                >> le_u32
                >> flags: flags
                >> texture_name: call!(super::common::cstring, 13)
                >> le_u32
                >> le_u32
                >> vertex_refs: count!(GeoVertexRef::parse, num_vertices as usize)
                >> (GeoFace {
                    index,
                    color,
                    flags,
                    texture_name,
                    normal: normal,
                    vertex_refs
                })
        )
    );
}

#[derive(Debug)]
pub struct GeoVertexRef {
    pub vertex_index: u32,
    pub normal_index: u32,
    pub uv: (f32, f32),
}
impl GeoVertexRef {
    named!(
        parse<GeoVertexRef>,
        do_parse!(
            vertex_index: le_u32
                >> normal_index: le_u32
                >> uv: pair!(le_f32, le_f32)
                >> (GeoVertexRef {
                    vertex_index,
                    normal_index,
                    uv
                })
        )
    );
}

named!(
    flags<(u8, u8, u8)>,
    do_parse!(a: le_u8 >> b: le_u8 >> c: le_u8 >> ((a, b, c)))
);