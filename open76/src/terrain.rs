use lib76::fileparsers::ter::TER;

use crate::gl;

pub fn get_block_position_at(x: f32, z: f32) -> (u32, u32) {
    ((x as u32) / 640, (z as u32) / 640)
}

pub fn get_terrain_block_at<'a, 'b>(
    ter: &'b TER,
    zone_references: &'a Vec<Vec<u8>>,
    x: u32,
    z: u32,
) -> Option<&'b Vec<u16>> {
    let block_ref = zone_references.get(x as usize)?.get(z as usize)?;
    let points = ter.entries.get(*block_ref as usize)?;
    Some(points)
}

pub fn render_block(points: Option<&Vec<u16>>) {
    let get_point = |x: u32, z: u32| match points {
        Some(points) => {
            let p = match points.get((z * 128 + x) as usize) {
                Some(p) => *p,
                None => 0xff,
            };

            let height = p & 0xfff;
            let height_unscaled = (height as f32) / 4096.0;
            let height_scaled = height_unscaled * 100.0;
            height_scaled
        }
        None => 0.0,
    };

    unsafe {
        for z in 0..127 {
            gl::Begin(gl::TRIANGLE_STRIP);
            for x in 0..128 {
                let p1 = get_point(x, z);
                let p2 = get_point(x, z + 1);
                gl::TexCoord2f(x as f32, z as f32);
                gl::Vertex3f(x as f32, p1, z as f32);
                gl::TexCoord2f(x as f32, (z + 1) as f32);
                gl::Vertex3f(x as f32, p2, (z + 1) as f32);
            }
            gl::End();
        }
    }
}

// fn render_terrain(camera_x: f32, camera_z: f32, ter: &TER, zone_references: &Vec<Vec<u8>>) {
//     let (block_position_x, block_position_z) = get_block_position_at(camera_x, camera_z);
//     todo!()
// }
