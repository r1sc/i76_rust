use lib76::{
    fileparsers::{bwd2::SGEOPart, common::RotationAxis, Geo},
    math::Vec3,
    virtual_fs,
};

pub struct GeoNode {
    pub name: String,
    pub geo: Geo,
    pub local_position: Vec3,
    pub axis: RotationAxis,
    pub children: Vec<GeoNode>,
}

pub fn from<'a>(parts: &Vec<SGEOPart>) -> Result<Vec<GeoNode>, std::io::Error> {
    let mut root_children: Vec<GeoNode> = vec![];

    for part in parts {
        let geo: Geo = virtual_fs::load(&format!("E:/i76/extracted/{}.geo", part.geo_part.name))?;

        let node = GeoNode {
            geo,
            name: part.geo_part.name.clone(),
            local_position: part.geo_part.position,
            axis: part.geo_part.axis,
            children: vec![],
        };

        if part.geo_part.relative_to == "WORLD" {
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
            match find_parent(&mut root_children, &part.geo_part.relative_to[..]) {
                Some(parent) => {
                    parent.children.push(node);
                }
                None => {
                    panic!("Cannot find parent {}", &part.geo_part.relative_to[..])
                }
            }
        }
    }

    Ok(root_children)
}
