mod cache;
mod caches;
mod camera;
mod geo_renderer;
mod gl;
mod sky;
//mod smacker_player;
mod terrain;

use glam::{Vec3, Vec4};
use glfw::{
    ffi::{glfwSetFramebufferSizeCallback, GLFWwindow},
    Action, Context,
};
use std::{cell::RefCell, path::Path, rc::Rc, time::Instant};

use lib76::fileparsers::{ter::TER, vtf::VTF};
use lib76::{
    fileparsers::{act::ACT, msn::MSN, sdf::SDF, vcf::VCF, vdf::VDF},
    virtual_fs::VirtualFS,
    zfs_archive::ZFSArchive,
};

use crate::{geo_renderer::RenderMode, sky::Sky};

fn load_vcf(vfs: &VirtualFS, vcf_filename: &str) -> anyhow::Result<(VCF, VDF, VTF)> {
    let vcf: VCF = vfs.load(vcf_filename)?;
    let vdf: VDF = vfs.load(&vcf.vcfc.vdf_filename)?;
    let vtf: VTF = vfs.load(&vcf.vcfc.vtf_filename)?;

    Ok((vcf, vdf, vtf))
}
fn main() -> anyhow::Result<()> {
    let mut camera = camera::Camera::new();

    println!("Loading data");
    let game_path = Path::new(r"C:\spel\Interstate 76");
    // let vfs = virtual_fs::VirtualFS::new(r"C:\spel\Interstate 76".to_string())?;
    let vfs = VirtualFS {
        disk_folders: vec![game_path.join("addon"), game_path.join("miss16")],
        zfs_archive: Some(ZFSArchive::new(&game_path.join("I76.ZFS"))?),
    };
    // let mut files = vfs.get_file_list();
    // files.sort();
    // let _ = std::fs::write("filelist.txt", files.join("\n"));

    let msn: MSN = vfs.load("T01.msn")?;
    let act: ACT = vfs.load(&msn.wrld.act_filename)?;
    let ter: TER = vfs.load(&msn.tdef.zone.ter_filename)?;

    let geo_cache = Rc::new(RefCell::new(caches::build_geo_cache(&vfs)));
    let mut cbk_cache = caches::build_cbk_cache(&vfs);
    let mut texture_cache = caches::build_texture_cache(&vfs, &mut cbk_cache, &act);
    let mut tmt_cache = caches::build_tmt_cache(&vfs);

    let objects: Vec<_> = msn
        .odef_objs
        .iter()
        .map(|o| {
            if o.class_id == 1 {
                let (vcf, vdf, vtf) = load_vcf(&vfs, &format!("{}.vcf", &o.label[..]))?;
                println!(
                    "Building render graph for {} {}",
                    &vdf.vdfc.name, &vcf.vcfc.variant_name
                );
                let vdf_graph =
                    lib76::geo_graph::from(vdf.vgeo.third_person_parts[0][0].iter(), |name| {
                        geo_cache
                            .borrow_mut()
                            .get(name)
                            .expect("Failed to load")
                            .clone()
                    })?;
                anyhow::Ok((vdf_graph, RenderMode::Vehicle(vtf), o))
            } else {
                let sdf: SDF = vfs.load(&format!("{}.sdf", &o.label))?;
                println!(
                    "Building render graph for {} ({})",
                    &sdf.sdfc.name, &o.label
                );
                let graph = lib76::geo_graph::from(
                    sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part),
                    |name| {
                        geo_cache
                            .borrow_mut()
                            .get(name)
                            .expect("Failed to load")
                            .clone()
                    },
                )?;
                anyhow::Ok((graph, RenderMode::SGeo, o))
            }
        })
        .filter_map(|f: anyhow::Result<_>| f.ok())
        .collect();

    camera.position = objects
        .iter()
        .find(|o| o.2.label == "vppirna1")
        .map(|o| o.2.position + Vec3::Y * 2.0)
        .unwrap_or(Vec3::ZERO);

    println!("Starting GLFW...");

    fn error_callback(err: glfw::Error, description: String) {
        eprintln!("GLFW error {:?}: {:?}", err, description);
    }

    let mut glfw = glfw::init(error_callback).unwrap();

    let (mut window, _events) = glfw
        .create_window(800, 600, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.make_current();
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.set_key_polling(true);
    window.set_raw_mouse_motion(true);

    gl::load_with(|s| window.get_proc_address(s));

    let light_pos = Vec4::new(-1.0, 1.0, -1.0, 0.0).to_array().as_ptr();
    let ambient_color = act.entries[247].to_vec3().extend(1.0).to_array();

    unsafe {
        unsafe fn resize(w: i32, h: i32) {
            gl::MatrixMode(gl::PROJECTION);
            let perspective =
                glam::Mat4::perspective_lh(60.0f32.to_radians(), w as f32 / h as f32, 0.1, 1000.0);
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

            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::ALPHA_TEST);
            gl::Enable(gl::LIGHTING);
            gl::Enable(gl::DEPTH_TEST);

            terrain::render_terrain(
                surface_texture,
                &msn.tdef.zmap,
                &ter,
                camera.position.x,
                camera.position.y,
                camera.position.z,
                100,
            );

            for object in &objects {
                gl::PushMatrix();
                gl::Translatef(
                    object.2.position.x - camera.position.x,
                    object.2.position.y - camera.position.y,
                    object.2.position.z - camera.position.z,
                );
                gl::MultMatrixf(object.2.rotation.matrix.to_cols_array().as_ptr());

                geo_renderer::draw_graph(
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

    Ok(())
}
