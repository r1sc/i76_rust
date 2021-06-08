use std::rc::Rc;

use lib76::{fileparsers::{Geo, bwd2::{GEOPart, SGEOPart}, common::RotationAxis}, math::Vec3};

use crate::{cache::FileCache, virtual_fs};

pub struct GeoNode {
    pub name: String,
    pub geo: Rc<Geo>,
    pub local_position: Vec3,
    pub axis: RotationAxis,
    pub children: Vec<GeoNode>,
}

pub fn from<'a, T>(parts: T, geo_cache: &'a mut FileCache<Geo>) -> Result<Vec<GeoNode>, std::io::Error>
where 
T : Iterator<Item=&'a GEOPart>
{
    let mut root_children: Vec<GeoNode> = vec![];

    for part in parts {
        if part.name == "NULL" {
            continue;
        }
        
        let geo = geo_cache.get(&part.name[..]).ok_or(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))?;

        let node = GeoNode {
            geo: geo.clone(),
            name: part.name.clone(),
            local_position: part.position,
            axis: part.axis,
            children: vec![],
        };

        if part.relative_to == "WORLD" {
            root_children.push(node);
        } else {
            fn find_parent<'a>(
                children: &'a mut Vec<GeoNode>,
                relative_to: &str,
            ) -> Option<&'a mut GeoNode> {
                for parent in children {
                    if parent.name == relative_to {
                        return Some(parent);
                    }
                    match find_parent(&mut parent.children, relative_to) {
                        Some(p) => return Some(p),
                        None => {}
                    }
                }
                None
            }
            match find_parent(&mut root_children, &part.relative_to[..]) {
                Some(parent) => {
                    parent.children.push(node);
                }
                None => {
                    root_children.push(node); // panic!("Cannot find parent {}", &part.relative_to[..])
                }
            }
        }
    }

    Ok(root_children)
}
