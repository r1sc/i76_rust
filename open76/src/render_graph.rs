use std::rc::Rc;

use crate::{
    caches::{GeoCache, TMTCache, TextureCache},
    gl::{self},
};
use glam::{Vec3, Vec4Swizzles};
use lib76::fileparsers::{self, bwd2::GEOPart, common::RotationAxis, vtf::VTF, geo::Geo};

pub struct GeoNode {
    pub name: String,
    pub geo: Rc<Geo>,
    pub local_position: Vec3,
    pub axis: RotationAxis,
    pub children: Vec<GeoNode>,
}


pub fn from<'a>(parts: impl Iterator<Item = &'a GEOPart>, geo_cache: &'a mut GeoCache) -> Result<Vec<GeoNode>, std::io::Error>
    {
        let mut root_children: Vec<GeoNode> = vec![];

        for part in parts {
            if part.name == "NULL" {
                continue;
            }

            geo_cache.get(&part.name[..]).map(|geo| {
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
            }).unwrap();
        }

        Ok(root_children)
    }

pub enum RenderMode {
    SGEO,
    Vehicle(VTF),
}

fn draw_geo(
    geo: &Geo,
    texture_cache: &mut TextureCache,
    tmt_cache: &mut TMTCache,
    use_face_normals: bool,
    render_mode: &RenderMode,
    _ambient_color: &[f32; 4],
) {
    let white: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    for face in &geo.faces {
        unsafe {
            if face.texture_name != "" && face.texture_name != "V1 BO DY.MAP" {
                let texture_name = match render_mode {
                    RenderMode::SGEO => &face.texture_name,
                    RenderMode::Vehicle(vtf) => {
                        if face.texture_name.starts_with("V1") {
                            let vtf_part_no =
                                fileparsers::vtf::car_texture_name_to_vtf_loc(&face.texture_name);

                            let filename = &vtf.vtfc.parts[vtf_part_no as usize][..];
                            if filename.ends_with(".TMT") || filename.ends_with(".tmt") {
                                let tmt = tmt_cache.get(filename).expect(&format!("Cannot find TMT: {}", filename));
                                &tmt.filenames[0][0][..]
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
                    .map(|tex| gl::BindTexture(gl::TEXTURE_2D, **tex))
                    .unwrap();

                // gl::Materialfv(gl::FRONT_AND_BACK, gl::AMBIENT, ambient_color.as_ptr());
                gl::Materialfv(gl::FRONT_AND_BACK, gl::DIFFUSE, white.as_ptr());
            } else {
                gl::BindTexture(gl::TEXTURE_2D, 0);
                // gl::Materialfv(gl::FRONT_AND_BACK, gl::AMBIENT, white.as_ptr());
                // gl::Materialfv(gl::FRONT_AND_BACK, gl::DIFFUSE, white.as_ptr());
                let diffuse = face.color.to_vec3().extend(0.0).to_array();
                gl::Materialfv(gl::FRONT_AND_BACK, gl::DIFFUSE, diffuse.as_ptr());
            }

            if (face.flags.1 & 4) == 4 {
                gl::Enable(gl::ALPHA_TEST);
            } else {
                gl::Disable(gl::ALPHA_TEST);
            }

            gl::Begin(gl::TRIANGLE_FAN);
            for v in &face.vertex_refs {
                let vert = geo.vertices[v.vertex_index as usize];
                let normal = if use_face_normals {
                    face.normal.xyz()
                } else {
                    geo.normals[v.normal_index as usize]
                };
                let (u, v) = v.uv;

                gl::TexCoord2f(u, v);
                gl::Normal3f(normal.x, -normal.y, normal.z);
                gl::Vertex3f(vert.x, vert.y, vert.z);
            }
            gl::End();
        }
    }
}

pub fn draw_graph(
    root_children: &Vec<GeoNode>,
    texture_cache: &mut TextureCache,
    tmt_cache: &mut TMTCache,
    use_face_normals: bool,
    render_mode: &RenderMode,
    ambient_color: &[f32; 4],
) {
    for part in root_children {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(
                part.local_position.x,
                part.local_position.y,
                part.local_position.z,
            );
            draw_geo(
                &part.geo,
                texture_cache,
                tmt_cache,
                use_face_normals,
                render_mode,
                ambient_color,
            );

            draw_graph(
                &part.children,
                texture_cache,
                tmt_cache,
                use_face_normals,
                render_mode,
                ambient_color,
            );

            gl::PopMatrix();
        }
    }
}
