use glam::{vec3, Mat4, Vec3};
use glow::HasContext;

use crate::mem_utils::slice_to_u8_slice;

pub struct TerrainBlock {
    pub vao: glow::VertexArray,
    pub vertex_buffer: glow::Buffer,
    pub index_buffer: glow::Buffer,
    pub num_indices: u32,
}

struct TerrainVertexData {
    pub position: Vec3,
    pub normal: Vec3,
}

impl TerrainBlock {
    pub fn load_heightmap(gl: &glow::Context, heightmap: &[u16]) -> Result<Self, String> {
        let vao = unsafe { gl.create_vertex_array()? };
        let vertex_buffer = unsafe { gl.create_buffer()? };
        let index_buffer = unsafe { gl.create_buffer()? };

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut index = 0;
        for z in 0..128 {
            for x in 0..128 {
                let height = heightmap[z * 128 + x] as f32;
                let v = TerrainVertexData {
                    position: vec3(x as f32, height / 4096.0, z as f32),
                    normal: vec3(0.0, 1.0, 0.0),
                };

                vertices.push(v);

                if z < 127 && x < 127 {
                    indices.push(index);
                    indices.push(index + 1);
                    indices.push(index + 129);

                    indices.push(index);
                    indices.push(index + 129);
                    indices.push(index + 128);
                }
                index += 1;
            }
        }

        unsafe {
            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                slice_to_u8_slice(&vertices),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                slice_to_u8_slice(&indices),
                glow::STATIC_DRAW,
            );

            let stride = std::mem::size_of::<TerrainVertexData>() as i32;

            // Position (vec3)
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);

            // Normal (vec3)
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                stride,
                std::mem::size_of::<Vec3>() as i32,
            );

            gl.bind_vertex_array(None);
        }

        Ok(Self {
            vao,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        })
    }

    pub fn render(
        &self,
        gl: &glow::Context,
        model_matrix: glam::Mat4,
        view_matrix: glam::Mat4,
        u_modelview: &glow::UniformLocation,
        u_normal: &glow::UniformLocation,
        texture: glow::Texture
    ) {
        unsafe {
            let modelview = view_matrix * model_matrix;
            gl.uniform_matrix_4_f32_slice(Some(u_modelview), false, modelview.as_ref());

            let normalmatrix = modelview.inverse().transpose();
            gl.uniform_matrix_4_f32_slice(Some(u_normal), false, normalmatrix.as_ref());
        }

        unsafe {
            gl.bind_vertex_array(Some(self.vao));

            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.draw_elements(
                glow::TRIANGLES,
                self.num_indices as i32,
                glow::UNSIGNED_SHORT,
                0,
            );
        }
    }
}
