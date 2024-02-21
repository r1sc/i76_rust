use std::rc::Rc;

use glam::Vec3;

use crate::fileparsers::{bwd2::GEOPart, common::RotationAxis, geo::Geo};

#[derive(Clone)]
pub struct GeoNode {
    pub name: String,
    pub geo: Rc<Geo>,
    pub local_position: Vec3,
    pub axis: RotationAxis,
    pub children: Vec<GeoNode>,
}

impl GeoNode {
    pub fn from_geopart(part: &GEOPart, geo_loader: impl Fn(&str) -> Rc<Geo>) -> Self {
        let geo = geo_loader(&part.name);

        Self {
            geo: geo.clone(),
            name: part.name.clone(),
            local_position: part.position,
            axis: part.axis,
            children: vec![],
        }
    }
}

pub fn from<'a, F>(
    parts: impl Iterator<Item = &'a GEOPart>,
    geo_loader: F,
) -> Result<Vec<GeoNode>, std::io::Error>
where
    F: Fn(&str) -> Rc<Geo>,
{
    let mut root_children: Vec<GeoNode> = vec![];

    for part in parts {
        if part.name == "NULL" {
            continue;
        }
        let geo = geo_loader(&part.name);

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

                    if let Some(p) = find_parent(&mut parent.children, relative_to) {
                        return Some(p);
                    }
                }
                None
            }
            match find_parent(&mut root_children, &part.relative_to[..]) {
                Some(parent) => {
                    parent.children.push(node);
                }
                None => {
                    root_children.push(node);
                    println!("Cannot find parent {}", &part.relative_to[..]);
                }
            }
        }
    }

    Ok(root_children)
}
