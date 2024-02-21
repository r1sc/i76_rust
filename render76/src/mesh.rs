use std::{collections::HashMap, path::Path};

use glam::{vec2, vec3, Vec2, Vec3, Vec4Swizzles};
use glow::HasContext;
use lib76::fileparsers::geo::{Geo, GeoFace, GeoVertexRef};

use crate::{caches::TMTCache, mem_utils::slice_to_u8_slice, RenderMode};

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct VertexData {
    position: Vec3,
    uv: Vec2,
    normal: Vec3,
    color: Vec3,
}

impl Eq for VertexData {}

fn hash_vec2(v: Vec2, state: &mut impl std::hash::Hasher) {
    state.write(&v.x.to_le_bytes());
    state.write(&v.y.to_le_bytes());
}

fn hash_vec3(v: Vec3, state: &mut impl std::hash::Hasher) {
    state.write(&v.x.to_le_bytes());
    state.write(&v.y.to_le_bytes());
    state.write(&v.z.to_le_bytes());
}

impl std::hash::Hash for VertexData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        hash_vec3(self.position, state);
        hash_vec2(self.uv, state);
        hash_vec3(self.normal, state);
        hash_vec3(self.color, state);
    }
}

pub struct Mesh {
    pub vao: glow::VertexArray,
    pub vertex_buffer: glow::Buffer,
    pub index_buffer: glow::Buffer,
    pub submeshes: Vec<Submesh>,
}

pub struct Submesh {
    pub index_start: usize,
    pub index_count: usize,
    pub texture_name: String,
}

pub struct VerticesIndicesSubmeshes {
    vertices: Vec<VertexData>,
    indices: Vec<u16>,
    submeshes: Vec<Submesh>,
}

pub fn submeshes_from_geo(
    geo: &Geo,
    use_face_normals: bool,
    render_mode: &RenderMode,
    tmt_cache: &mut TMTCache
) -> VerticesIndicesSubmeshes {
    let triangulate_fan = |face: &GeoFace| -> Vec<VertexData> {
        let mut current_color = vec3(1.0, 1.0, 1.0);

        let mut vref_to_vertex_data = |vref: &GeoVertexRef| -> VertexData {
            let vertex = geo.vertices[vref.vertex_index as usize];
            let normal = if use_face_normals {
                face.normal.xyz()
            } else {
                geo.normals[vref.normal_index as usize]
            };

            if face.color.0 != 0 && face.color.1 != 0 && face.color.2 != 0 {
                current_color = vec3(
                    face.color.0 as f32 / 255.0,
                    face.color.1 as f32 / 255.0,
                    face.color.2 as f32 / 255.0,
                );
            }

            VertexData {
                position: vertex,
                uv: vec2(vref.uv.0, vref.uv.1),
                normal: vec3(normal.x, -normal.y, normal.z),
                color: current_color,
            }
        };

        let mut vertices = Vec::new();
        let vrefs = face.vertex_refs.as_slice();
        let first = &vrefs[0];
        for v in vrefs[1..].windows(2) {
            vertices.push(vref_to_vertex_data(first));
            vertices.push(vref_to_vertex_data(&v[0]));
            vertices.push(vref_to_vertex_data(&v[1]));
        }

        vertices
    };

    let mut faces_by_texture_name: HashMap<String, Vec<&GeoFace>> = HashMap::new();

    for face in &geo.faces {
        // remove any extension from face.texture_name
        let stripped_texture_name = Path::new(&face.texture_name)
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap();

        let texture_name = match render_mode {
            RenderMode::SGeo => stripped_texture_name,
            RenderMode::Vehicle(vtf) => {
                if stripped_texture_name == "V1 BO DY" {
                    ""
                } else if stripped_texture_name.starts_with("V1") {
                    let vtf_part_no =
                        lib76::fileparsers::vtf::car_texture_name_to_vtf_loc(stripped_texture_name);

                    &vtf.vtfc.parts[vtf_part_no as usize][..]                    
                } else {
                    stripped_texture_name
                }
            }
        };

        let texture_name = if texture_name.ends_with(".TMT") || texture_name.ends_with(".tmt") {
            let tmt = tmt_cache
                .get(texture_name)
                .unwrap_or_else(|_| panic!("Cannot find TMT: {}", texture_name));
            &tmt.filenames[0][0][..]
        } else {
            texture_name
        };

        if texture_name.ends_with("TMT") {
            panic!("What");
        }

        let faces = faces_by_texture_name
            .entry(texture_name.to_string())
            .or_default();
        faces.push(face);
    }

    let mut vertices: Vec<VertexData> = Vec::new();
    let mut submeshes: Vec<Submesh> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let mut i = 0;

    for (texture_name, faces) in faces_by_texture_name {
        let index_start = i;

        for face in faces {
            let triangulated_face_vertices = triangulate_fan(face);
            for v in triangulated_face_vertices {
                vertices.push(v);
                indices.push(i as u16);
                i += 1;
            }
        }
        let index_count = i - index_start;

        submeshes.push(Submesh {
            index_start,
            index_count,
            texture_name,
        });
    }

    VerticesIndicesSubmeshes {
        vertices,
        indices,
        submeshes,
    }
}

impl Mesh {
    pub fn from_submeshes(
        gl: &glow::Context,
        vis: VerticesIndicesSubmeshes,
    ) -> Result<Self, String> {
        let vao = unsafe { gl.create_vertex_array()? };
        let vertex_buffer = unsafe { gl.create_buffer()? };
        let index_buffer = unsafe { gl.create_buffer()? };

        unsafe {
            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                slice_to_u8_slice(&vis.vertices),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                slice_to_u8_slice(&vis.indices),
                glow::STATIC_DRAW,
            );

            let stride = std::mem::size_of::<VertexData>() as i32;

            // Position (vec3)
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);

            // Uv (vec2)
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                stride,
                std::mem::size_of::<Vec3>() as i32,
            );

            // Normal (vec3)
            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(
                2,
                3,
                glow::FLOAT,
                false,
                stride,
                std::mem::size_of::<Vec3>() as i32 + std::mem::size_of::<Vec2>() as i32,
            );

            // Color (vec3)
            gl.enable_vertex_attrib_array(3);
            gl.vertex_attrib_pointer_f32(
                3,
                3,
                glow::FLOAT,
                false,
                stride,
                std::mem::size_of::<Vec3>() as i32
                    + std::mem::size_of::<Vec2>() as i32
                    + std::mem::size_of::<Vec3>() as i32,
            );

            gl.bind_vertex_array(None);
        }

        Ok(Mesh {
            vao,
            vertex_buffer,
            index_buffer,
            submeshes: vis.submeshes,
        })
    }
}
