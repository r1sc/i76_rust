use std::{collections::HashMap, fs::File, io::Read};
use crate::fileparsers::bwd2::{BWD2Chunk, bwd2_parser};
use crate::fileparsers::sdf::*;

use crate::fileparsers::{common::Parsable, geo::Geo};

pub struct VirtualFS {
}

fn read_to_buffer(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut f = File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}


fn load<T>(path: &str) -> T where T : Parsable<T> {
    let buffer = read_to_buffer(path).expect("Failed to read file");
    let obj = T::parse(&buffer).expect("Error reading geo!").1;
    obj
}

fn load_bwd2(path: &str) -> Vec<BWD2Chunk> {
    let buffer = read_to_buffer(path).expect("Failed to read file");
    let tags = bwd2_parser(&buffer).expect("Failed to parse BWD2 tags").1;
    tags
}

impl VirtualFS {
    pub fn new() -> Self {
        VirtualFS {
        }
    }

    pub fn load_geo(&self, file_name: &str) -> Geo {
        let geo: Geo = load(file_name);
        geo
    }    

    pub fn load_sdf(&self, file_name: &str) -> SDF {
        SDF::new(&mut load_bwd2(file_name)).expect("Failed to parse SDF")
    }    
}