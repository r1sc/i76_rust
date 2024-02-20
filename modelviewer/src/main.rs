use std::path::Path;

use glfw::Context;
use lib76::fileparsers::act::ACT;
use lib76::fileparsers::sdf::SDF;
use lib76::fileparsers::vcf::VCF;
use lib76::virtual_fs::VirtualFS;
use lib76::zfs_archive::ZFSArchive;
use render76::glow::HasContext;
use render76::glam;

mod egui_glfw_input;

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
    window.set_mouse_button_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_framebuffer_size_polling(true);
    window.make_current();

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

    let use_face_normals = true;

    let sdf_node = match extension {
        "sdf" => {
            let sdf = vfs.load::<SDF>(model_filename).expect("Failed to load SDF");
            render76::SceneNode::from_sdf(&gl, &vfs, &sdf, use_face_normals, &mut tmt_cache)
                .expect("Failed to build scene nodes")
        }
        "vcf" => {
            let vcf: VCF = vfs.load(model_filename)?;
            render76::SceneNode::from_vcf(&gl, &vfs, &vcf, use_face_normals, &mut tmt_cache)
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

    let mut model_matrix = glam::Mat4::IDENTITY;

    let mut painter =
        egui_glow::Painter::new(gl.clone(), "", Some(egui_glow::ShaderVersion::Es300))
            .expect("Failed to create egui_glow::Painter");

    let egui_ctx = egui::Context::default();
    egui_ctx.set_pixels_per_point(1.0);

    let mut input = egui_glfw_input::EguiInputState::new(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800 as f32, 600 as f32),
        )),
        ..egui::RawInput::default()
    });

    let mut window_width: u32 = 0;
    let mut window_height: u32 = 0;

    while !window.should_close() {
        shader_program.use_program(&gl);
        unsafe {
            gl.clear_color(0.39, 0.58, 0.93, 1.0);
            gl.enable(render76::glow::DEPTH_TEST);
            gl.enable(render76::glow::CULL_FACE);
        }

        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            input.handle_event(event.clone(), &gl, &mut window_width, &mut window_height);

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

        // EGUI
        let full_output = egui_ctx.run(input.input.take(), |ctx| {
            egui::Window::new("Hello").show(ctx, |ui| {
                ui.heading("Hello World!");
            });
        });

        for (id, image_delta) in full_output.textures_delta.set {
            painter.set_texture(id, &image_delta);
        }

        let clipped_primitives =
            egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);

        let (width, height) = window.get_framebuffer_size();
        painter.paint_primitives(
            [width as u32, height as u32],
            full_output.pixels_per_point,
            &clipped_primitives,
        );

        for id in full_output.textures_delta.free {
            painter.free_texture(id);
        }

        unsafe {
            gl.finish();
        }

        window.swap_buffers();
    }

    Ok(())
}
