use std::path::Path;
use std::rc::Rc;

use glfw::Context;
use lib76::fileparsers::act::ACT;
use lib76::fileparsers::cbk::CBK;
use lib76::fileparsers::geo::Geo;
use lib76::fileparsers::map::MAP;
use lib76::fileparsers::sdf::SDF;
use lib76::fileparsers::vqm::VQM;
use lib76::geo_graph::{self, GeoNode};
use lib76::virtual_fs::VirtualFS;
use lib76::zfs_archive::ZFSArchive;
use render76::glam::{vec3, vec4};
use render76::glow::HasContext;
use render76::{glam, glow, SceneNode};

fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 3 {
        println!("usage: modelviewer <game-path> <model-filename>");
        return Ok(());
    }

    // Init GL

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::Samples(Some(16)));

    let (mut window, events) = glfw
        .create_window(800, 600, "modelviewer", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    let gl =
        unsafe { render76::glow::Context::from_loader_function(|s| window.get_proc_address(s)) };

    // Load assets

    let game_path = Path::new(&args[1]);
    let vfs = VirtualFS {
        disk_folders: vec![game_path.join("addon"), game_path.join("miss16")],
        zfs_archive: Some(ZFSArchive::new(&game_path.join("I76.ZFS"))?),
    };

    let load_geo = |name: &str| -> Rc<Geo> {
        let filename = format!("{}.geo", name);
        Rc::new(vfs.load::<Geo>(&filename).expect("Failed to load geo"))
    };

    let act: ACT = vfs.load("t01.act")?;

    let model_filename = &args[2];
    let extension = Path::new(model_filename)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();

    let use_face_normals = true;

    let sdf_node = if extension == "sdf" {
        let sdf = vfs.load::<SDF>(model_filename).expect("Failed to load SDF");
        render76::SceneNode::from_sdf(&gl, &vfs, &sdf, use_face_normals)
            .expect("Failed to build scene nodes")
    } else {
        panic!("Unknown file type");
    };

    let vertex_src =
        std::fs::read_to_string("shaders/directional.vs").expect("Failed to read vertex shader");
    let fragment_src =
        std::fs::read_to_string("shaders/directional.fs").expect("Failed to read fragment shader");
    let shader_program = render76::shader::ShaderProgram::load(&gl, &vertex_src, &fragment_src);
    shader_program.use_program(&gl);

    unsafe {
        gl.clear_color(0.39, 0.58, 0.93, 1.0);
        gl.enable(render76::glow::DEPTH_TEST);
        gl.disable(render76::glow::CULL_FACE);
    }

    let compute_projection_matrix =
        |width, height| glam::Mat4::perspective_lh(45.0, width as f32 / height as f32, 0.1, 1000.0);

    let (width, height) = window.get_framebuffer_size();
    let mut projection_matrix = compute_projection_matrix(width, height);
    let camera_position = glam::Vec3::new(0.0, -2.0, 10.0);
    let view_matrix = glam::Mat4::from_translation(camera_position);

    let u_model = shader_program
        .get_uniform_location(&gl, "u_model")
        .expect("Failed to get u_model");
    let u_view = shader_program
        .get_uniform_location(&gl, "u_view")
        .expect("Failed to get u_view");
    let u_projection = shader_program
        .get_uniform_location(&gl, "u_projection")
        .expect("Failed to get u_projection");

    unsafe {
        gl.uniform_matrix_4_f32_slice(Some(&u_projection), false, projection_matrix.as_ref());
    }

    let mut cbk_cache = render76::caches::build_cbk_cache(&vfs);
    let mut texture_cache = render76::caches::build_texture_cache(&gl, &vfs, &mut cbk_cache, &act);
    let mut model_matrix = glam::Mat4::IDENTITY;

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    projection_matrix = compute_projection_matrix(width, height);

                    unsafe {
                        gl.viewport(0, 0, width, height);
                        gl.uniform_matrix_4_f32_slice(
                            Some(&u_projection),
                            false,
                            projection_matrix.as_ref(),
                        );
                    }
                }
                _ => (),
            }
        }

        // view_matrix *= glam::Mat4::from_rotation_y(0.01);
        model_matrix *= glam::Mat4::from_rotation_y(0.01);

        unsafe {
            gl.clear(render76::glow::COLOR_BUFFER_BIT | render76::glow::DEPTH_BUFFER_BIT);
            gl.uniform_matrix_4_f32_slice(Some(&u_view), false, view_matrix.as_ref());
        }

        for node in &sdf_node {
            node.render(&gl, model_matrix, &u_model, &mut texture_cache);
        }

        window.swap_buffers();
    }

    Ok(())
}
