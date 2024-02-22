use std::path::Path;

use glfw::Context;
use lib76::fileparsers::act::ACT;
use lib76::fileparsers::sdf::SDF;
use lib76::fileparsers::vcf::VCF;
use lib76::virtual_fs::VirtualFS;
use lib76::zfs_archive::ZFSArchive;
use render76::{glam, SceneNodeLoaderParams};
use render76::glow::HasContext;

mod gui;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 3 {
        println!("usage: modelviewer <game-path> <model-filename>");
        return Ok(());
    }

    // Init GLFW

    let mut window_width: u32 = 800;
    let mut window_height: u32 = 600;

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(3));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(3));
    glfw.window_hint(glfw::WindowHint::Samples(Some(16)));

    let (mut window, events) = glfw
        .create_window(
            window_width,
            window_height,
            "modelviewer",
            glfw::WindowMode::Windowed,
        )
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

    #[allow(clippy::arc_with_non_send_sync)]
    // bc egui_glow requires an Arc.. why IDK it's single threaded..
    let gl = std::sync::Arc::new(unsafe {
        render76::glow::Context::from_loader_function(|s| window.get_proc_address(s))
    });

    // Load assets

    let game_path = Path::new(&args[1]);
    let vfs = VirtualFS {
        disk_folders: vec![game_path.join("addon"), game_path.join("miss16")],
        zfs_archive: Some(ZFSArchive::new(&game_path.join("I76.ZFS"))?),
    };

    let act: ACT = vfs.load("t01.act")?;

    let model_filename = &args[2];
    let extension = Path::new(model_filename)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();

    let mut cbk_cache = render76::caches::build_cbk_cache(&vfs);
    let mut texture_cache = render76::caches::build_texture_cache(&gl, &vfs, &mut cbk_cache, &act);
    let mut tmt_cache = render76::caches::build_tmt_cache(&vfs);

    let use_face_normals = false;

    let mut loader_params = SceneNodeLoaderParams {
        gl: &gl,
        vfs: &vfs,
        use_face_normals,
        tmt_cache: &mut tmt_cache,
    };

    let scene_node = match extension {
        "sdf" => {
            let sdf = vfs.load::<SDF>(model_filename).expect("Failed to load SDF");
            render76::SceneNode::from_sdf(&mut loader_params, &sdf)
                .expect("Failed to build scene nodes")
        }
        "vcf" => {
            let vcf: VCF = vfs.load(model_filename)?;
            render76::SceneNode::from_vcf(&mut loader_params, &vcf)
                .expect("Failed to build scene nodes")
        }
        _ => {
            panic!("Unknown file type {}", extension);
        }
    };

    let vertex_src =
        std::fs::read_to_string("shaders/directional.vs").expect("Failed to read vertex shader");
    let fragment_src =
        std::fs::read_to_string("shaders/directional.fs").expect("Failed to read fragment shader");
    let shader_program = render76::shader::ShaderProgram::load(&gl, &vertex_src, &fragment_src);
    shader_program.use_program(&gl);

    let compute_projection_matrix =
        |width, height| glam::Mat4::perspective_lh(45.0, width as f32 / height as f32, 0.1, 1000.0);

    let mut projection_matrix = compute_projection_matrix(window_width, window_height);
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

    let mut gui = gui::Gui::new(gl.clone(), window_width, window_height);

    let mut auto_rotate = true;
    let mut angle = 0.0;

    const DEG2RAD: f32 = std::f32::consts::PI / 180.0;

    while !window.should_close() {
        shader_program.use_program(&gl);

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            gui.handle_event(event.clone());

            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    window_width = width as u32;
                    window_height = height as u32;

                    projection_matrix = compute_projection_matrix(window_width, window_height);

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

        // Handle EGUI
        gui.run(|ctx| {
            egui::Window::new("Options")
                .auto_sized()
                .title_bar(false)
                .show(ctx, |ui| {
                    ui.label(&scene_node.name);
                    ui.checkbox(&mut auto_rotate, "Auto-rotate model");
                    ui.add_enabled(
                        !auto_rotate,
                        egui::Slider::new(&mut angle, 0.0..=360.0).text("Angle"),
                    );
                });
        });

        unsafe {
            gl.clear_color(0.39, 0.58, 0.93, 1.0);
            gl.enable(render76::glow::DEPTH_TEST);
            gl.enable(render76::glow::CULL_FACE);
        }

        if auto_rotate {
            angle += 0.5;
            angle %= 360.0;
        }

        unsafe {
            gl.clear(render76::glow::COLOR_BUFFER_BIT | render76::glow::DEPTH_BUFFER_BIT);
            gl.uniform_matrix_4_f32_slice(Some(&u_view), false, view_matrix.as_ref());
        }

        let model_matrix = glam::Mat4::from_rotation_y(angle * DEG2RAD);

        scene_node.render(&gl, model_matrix, &u_model, &mut texture_cache);

        gui.render(window_width, window_height);

        window.swap_buffers();
    }

    Ok(())
}
