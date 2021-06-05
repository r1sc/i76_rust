use std::ffi::c_void;

use crate::gl::types::GLuint;

use super::gl;

pub fn gen_texture(width: u32, height: u32) -> GLuint {
    let mut texture: GLuint = 0;
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    }
    texture
}

pub fn update_texture(width: u32, height: u32, rgba_texture: &[u32]) {
    const GL_BGRA_EXT: u32 = 0x80E1;
    unsafe {
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32, 0, GL_BGRA_EXT, gl::UNSIGNED_BYTE, rgba_texture.as_ptr() as *const c_void);
    }
}

pub fn load_gl_texture(width: u32, height: u32, rgba_texture: &[u32]) -> u32 {    
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