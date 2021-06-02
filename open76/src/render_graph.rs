use std::collections::HashMap;

use lib76::{
    fileparsers::{bwd2::SGEOPart, common::RotationAxis, Geo},
    math::Vec3,
    virtual_fs,
};

pub struct GeoNode {
    pub geo: Geo,
    pub local_position: Vec3,
    pub axis: RotationAxis,
    pub children_indices: Vec<usize>,
}

pub struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            items: vec![],
        }
    }

    pub fn add(&mut self, data: T) -> usize {
        let new_index = self.items.len();
        self.items.push(data);
        new_index
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        &mut self.items[index]
    }

    pub fn get(&self, index: usize) -> &T {
        &self.items[index]
    }
}

pub fn from<'a>(parts: &Vec<SGEOPart>) -> Result<(Vec<usize>, Arena<GeoNode>), std::io::Error> {
    let mut arena = Arena::<GeoNode>::new();
    let mut cache = HashMap::<&str, usize>::new();
    let mut root_children: Vec<usize> = vec![];
    
    for part in parts {
        let geo: Geo = virtual_fs::load(&format!("E:/i76/extracted/{}.geo", part.geo_part.name))?;

        let node = GeoNode {
            geo,
            local_position: part.geo_part.position,
            axis: part.geo_part.axis,
            children_indices: vec![],
        };

        let new_index = arena.add(node);
        cache.insert(&part.geo_part.name[..], new_index);

        if part.geo_part.relative_to == "WORLD" {
            root_children.push(new_index);
        }
        else {
            let parent_index = cache.get(&part.geo_part.relative_to[..]).unwrap();
            arena.get_mut(*parent_index).children_indices.push(new_index);
        }

    }

    Ok((root_children, arena))
}
