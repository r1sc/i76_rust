use std::rc::Rc;

use crate::{
    cache::{self, FileCache},
    gl::{self, types::GLuint},
};
use lib76::{
    fileparsers::{
        self,
        bwd2::GEOPart,
        common::{ColorRGB, RotationAxis},
        tmt::TMT,
        vtf::VTF,
        Geo,
    },
    math::Vec3,
};

pub struct GeoNode {
    pub name: String,
    pub geo: Rc<Geo>,
    pub local_position: Vec3,
    pub axis: RotationAxis,
    pub children: Vec<GeoNode>,
}

pub fn from<'a, T>(
    parts: T,
    geo_cache: &'a mut FileCache<Geo>,
) -> Result<Vec<GeoNode>, std::io::Error>
where
    T: Iterator<Item = &'a GEOPart>,
{
    let mut root_children: Vec<GeoNode> = vec![];

    for part in parts {
        if part.name == "NULL" {
            continue;
        }

        let geo = geo_cache.get(&part.name[..]).unwrap();

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
                    root_children.push(node);
                    println!("Cannot find parent {}", &part.relative_to[..]);
                }
            }
        }
    }

    Ok(root_children)
}

pub enum RenderMode {
    SGEO,
    Vehicle(VTF),
}

fn draw_geo(
    geo: &fileparsers::Geo,
    texture_cache: &mut cache::FileCache<GLuint>,
    tmt_cache: &mut cache::FileCache<TMT>,
    use_face_normals: bool,
    render_mode: &RenderMode,
) {
    for face in &geo.faces {
        unsafe {
            if face.texture_name != "" && face.texture_name != "V1 BO DY" {
                let texture_name = match render_mode {
                    RenderMode::SGEO => &face.texture_name,
                    RenderMode::Vehicle(vtf) => {
                        if face.texture_name.starts_with("V1") {
                            let vtf_part_no =
                                fileparsers::vtf::car_texture_name_to_vtf_loc(&face.texture_name);

                            let filename = &vtf.vtfc.parts[vtf_part_no as usize][..];
                            if filename.ends_with(".TMT") || filename.ends_with(".tmt") {
                                let tmt = tmt_cache.get(filename);
                                match tmt {
                                    Some(tmt) => &tmt.filenames[0][0][..],
                                    None => {
                                        panic!("Cannot find TMT file {}", filename)
                                    }
                                }
                            } else {
                                filename
                            }
                        } else {
                            &face.texture_name[..]
                        }
                    }
                };

                texture_cache
                    .get(texture_name)
                    .map(|tex| gl::BindTexture(gl::TEXTURE_2D, **tex));


            } else {
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            if (face.flags.1 & 4) == 4 {
                gl::Enable(gl::ALPHA_TEST);
            } else {
                gl::Disable(gl::ALPHA_TEST);
            }

            let ColorRGB(r, g, b) = face.color;
            gl::Color4f((r as f32) / 255.0, (g as f32) / 255.0, (b as f32) / 255.0, 1.0);

            gl::Begin(gl::TRIANGLE_FAN);
            for v in &face.vertex_refs {
                let Vec3(x, y, z) = geo.vertices[v.vertex_index as usize];
                let Vec3(nx, ny, nz) = if use_face_normals {
                    Vec3(face.normal.0, face.normal.1, face.normal.2)
                } else {
                    geo.normals[v.normal_index as usize]
                };
                let (u, v) = v.uv;

                gl::TexCoord2f(u, v);
                gl::Normal3f(nx, ny, nz);
                gl::Vertex3f(x, y, z);
            }
            gl::End();
        }
    }
}

pub fn draw_graph(
    root_children: &Vec<GeoNode>,
    texture_cache: &mut cache::FileCache<GLuint>,
    tmt_cache: &mut cache::FileCache<TMT>,
    use_face_normals: bool,
    render_mode: &RenderMode,
) {
    for part in root_children {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(
                part.local_position.0,
                part.local_position.1,
                part.local_position.2,
            );
            draw_geo(
                &part.geo,
                texture_cache,
                tmt_cache,
                use_face_normals,
                render_mode,
            );

            draw_graph(
                &part.children,
                texture_cache,
                tmt_cache,
                use_face_normals,
                render_mode,
            );

            gl::PopMatrix();
        }
    }
}
