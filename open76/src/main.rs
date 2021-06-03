mod frustum;
mod gl;
mod render_graph;
mod texture_loader;

extern crate glfw;

use std::{fs::File, io::BufReader};

use glfw::{
    ffi::{glfwSetFramebufferSizeCallback, GLFWwindow},
    Context,
};

use lib76::fileparsers;
use lib76::math::*;
use lib76::virtual_fs;
use render_graph::{ GeoNode};
use rodio::{Decoder, OutputStream, Source};

fn render_geo(geo: &fileparsers::Geo) {
    for face in &geo.faces {
        let Vec4(nx, ny, nz, _) = face.normal;
        unsafe {
            gl::Begin(gl::TRIANGLE_FAN);
            for v in &face.vertex_refs {
                let Vec3(x, y, z) = geo.vertices[v.vertex_index as usize];
                gl::Normal3f(nx, ny, nz);
                gl::Vertex3f(-x, y, z);
            }
            gl::End();
        }
    }
}

fn render_graph(root_children: &Vec<GeoNode>) {
    for part in root_children {
        let c = part;

        unsafe {
            gl::PushMatrix();
            gl::Translatef(
                -c.local_position.0,
                c.local_position.1,
                c.local_position.2,
            );
            render_geo(&c.geo);

            render_graph(&c.children);

            gl::PopMatrix();
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    // let geo = virtual_fs::load("E:\\i76\\extracted\\aa2_rmp1.geo")?;
    let sdf: fileparsers::SDF = virtual_fs::load("E:/i76/extracted/bddonut1.sdf")?;
    let what = render_graph::from(&sdf.sgeo.lod_levels[0].lod_parts)?;

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let file = BufReader::new(File::open("E:/i76/music/2.mp3").unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle
        .play_raw(source.convert_samples())
        .expect("Couldn't play sound");

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, _events) = glfw
        .create_window(640, 480, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));

    // let rgba_pixels = vqm.to_rgba_pixels(&cbk, &LUT);
    // let gl_texture = texture_loader::load_gl_texture(vqm.width, vqm.height, &rgba_pixels);

    let light_pos = [0.0, 0.0, -50.0, 0.0].as_ptr();
    unsafe {
        unsafe fn resize(w: i32, h: i32) {
            gl::MatrixMode(gl::PROJECTION);
            gl::LoadIdentity();

            let mut matrix = [0.0; 16];
            frustum::glh_perspectivef2(&mut matrix, 60.0, w as f32 / h as f32, 0.1, 100.0);
            gl::LoadMatrixf(matrix.as_ptr());
            gl::MatrixMode(gl::MODELVIEW);
        }

        extern "C" fn resize_callback(_: *mut GLFWwindow, w: i32, h: i32) {
            unsafe {
                gl::Viewport(0, 0, w, h);
                resize(w, h);
            }
        }

        let (w, h) = window.get_size();
        resize(w, h);

        glfwSetFramebufferSizeCallback(window.window_ptr(), Some(resize_callback));

        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::BACK);
        gl::FrontFace(gl::CCW);

        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::TEXTURE_2D);

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);

        gl::Enable(gl::LIGHTING);
        gl::Enable(gl::LIGHT0);
        gl::Lightfv(gl::LIGHT0, gl::DIFFUSE, [1.0, 1.0, 1.0, 1.0].as_ptr());
        gl::Lightfv(gl::LIGHT0, gl::AMBIENT, [0.2, 0.2, 0.2, 1.0].as_ptr());
    }

    let mut an = 0.0;
    while !window.should_close() {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::LoadIdentity();
            gl::Lightfv(gl::LIGHT0, gl::POSITION, light_pos);

            gl::Translated(0.0, 0.0, -50.0);
            gl::Rotated(an, 0.0, 1.0, 0.0);
            an = an + 1.0;

            //render_geo(&geo);
            render_graph(&what);

            // gl::BindTexture(gl::TEXTURE_2D, gl_texture);
            // gl::Begin(gl::QUADS);
            //     gl::TexCoord2f(0.0, 0.0);
            //     gl::Vertex3f(-1.0, 1.0, 0.0);

            //     gl::TexCoord2f(1.0, 0.0);
            //     gl::Vertex3f(1.0, 1.0, 0.0);

            //     gl::TexCoord2f(1.0, 1.0);
            //     gl::Vertex3f(1.0, -1.0, 0.0);

            //     gl::TexCoord2f(0.0, 1.0);
            //     gl::Vertex3f(-1.0, -1.0, 0.0);
            // gl::End();
        }

        glfw.poll_events();
        // for (_, event) in glfw::flush_messages(&events) {
        //     handle_window_event(&mut window, event);
        // }

        window.swap_buffers();
    }

    return Ok(());
}
