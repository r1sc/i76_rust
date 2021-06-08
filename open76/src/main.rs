mod cache;
mod camera;
mod frustum;
mod gl;
mod render_graph;
mod smacker_player;
mod texture_loader;
mod virtual_fs;

use std::{path::Path, time::Instant};

use gl::types::GLuint;
use glfw::{
    ffi::{glfwSetFramebufferSizeCallback, GLFWwindow},
    Action, Context,
};

use lib76::{clut::LUT, fileparsers::{self, Geo, cbk::CBK, map::Map, vcf::VCF, vdf::VDF, vqm::VQM}};
use lib76::{
    fileparsers::{tmt::TMT, vtf::VTF},
    math::*,
};
use render_graph::GeoNode;
use texture_loader::load_gl_texture;

enum RenderMode<'a> {
    SGEO,
    Vehicle(&'a VTF),
}

fn render_geo(
    geo: &fileparsers::Geo,
    texture_cache: &mut cache::FileCache<GLuint>,
    tmt_cache: &mut cache::FileCache<TMT>,
    use_face_normals: bool,
    render_mode: &RenderMode,
) {
    for face in &geo.faces {
        unsafe {
            if face.texture_name != "" && face.texture_name != "V1 BO DY" {
                let texture_name = match render_mode {
                    RenderMode::SGEO => &face.texture_name,
                    RenderMode::Vehicle(vtf) => {
                        if face.texture_name.starts_with("V1") {
                            let vtf_part_no =
                                fileparsers::vtf::car_texture_name_to_vtf_loc(&face.texture_name);

                            let filename = &vtf.vtfc.parts[vtf_part_no as usize][..];
                            if filename.ends_with(".TMT") || filename.ends_with(".tmt") {
                                let tmt = tmt_cache.get(filename);
                                match tmt {
                                    Some(tmt) => &tmt.filenames[0][0][..],
                                    None => {
                                        panic!("Cannot find TMT file {}", filename)
                                    }
                                }
                            } else {
                                filename
                            }
                        } else {
                            &face.texture_name[..]
                        }
                    }
                };

                texture_cache
                    .get(texture_name)
                    .map(|tex| gl::BindTexture(gl::TEXTURE_2D, **tex));
            } else {
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            if (face.flags.1 & 4) == 4 {
                gl::Enable(gl::ALPHA_TEST);
            } else {
                gl::Disable(gl::ALPHA_TEST);
            }

            gl::Begin(gl::TRIANGLE_FAN);
            for v in &face.vertex_refs {
                let Vec3(x, y, z) = geo.vertices[v.vertex_index as usize];
                let Vec3(nx, ny, nz) = if use_face_normals {
                    Vec3(face.normal.0, face.normal.1, face.normal.2)
                } else {
                    geo.normals[v.normal_index as usize]
                };
                let (u, v) = v.uv;

                gl::TexCoord2f(u, v);
                gl::Normal3f(nx, ny, nz);
                gl::Vertex3f(-x, y, z);
            }
            gl::End();
        }
    }
}

fn render_graph(
    root_children: &Vec<GeoNode>,
    texture_cache: &mut cache::FileCache<GLuint>,
    tmt_cache: &mut cache::FileCache<TMT>,
    use_face_normals: bool,
    render_mode: &RenderMode,
) {
    for part in root_children {
        unsafe {
            gl::PushMatrix();
            gl::Translatef(
                -part.local_position.0,
                part.local_position.1,
                part.local_position.2,
            );
            render_geo(
                &part.geo,
                texture_cache,
                tmt_cache,
                use_face_normals,
                render_mode,
            );

            render_graph(
                &part.children,
                texture_cache,
                tmt_cache,
                use_face_normals,
                render_mode,
            );

            gl::PopMatrix();
        }
    }
}

fn load_vcf(vcf_filename: &str) -> Result<(VCF, VDF, VTF), std::io::Error> {
    let vcf: fileparsers::vcf::VCF = virtual_fs::load(&format!("E:/i76/extracted/{}", vcf_filename))?;
    let vdf: fileparsers::vdf::VDF =
        virtual_fs::load(&format!("E:/i76/extracted/{}", vcf.vcfc.vdf_filename))?;
    let vtf: fileparsers::vtf::VTF =
        virtual_fs::load(&format!("E:/i76/extracted/{}", vcf.vcfc.vtf_filename))?;

    Ok((vcf, vdf, vtf))
}

fn main() -> Result<(), std::io::Error> {
    let (vcf, vdf, vtf) = load_vcf("vppirna1.vcf")?;

    let mut geo_cache = cache::FileCache::new(|name| {
        virtual_fs::load::<Geo>(&format!("E:/i76/extracted/{}.geo", name)).ok()
    });

    let vdf_graph = render_graph::from(vdf.vgeo.third_person_parts[0][0].iter(), &mut geo_cache)?;

    // let sdf: fileparsers::SDF = virtual_fs::load("E:/i76/extracted/BDWGNWL1.sdf")?;
    // let graph = render_graph::from(sdf.sgeo.lod_levels[0].lod_parts.iter().map(|a| &a.geo_part))?;
    let mut camera = camera::Camera::new();

    let mut cbk_cache = cache::FileCache::new(|name| {
        virtual_fs::load::<CBK>(&format!("E:/i76/extracted/{}", name)).ok()
    });

    let mut texture_cache = cache::FileCache::new(|name| {
        let vqm_path = format!("E:/i76/extracted/{}.vqm", name);
        let map_path = format!("E:/i76/extracted/{}.map", name);

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
    });

    let mut tmt_cache = cache::FileCache::new(|name| {
        virtual_fs::load::<TMT>(&format!("E:/i76/extracted/{}", name)).ok()
    });

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw
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

    let (mut ox, mut oy) = (0.0, 0.0);
    let mut last_time = Instant::now();

    let mut an = 0.0;
    while !window.should_close() {
        let now = Instant::now();
        let delta = now.duration_since(last_time);
        last_time = now;
        let secs = delta.as_secs_f64();

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::LoadIdentity();
            camera.do_gl_transform();

            gl::Lightfv(gl::LIGHT0, gl::POSITION, light_pos);

            gl::PushMatrix();
            gl::Translated(0.0, 0.0, -5.0);
            //gl::Rotatef(an, 0.0, 1.0, 0.0);
            render_graph(
                &vdf_graph,
                &mut texture_cache,
                &mut tmt_cache,
                false,
                &RenderMode::Vehicle(&vtf),
            );
            gl::PopMatrix();

            an+=1.0;
            // gl::PushMatrix();
            // gl::Translated(10.0, 0.0, -10.0);
            // render_graph(
            //     &graph,
            //     &mut texture_cache,
            //     &mut tmt_cache,
            //     false,
            //     &RenderMode::SGEO,
            // );
            // gl::PopMatrix();
        }

        window.swap_buffers();

        glfw.poll_events();

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

        camera.translate((z_disp * 10.0 * secs) as f32, (x_disp * 10.0 * secs) as f32);

        let (x, y) = window.get_cursor_pos();
        let (dx, dy) = (ox - x, oy - y);
        ox = x;
        oy = y;

        camera.turn((dx * 2.0) as f32, (dy * 2.0) as f32, secs as f32);
    }

    return Ok(());
}
