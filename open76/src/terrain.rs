use glam::vec3;
use lib76::fileparsers::{msn::ZMAP, ter::TER};

use crate::gl::{self, types::GLuint};

pub fn render_terrain(
    surface_texture: GLuint,
    zmap: &ZMAP,
    ter: &TER,
    camera_x: f32,
    camera_y: f32,
    camera_z: f32,
    cells_wide: u32,
) {
    let camera_cell_x = (camera_x as i32) / 5;
    let camera_cell_z = (camera_z as i32) / 5;

    let get_height_at_relative_cell = move |x: i32, z: i32| {
        let absolute_x = (camera_cell_x as i32) + x;
        let absolute_z = (camera_cell_z as i32) + z;

        let block_x = absolute_x / 128;
        let block_z = absolute_z / 128;

        let block = zmap
            .zone_references
            .get((block_z * 80 + block_x) as usize)
            .unwrap_or(&0xFF);
        if *block == 0xFF {
            0.0 // Block out of bounds, return default height
        } else {
            match ter.entries.get(*block as usize) {
                Some(block_heights) => {
                    let relative_x = absolute_x % 128;
                    let relative_z = absolute_z % 128;
                    let heightmap_point = block_heights
                        .get((relative_z * 128 + relative_x) as usize)
                        .expect("Overflowed block, this shouldnt happen");

                    let height = heightmap_point & 0xfff;
                    let height_unscaled = (height as f32) / 4096.0;
                    let height_scaled = height_unscaled * 409.6; // TODO: Investigate
                    height_scaled
                }
                None => 0.0, // Block doesn't exist, return default height
            }
        }
    };

    unsafe {
        gl::Enable(gl::CULL_FACE);
        gl::Enable(gl::ALPHA_TEST);
        gl::Enable(gl::LIGHTING);
        gl::Enable(gl::DEPTH_TEST);

        //gl::Enable(gl::POLYGON_OFFSET_FILL);
        //gl::PolygonOffset(5.0, 1.0);
        gl::BindTexture(gl::TEXTURE_2D, surface_texture);

        let white: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        gl::Materialfv(gl::FRONT_AND_BACK, gl::DIFFUSE, white.as_ptr());

        gl::PushMatrix();
        gl::Translatef(-camera_x, -camera_y, -camera_z);
    }

    let cells_wide_i = cells_wide as i32;
    for z in -cells_wide_i..cells_wide_i {
        unsafe {
            gl::Begin(gl::TRIANGLE_STRIP);
        }
        for x in -cells_wide_i..cells_wide_i {
            let p1 = get_height_at_relative_cell(x, z);

            let world_x = ((camera_cell_x + x) * 5) as f32;
            let world_z1 = ((camera_cell_z + z) * 5) as f32;

            let p2 = get_height_at_relative_cell(x, z + 1);

            let world_z2 = (((camera_cell_z as i32) + z + 1) * 5) as f32;

            let p3 = get_height_at_relative_cell(x + 1, z);

            let v1 = vec3(x as f32, p1, z as f32);
            let v2 = vec3(x as f32, p2, (z + 5) as f32);
            let v3 = vec3((x + 5) as f32, p3, z as f32);

            let v1v2 = v2 - v1;
            let v1v3 = v3 - v1;
            let n = v1v2.cross(v1v3).normalize();

            unsafe {
                gl::TexCoord2f(world_x / 128.0, world_z2 / 128.0);
                gl::Normal3f(n.x, n.y, n.z);
                gl::Vertex3f(world_x, p2, world_z2);

                gl::TexCoord2f(world_x / 128.0, world_z1 / 128.0);
                gl::Normal3f(n.x, n.y, n.z);
                gl::Vertex3f(world_x, p1, world_z1);
            }
        }
        unsafe {
            gl::End();
        }
    }

    unsafe {
        gl::PopMatrix();
        //gl::Disable(gl::POLYGON_OFFSET_FILL);
    }
}
