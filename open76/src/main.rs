mod cache;
mod camera;
mod frustum;
mod gl;
mod render_graph;
mod smacker_player;
mod terrain;
mod texture_loader;
mod virtual_fs;

use std::{path::Path, time::Instant};

use cache::FileCache;
use gl::types::GLuint;
use glam::Vec3;
use glfw::{
    ffi::{glfwSetFramebufferSizeCallback, GLFWwindow},
    Action, Context,
};

use lib76::{
    clut::LUT,
    fileparsers::{self, cbk::CBK, map::Map, msn::MSN, vcf::VCF, vdf::VDF, vqm::VQM, Geo},
};
use lib76::{
    fileparsers::{msn::ODEFObj, ter::TER, tmt::TMT, vtf::VTF},
    math,
};

use texture_loader::load_gl_texture;

use crate::{
    render_graph::{GeoNode, RenderMode},
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

fn build_geo_cache<'a>() -> FileCache<'a, Geo> {
    cache::FileCache::new(|name| {
        virtual_fs::load::<Geo>(&format!("E:/i76/extracted/{}.geo", name)).ok()
    })
}

fn build_cbk_cache<'a>() -> FileCache<'a, CBK> {
    cache::FileCache::new(|name| {
        virtual_fs::load::<CBK>(&format!("E:/i76/extracted/{}", name)).ok()
    })
}

fn build_texture_cache<'a>(cbk_cache: &'a mut FileCache<CBK>) -> FileCache<'a, GLuint> {
    cache::FileCache::new(move |name| {
        let fixed_name = if name.to_ascii_lowercase().ends_with(".map") {
            &name[0..name.len() - 4]
        } else {
            name
        };
        let vqm_path = format!("E:/i76/extracted/{}.vqm", fixed_name);
        let map_path = format!("E:/i76/extracted/{}.map", fixed_name);

        let tex = match (Path::new(&vqm_path).exists(), Path::new(&map_path).exists()) {
            (true, _) => {
                let vqm: VQM =
                    virtual_fs::load(&&vqm_path).expect(&format!("Failed to load {}", vqm_path));
                let cbk = cbk_cache.get(&vqm.cbk_filename)?;
                Some(load_gl_texture(
                    vqm.width,
                    vqm.height,
                    &vqm.to_rgba_pixels(cbk, &LUT),
                ))
            }
            (_, true) => {
                let map: Map =
                    virtual_fs::load(&&&map_path).expect(&format!("Failed to load {}", map_path));
                Some(load_gl_texture(
                    map.width,
                    map.height,
                    &map.to_rgba_pixels(&LUT),
                ))
            }
            (false, false) => None,
        }?;

        Some(tex)
    })
}

fn build_tmt_cache<'a>() -> FileCache<'a, TMT> {
    cache::FileCache::new(|name| {
        virtual_fs::load::<TMT>(&format!("E:/i76/extracted/{}", name)).ok()
    })
}

