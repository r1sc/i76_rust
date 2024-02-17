use std::cell::RefCell;
use std::collections::HashMap;
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
use render76::{glam, glow, SceneNode};
use render76::glow::HasContext;

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

    let mut sdf_node = if extension == "sdf" {
        let sdf = vfs.load::<SDF>(model_filename).expect("Failed to load SDF");
        render76::build_static_sdf(&gl, &vfs, &sdf).expect("Failed to build scene nodes")
    } else {
        panic!("Unknown file type");
    };

    let vertex_src = r"#version 330 core
layout(location = 0) in vec3 a_position;
layout(location = 1) in vec2 a_uv;
layout(location = 2) in vec3 a_normal;
layout(location = 3) in vec3 a_color;

out vec2 v_uv;
out vec3 v_normal;
out vec3 v_color;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

void main() {
    mat4 model_view = u_view * u_model;
    gl_Position = u_projection * model_view * vec4(a_position, 1.0);
    v_uv = a_uv;
    v_normal = mat3(model_view) * a_normal;
    v_color = a_color;
}";

    let fragment_src = r"#version 330 core
precision highp float;
in vec2 v_uv;
in vec3 v_normal;
in vec3 v_color;

uniform vec3 u_light;
uniform vec3 u_ambient;
uniform mat4 u_view;

out vec4 color;

void main() {
    vec3 normal = normalize(v_normal);
    vec3 light_dir = normalize(u_light);
    float diff = max(dot(normal, light_dir), 0.0);
    vec3 diffuse = vec3(diff, diff, diff);
    vec3 ambient = u_ambient;
    color = vec4(diffuse + ambient, 1.0);
}";

    let shader_program = render76::shader::ShaderProgram::load(&gl, vertex_src, fragment_src);
    shader_program.use_program(&gl);

    unsafe {
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.enable(render76::glow::DEPTH_TEST);
        gl.disable(render76::glow::CULL_FACE);
    }

    let compute_projection_matrix = |width, height| {
        glam::Mat4::perspective_lh(45.0, width as f32 / height as f32, 0.1, 1000.0)
    };
    
    let (width, height) = window.get_framebuffer_size();
    let mut projection_matrix = compute_projection_matrix(width, height);
    let camera_position = glam::Vec3::new(0.0, 0.0, 50.0);
    let mut view_matrix = glam::Mat4::from_translation(camera_position);

    let u_model = shader_program.get_uniform_location(&gl, "u_model").expect("Failed to get u_model");
    let u_view = shader_program.get_uniform_location(&gl, "u_view").expect("Failed to get u_view");
    let u_projection = shader_program.get_uniform_location(&gl, "u_projection").expect("Failed to get u_projection");
    let u_light = shader_program.get_uniform_location(&gl, "u_light").expect("Failed to get u_light");
    let u_ambient = shader_program.get_uniform_location(&gl, "u_ambient").expect("Failed to get u_ambient");

    let light_dir = vec3(1.0, -1.0, 1.0);

    unsafe {
        gl.uniform_matrix_4_f32_slice(Some(&u_projection), false, projection_matrix.as_ref());

        gl.uniform_3_f32_slice(Some(&u_light), light_dir.as_ref());
        gl.uniform_3_f32(Some(&u_ambient), 0.3, 0.3, 0.3);
    }

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => window.set_should_close(true),
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    projection_matrix = compute_projection_matrix(width, height);

                    unsafe {
                        gl.viewport(0, 0, width, height);
                        gl.uniform_matrix_4_f32_slice(Some(&u_projection), false, projection_matrix.as_ref());
                    }
                }
                _ => (),
            }
        }

        view_matrix *= glam::Mat4::from_rotation_y(0.01);

        unsafe {
            gl.clear(render76::glow::COLOR_BUFFER_BIT | render76::glow::DEPTH_BUFFER_BIT);
            gl.uniform_matrix_4_f32_slice(Some(&u_view), false, view_matrix.as_ref());
            
        }

        for node in &sdf_node {
            render_node(node, &gl, glam::Mat4::IDENTITY, &u_model);            
        }

        window.swap_buffers();
    }

    Ok(())
}

fn render_node(node: &SceneNode, gl: &glow::Context, mut model_matrix: glam::Mat4, loc: &glow::UniformLocation) {
    unsafe {
        gl.uniform_matrix_4_f32_slice(Some(loc), false, model_matrix.as_ref());
    }

    if let Some(mesh) = &node.mesh {
        unsafe {
            gl.bind_vertex_array(Some(mesh.vao));
            for submesh in &mesh.submeshes {
                gl.draw_elements(
                    render76::glow::TRIANGLES,
                    submesh.index_count as i32,
                    render76::glow::UNSIGNED_SHORT,
                    submesh.index_start as i32 * std::mem::size_of::<u16>() as i32,
                );
            }
        }
    }

    model_matrix *= glam::Mat4::from_translation(node.local_position);
    model_matrix *= glam::Mat4::from_quat(node.local_rotation);
    for child in &node.children {
        render_node(child, gl, model_matrix, loc);
    }
}