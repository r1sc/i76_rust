mod cache;
mod caches;
mod camera;
mod gl;
mod render_graph;
mod sky;
//mod smacker_player;
mod terrain;
mod virtual_fs;
mod ivec;

use glam::{Vec3, Vec4};
use glfw::{
    ffi::{glfwSetFramebufferSizeCallback, GLFWwindow},
    Action, Context,
};
use std::time::Instant;

use lib76::fileparsers::{self, act::ACT, msn::MSN, vcf::VCF, vdf::VDF};
use lib76::fileparsers::{ter::TER, vtf::VTF};

use crate::{
    render_graph::RenderMode,
    sky::Sky,
};

fn load_vcf(vcf_filename: &str) -> Result<(VCF, VDF, VTF), std::io::Error> {
    let vcf: fileparsers::vcf::VCF =
        virtual_fs::load(&format!("E:/i76/extracted/{}", vcf_filename))?;
    let vdf: fileparsers::vdf::VDF =
        virtual_fs::load(&format!("E:/i76/extracted/{}", vcf.vcfc.vdf_filename))?;
    let vtf: fileparsers::vtf::VTF =
        virtual_fs::load(&format!("E:/i76/extracted/{}", vcf.vcfc.vtf_filename))?;

    Ok((vcf, vdf, vtf))
}
fn main() -> Result<(), std::io::Error> {
    let mut camera = camera::Camera::new();

    println!("Loading data");
    let msn: MSN = virtual_fs::load("E:/i76/MISSIONS/A01.msn")?;
    let act: ACT = virtual_fs::load(&format!("E:/i76/extracted/{}", &msn.wrld.act_filename))?;
    let ter: TER = virtual_fs::load(&format!("E:/i76/MISSIONS/{}", &msn.tdef.zone.ter_filename))?;

    let mut geo_cache = caches::build_geo_cache();
    let mut cbk_cache = caches::build_cbk_cache();
    let mut texture_cache = caches::build_texture_cache(&mut cbk_cache, &act);
    let mut tmt_cache = caches::build_tmt_cache();

    let objects: Vec<_> = msn
        .odef_objs
        .iter()
        .map(|o| {
            if o.class_id == 1 {
                let (vcf, vdf, vtf) = load_vcf(&format!("{}.vcf", &o.label[..]))?;
                println!(
                    "Building render graph for {} {}",
                    &vdf.vdfc.name, &vcf.vcfc.variant_name
                );
                let vdf_graph =
                    render_graph::from(vdf.vgeo.third_person_parts[0][0].iter(), &mut geo_cache)?;
                Ok((vdf_graph, RenderMode::Vehicle(vtf), o))
            } else {
                let sdf: fileparsers::SDF =
                    virtual_fs::load(&format!("E:/i76/extracted/{}.sdf", &o.label))?;
                println!(
                    "Building render graph for {} ({})",
                    &sdf.sdfc.name, &o.label
                );
                let graph = render_graph::from(
                    sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part),
                    &mut &mut geo_cache,
                )?;
                Ok((graph, RenderMode::SGEO, o))
            }
        })
        .filter_map(|f: Result<_, std::io::Error>| f.ok())
        .collect();

    camera.position = objects
        .iter()
        .find(|o| o.2.label == "vppirna1")
        .map(|o| o.2.position + Vec3::Y * 2.0)
        .unwrap_or(Vec3::ZERO);

    println!("Starting GLFW...");
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, _events) = glfw
        .create_window(800, 600, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.set_key_polling(true);
    window.set_raw_mouse_motion(true);

    gl::load_with(|s| window.get_proc_address(s));

    let light_pos = Vec4::new(5.0, 5.0, 5.0, 0.0).to_array().as_ptr();
    let ambient_color = act.entries[247].to_vec3().extend(1.0).to_array();

    unsafe {
        unsafe fn resize(w: i32, h: i32) {
            gl::MatrixMode(gl::PROJECTION);
            let perspective = glam::Mat4::perspective_lh(60.0f32.to_radians(), w as f32 / h as f32, 0.1, 1000.0);
            gl::LoadMatrixf(&perspective.to_cols_array() as *const _);

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
        gl::FrontFace(gl::CCW);

        gl::Enable(gl::ALPHA_TEST);
        gl::AlphaFunc(gl::GEQUAL, 1.0);

        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::TEXTURE_2D);

        let cc = act.entries[239].to_vec3();
        gl::ClearColor(cc.x, cc.y, cc.z, 1.0);

        gl::Enable(gl::LIGHTING);
        gl::Enable(gl::LIGHT0);
        gl::Lightfv(
            gl::LIGHT0,
            gl::DIFFUSE,
            act.entries[176].to_vec3().extend(1.0).to_array().as_ptr(),
        );
        gl::Lightfv(gl::LIGHT0, gl::AMBIENT, ambient_color.as_ptr());
    }

    println!("...done");

    let surface_texture = **texture_cache
        .get(&msn.wrld.surface_texture_filename)
        .unwrap();
    let sky_texture = **texture_cache.get(&msn.wrld.sky_texture_filename).unwrap();

    let (mut ox, mut oy) = (0.0, 0.0);
    let mut last_time = Instant::now();
    let mut sky = Sky::new();

    while !window.should_close() {
        let now = Instant::now();
        let delta = now.duration_since(last_time);
        last_time = now;
        let delta = delta.as_secs_f32();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::LoadIdentity();

            sky.draw(sky_texture);

            gl::LoadMatrixf(&camera.get_view().to_cols_array() as *const _);
            gl::Lightfv(gl::LIGHT0, gl::POSITION, light_pos);

            terrain::render_terrain(
                surface_texture,
                &msn.tdef.zmap,
                &ter,
                camera.position.x,
				camera.position.y,
                camera.position.z,
                50,
            );

            for object in &objects {
                gl::PushMatrix();
                gl::Translatef(
                    object.2.position.x - camera.position.x,
                    object.2.position.y - camera.position.y,
                    object.2.position.z - camera.position.z,
                );
                gl::MultMatrixf(object.2.rotation.matrix.to_cols_array().as_ptr());

                render_graph::draw_graph(
                    &object.0,
                    &mut texture_cache,
                    &mut tmt_cache,
                    false,
                    &object.1,
                    &ambient_color,
                );
                gl::PopMatrix();
            }
        }

        window.swap_buffers();

        glfw.poll_events();

        sky.tick(delta);

        let mut x_disp: f32 = 0.0;
        let mut z_disp: f32 = 0.0;

        if window.get_key(glfw::Key::Escape) == Action::Press {
            window.set_should_close(true);
        }

        if window.get_key(glfw::Key::W) == Action::Press {
            z_disp = 1.0;
        } else if window.get_key(glfw::Key::S) == Action::Press {
            z_disp = -1.0;
        }

        if window.get_key(glfw::Key::A) == Action::Press {
            x_disp = -1.0;
        } else if window.get_key(glfw::Key::D) == Action::Press {
            x_disp = 1.0;
        }

        if window.get_key(glfw::Key::LeftShift) == Action::Press {
            x_disp *= 10.0;
            z_disp *= 10.0;
        }

        camera.translate(x_disp * 10.0 * delta, z_disp * 10.0 * delta);

        let (x, y) = window.get_cursor_pos();
        let (dx, dy) = (ox - x, oy - y);
        ox = x;
        oy = y;

        camera.turn((dx as f32) * 2.0, (dy as f32) * 2.0, delta);

        window.set_title(&format!(
            "{}, {}, {}",
            &camera.position.x, &camera.position.y, &camera.position.z
        ));
    }

    return Ok(());
}
