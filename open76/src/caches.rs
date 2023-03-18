use std::{path::Path, ffi::c_void};

use lib76::fileparsers::{act::ACT, cbk::CBK, map::Map, tmt::TMT, vqm::VQM, Geo};

use crate::{
    cache::{self, FileCache},
    gl::{types::GLuint, self},
    virtual_fs,
};

pub fn build_cbk_cache<'a>() -> FileCache<'a, CBK> {
    cache::FileCache::new(|name| {
        virtual_fs::load::<CBK>(&format!("E:/i76/extracted/{}", name)).ok()
    })
}

pub type TextureCache<'a> = FileCache<'a, GLuint>;

fn load_gl_texture(width: u32, height: u32, rgba_texture: &[u32]) -> u32 {    
    const GL_BGRA_EXT: u32 = 0x80E1;
    let mut texture: u32 = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, GL_BGRA_EXT, gl::UNSIGNED_BYTE, rgba_texture.as_ptr() as *const c_void)
    }
    texture
}

pub fn build_texture_cache<'a>(
    cbk_cache: &'a mut FileCache<CBK>,
    act: &'a ACT,
) -> TextureCache<'a> {
    cache::FileCache::new(move |name| {
        let fixed_name = if name.to_ascii_lowercase().ends_with(".map") {
            &name[0..name.len() - 4]
        } else {
            name
        };
        let vqm_path = format!("E:/i76/extracted/{}.vqm", fixed_name);
        let map_path = format!("E:/i76/extracted/{}.map", fixed_name);

        let tex = match (Path::new(&vqm_path).exists(), Path::new(&map_path).exists()) {
            (true, _) => {
                let vqm: VQM =
                    virtual_fs::load(&&vqm_path).expect(&format!("Failed to load {}", vqm_path));
                let cbk = cbk_cache.get(&vqm.cbk_filename)?;
                Some(load_gl_texture(
                    vqm.width,
                    vqm.height,
                    &vqm.to_rgba_pixels(cbk, act),
                ))
            }
            (_, true) => {
                let map: Map =
                    virtual_fs::load(&&&map_path).expect(&format!("Failed to load {}", map_path));
                Some(load_gl_texture(
                    map.width,
                    map.height,
                    &map.to_rgba_pixels(act),
                ))
            }
            (false, false) => None,
        }?;

        Some(tex)
    })
}

pub type GeoCache<'a> = FileCache<'a, Geo>;
pub fn build_geo_cache<'a>() -> GeoCache<'a> {
    cache::FileCache::new(|name| {
        virtual_fs::load::<Geo>(&format!("E:/i76/extracted/{}.geo", name)).ok()
    })
}

pub type TMTCache<'a> = FileCache<'a, TMT>;
pub fn build_tmt_cache<'a>() -> TMTCache<'a> {
    cache::FileCache::new(|name| {
        virtual_fs::load::<TMT>(&format!("E:/i76/extracted/{}", name)).ok()
    })
}
