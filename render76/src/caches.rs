use anyhow::bail;
use glow::HasContext;
use lib76::{
    fileparsers::{act::ACT, cbk::CBK, geo::Geo, map::MAP, tmt::TMT, vqm::VQM},
    virtual_fs::VirtualFS,
};

use crate::{cache::{self, FileCache}, mem_utils};

pub fn build_cbk_cache(vfs: &VirtualFS) -> FileCache<CBK> {
    cache::FileCache::new(|name| Ok(vfs.load(name)?))
}

fn load_gl_texture(gl: &glow::Context, width: u32, height: u32, rgba_texture: &[u32]) -> glow::Texture {
    unsafe {
        let texture = gl.create_texture().expect("Failed to create texture");
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
        gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, width as i32, height as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, Some(mem_utils::slice_to_u8_slice(rgba_texture)));
        texture
    }
}

pub type TextureCache<'a> = FileCache<'a, glow::Texture>;
pub fn build_texture_cache<'a>(
    gl: &'a glow::Context,
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

        if vfs.exists(&vqm_path) {
            let vqm: VQM = vfs
                .load(&vqm_path)
                .unwrap_or_else(|_| panic!("Failed to load {}", vqm_path));
            let cbk = cbk_cache.get(&vqm.cbk_filename)?;
            Ok(load_gl_texture(
                gl,
                vqm.width,
                vqm.height,
                &vqm.to_rgba_pixels(cbk, act, false),
            ))
        } else if vfs.exists(&map_path) {
            let map: MAP = vfs
                .load(&map_path)
                .unwrap_or_else(|_| panic!("Failed to load {}", map_path));
            Ok(load_gl_texture(
                gl,
                map.width,
                map.height,
                &map.to_rgba_pixels(act, false),
            ))
        } else {
            bail!("Failed to load texture {}", lowercase_name)
        }
    })
}

pub type GeoCache<'a> = FileCache<'a, Geo>;
pub fn build_geo_cache(vfs: &VirtualFS) -> GeoCache {
    cache::FileCache::new(|name| Ok(vfs.load(&format!("{}.geo", name))?))
}

pub type TMTCache<'a> = FileCache<'a, TMT>;
pub fn build_tmt_cache(vfs: &VirtualFS) -> TMTCache {
    cache::FileCache::new(|name| Ok(vfs.load(name)?))
}
