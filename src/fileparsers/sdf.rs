use super::bwd2::{BWD2Chunk, SCHK, SDFC, SGEO, SOBJ};

// use std::collections::HashMap;

// use nom::{named};
// use nom::combinator::map;

// use super::bwd2::BWD2Chunk;
// use super::{bwd2::{SCHK, SDFC, SGEO, SOBJ, bwd2_parser}, common::Parsable};

// pub type SDF = HashMap<String, BWD2Chunk>;

// impl Parsable<Self> for SDF {
//     named!(parse<Self>, bwd2_parser)
// }

pub struct SDF {
    pub sdfc: SDFC,
    pub sobj: SOBJ,
    pub sgeo: SGEO,
    pub schks: Vec<SCHK>
}

impl SDF {
    pub fn new(tags: &mut Vec<BWD2Chunk>) -> Result<SDF, &'static str> {
        let mut schks: Vec<SCHK> = Vec::new();
        let mut sobj: Option<SOBJ> = None;
        let mut sgeo: Option<SGEO> = None;
        let mut sdfc: Option<SDFC> = None;


        for tag in tags.drain(..) {
            match tag {
                BWD2Chunk::ChunkSCHK(v) => { schks.push(v); }
                BWD2Chunk::ChunkSOBJ(v) => { sobj = Some(v); }
                BWD2Chunk::ChunkSGEO(v) => { sgeo = Some(v); }
                BWD2Chunk::ChunkSDFC(v) => { sdfc = Some(v); }
                BWD2Chunk::ChunkEXIT => {}
                BWD2Chunk::ChunkBWD2 => {}
                BWD2Chunk::ChunkREV => {}
                BWD2Chunk::ChunkUnk(_, _) => {}
            }
        }

        match (sdfc, sobj, sgeo) {
            (Some(sdfc), Some(sobj), Some(sgeo)) => Ok(SDF { sdfc, sgeo, sobj, schks }),
            _ => Err("Some tags missing in SDF")
        }
    }
}