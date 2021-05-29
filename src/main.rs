mod fileparsers;
mod frustum;
mod gl;
mod math;
mod texture_loader;
mod clut;
mod virtual_fs;

extern crate glfw;

use fileparsers::*;
use math::*;

use glfw::{
    ffi::{glfwSetFramebufferSizeCallback, GLFWwindow},
    Context,
};
use virtual_fs::VirtualFS;


fn render_geo(geo: &geo::Geo) {
    for face in &geo.faces {
        let Vec4(nx, ny, nz, _) = face.normal;
        unsafe {
            gl::Begin(gl::TRIANGLE_FAN);
            for v in &face.vertex_refs {                
                let Vec3(x, y, z) = geo.vertices[v.vertex_index as usize];
                gl::Normal3f(nx, ny, nz);
                gl::Vertex3f(x, y, z);
            }
            gl::End();
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let vfs = VirtualFS::new();
    let geo = vfs.load_geo("E:\\i76\\extracted\\aa2_rmp1.geo");
    // let cbk: cbk::CBK = load("E:\\i76\\extracted\\roadmap3.cbk");
    // let map: map::Map = load("E:\\i76\\extracted\\ao_1sg_0.map");
    // let vqm: vqm::VQM = load("E:\\i76\\extracted\\zhpd6.vqm");
    let sdf = vfs.load_sdf("E:\\i76\\extracted\\aaramp1.sdf");

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
        gl::FrontFace(gl::CW);
        // gl::CullFace(gl::BACK);

        // gl::Enable(gl::DEPTH_TEST);
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

            gl::Translated(0.0, 0.0, -10.0);
            gl::Rotated(an, 0.0, 1.0, 0.0);
            an = an + 1.0;

            render_geo(&geo);

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