fn main() -> Result<(), std::io::Error> {
    println!("Initializing");

    let mut camera = camera::Camera::new();

    let mut geo_cache = build_geo_cache();
    let mut cbk_cache = build_cbk_cache();
    let mut texture_cache = build_texture_cache(&mut cbk_cache);
    let mut tmt_cache = build_tmt_cache();

    println!("Loading data");
    let msn: MSN = virtual_fs::load("E:/i76/MISSIONS/T01.msn")?;

    let ter: TER = virtual_fs::load(&format!("E:/i76/MISSIONS/{}", &msn.tdef.zone.ter_filename))?;

    let objects = msn
        .odef_objs
        .iter()
        .map(|o| {
            if o.class_id == 1 {
                let (vcf, vdf, vtf) = load_vcf(&format!("{}.vcf", &o.label[..]))?;
                println!(
                    "Building render graph for {} {}",
                    vdf.vdfc.name, vcf.vcfc.variant_name
                );
                let vdf_graph =
                    render_graph::from(vdf.vgeo.third_person_parts[0][0].iter(), &mut geo_cache)?;
                Ok((vdf_graph, RenderMode::Vehicle(vtf), o))
            } else {
                let sdf: fileparsers::SDF =
                    virtual_fs::load(&format!("E:/i76/extracted/{}.sdf", o.label))?;
                println!("Building render graph for {}", sdf.sdfc.name);
                let graph = render_graph::from(
                    sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part),
                    &mut &mut geo_cache,
                )?;
                Ok((graph, RenderMode::SGEO, o))
            }
        })
        .collect::<Result<Vec<(Vec<GeoNode>, render_graph::RenderMode, &ODEFObj)>, std::io::Error>>(
        )?;

    camera.position = objects
        .iter()
        .find(|o| o.2.label == "vppirna1")
        .map(|o| Vec3::new(o.2.position.0, o.2.position.1 + 10.0, o.2.position.2))
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

    let light_pos = [0.0, 0.0, -50.0, 0.0].as_ptr();
    unsafe {
        unsafe fn resize(w: i32, h: i32) {
            gl::MatrixMode(gl::PROJECTION);
            gl::LoadIdentity();

            let mut matrix = [0.0; 16];
            frustum::glh_perspectivef2(&mut matrix, 60.0, w as f32 / h as f32, 0.1, 1000.0);
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

        gl::Enable(gl::ALPHA_TEST);
        gl::AlphaFunc(gl::GEQUAL, 1.0);

        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::TEXTURE_2D);
        

        gl::ClearColor(0.0, 0.0, 0.0, 1.0);

        gl::Enable(gl::LIGHTING);
        gl::Enable(gl::LIGHT0);
        gl::Lightfv(gl::LIGHT0, gl::DIFFUSE, [1.0, 1.0, 1.0, 1.0].as_ptr());
        gl::Lightfv(gl::LIGHT0, gl::AMBIENT, [0.5, 0.5, 0.5, 1.0].as_ptr());
    }

    println!("...done");

    fn rotate_by_xyz(x_vector: &math::Vec3, y_vector: &math::Vec3, z_vector: &math::Vec3) {
        let mat = [
            x_vector.0,
            y_vector.0,
            z_vector.0,
            0.0,
            x_vector.1,
            y_vector.1,
            z_vector.1,
            0.0,
            -x_vector.2,
            -y_vector.2,
            -z_vector.2,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        ];

        unsafe {
            gl::MultMatrixf(mat.as_ptr());
        }
    }

    let (mut ox, mut oy) = (0.0, 0.0);
    let mut last_time = Instant::now();
    let mut sky_drift = 0.0;
    while !window.should_close() {
        let now = Instant::now();
        let delta = now.duration_since(last_time);
        last_time = now;
        let secs = delta.as_secs_f64();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::LoadIdentity();
            gl::Rotatef(camera.pitch, 1.0, 0.0, 0.0);
            gl::Rotatef(camera.yaw, 0.0, 1.0, 0.0);

            gl::Disable(gl::LIGHTING);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::ALPHA_TEST);
            gl::Enable(gl::TEXTURE_2D);

            gl::PushMatrix();

            texture_cache
                .get(&msn.wrld.sky_texture_filename)
                .map(|tex| gl::BindTexture(gl::TEXTURE_2D, **tex));

            gl::Color3f(1.0, 1.0, 1.0);
            gl::Begin(gl::QUADS);
            gl::TexCoord2d(sky_drift, 0.0);
            gl::Vertex3f(-100.0, 1.0, -100.0);
            gl::TexCoord2d(sky_drift + 100.0, 0.0);
            gl::Vertex3f(100.0, 1.0, -100.0);
            gl::TexCoord2d(sky_drift + 100.0, 100.0);
            gl::Vertex3f(100.0, 1.0, 100.0);
            gl::TexCoord2d(sky_drift, 100.0);
            gl::Vertex3f(-100.0, 1.0, 100.0);
            gl::End();

            gl::PopMatrix();

            gl::Translatef(-camera.position.x, -camera.position.y, camera.position.z);

            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::ALPHA_TEST);
            gl::Enable(gl::LIGHTING);
            gl::Enable(gl::DEPTH_TEST);            
            
            gl::Lightfv(gl::LIGHT0, gl::POSITION, light_pos);

            
            gl::Enable(gl::POLYGON_OFFSET_FILL);
            gl::PolygonOffset(5.0, 1.0);

            texture_cache
                .get(&msn.wrld.surface_texture_filename)
                .map(|tex| gl::BindTexture(gl::TEXTURE_2D, **tex));
                
            terrain::render_terrain(&msn.tdef.zmap,&ter, camera.position.x, camera.position.z, 50);

            gl::Disable(gl::POLYGON_OFFSET_FILL);

            for object in &objects {
                gl::PushMatrix();
                gl::Translatef(
                    object.2.position.0,
                    object.2.position.1,
                    -object.2.position.2,
                );
                rotate_by_xyz(
                    &object.2.rotation.right,
                    &object.2.rotation.up,
                    &object.2.rotation.forward,
                );

                render_graph::draw_graph(
                    &object.0,
                    &mut texture_cache,
                    &mut tmt_cache,
                    false,
                    &object.1,
                );
                gl::PopMatrix();
            }

            
        }

        window.swap_buffers();

        glfw.poll_events();

        sky_drift = (sky_drift + 0.1 * secs) % 100.0;

        let mut x_disp: f64 = 0.0;
        let mut z_disp: f64 = 0.0;

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

        camera.translate((z_disp * 10.0 * secs) as f32, (x_disp * 10.0 * secs) as f32);

        let (x, y) = window.get_cursor_pos();
        let (dx, dy) = (ox - x, oy - y);
        ox = x;
        oy = y;

        camera.turn((dx * 2.0) as f32, (dy * 2.0) as f32, secs as f32);

        window.set_title(&format!(
            "{}, {}, {}",
            &camera.position.x, &camera.position.y, &camera.position.z
        ));
    }

    return Ok(());
}
