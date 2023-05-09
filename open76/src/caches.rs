use std::ffi::c_void;

use anyhow::bail;
use lib76::fileparsers::{act::ACT, cbk::CBK, geo::Geo, map::MAP, tmt::TMT, vqm::VQM};

use crate::{
    cache::{self, FileCache},
    gl::{self, types::GLuint},
    virtual_fs::VirtualFS,
};

pub fn build_cbk_cache(vfs: &VirtualFS) -> FileCache<CBK> {
    cache::FileCache::new(|name| vfs.load(name))
}

fn load_gl_texture(width: u32, height: u32, rgba_texture: &[u32]) -> u32 {
    const GL_BGRA_EXT: u32 = 0x80E1;
    let mut texture: u32 = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as i32,
            width as i32,
            height as i32,
            0,
            GL_BGRA_EXT,
            gl::UNSIGNED_BYTE,
            rgba_texture.as_ptr() as *const c_void,
        )
    }
    texture
}

pub type TextureCache<'a> = FileCache<'a, GLuint>;
pub fn build_texture_cache<'a>(
    vfs: &'a VirtualFS,
    cbk_cache: &'a mut FileCache<CBK>,
    act: &'a ACT,
) -> TextureCache<'a> {
    cache::FileCache::new(|name| {
        let lowercase_name = name.to_ascii_lowercase();
        let fixed_name = if lowercase_name.ends_with(".map") {
            &lowercase_name[0..name.len() - 4]
        } else {
            &lowercase_name
        };
        let vqm_path = format!("{}.vqm", fixed_name);
        let map_path = format!("{}.map", fixed_name);

        let tex = match (vfs.exists(&vqm_path), vfs.exists(&map_path)) {
            (true, _) => {
                let vqm: VQM = vfs
                    .load(&vqm_path)
                    .unwrap_or_else(|_| panic!("Failed to load {}", vqm_path));
                let cbk = cbk_cache.get(&vqm.cbk_filename)?;
                anyhow::Ok(load_gl_texture(
                    vqm.width,
                    vqm.height,
                    &vqm.to_rgba_pixels(cbk, act),
                ))
            }
            (_, true) => {
                let map: MAP = vfs
                    .load(&map_path)
                    .unwrap_or_else(|_| panic!("Failed to load {}", map_path));
                anyhow::Ok(load_gl_texture(
                    map.width,
                    map.height,
                    &map.to_rgba_pixels(act),
                ))
            }
            (false, false) => bail!("Failed to load texture {}", lowercase_name),
        }?;

        Ok(tex)
    })
}

pub type GeoCache<'a> = FileCache<'a, Geo>;
pub fn build_geo_cache(vfs: &VirtualFS) -> GeoCache {
    cache::FileCache::new(|name| vfs.load(&format!("{}.geo", name)))
}

pub type TMTCache<'a> = FileCache<'a, TMT>;
pub fn build_tmt_cache(vfs: &VirtualFS) -> TMTCache {
    cache::FileCache::new(|name| vfs.load(name))
}
