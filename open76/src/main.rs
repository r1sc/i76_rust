mod frustum;
mod gl;
mod render_graph;
mod smacker_player;
mod texture_loader;

extern crate glfw;

use std::{collections::HashMap, path::Path};

use gl::types::GLuint;
use glfw::{
    ffi::{glfwSetFramebufferSizeCallback, GLFWwindow},
    Context,
};

use lib76::{clut::LUT, fileparsers::{self, cbk::CBK, map::Map, vqm::VQM}};
use lib76::math::*;
use lib76::virtual_fs;
use render_graph::GeoNode;
use texture_loader::load_gl_texture;

struct FileCache<'a, T> {
    content: HashMap<String, T>,
    loader: Box<dyn FnMut(&str) -> Option<T> + 'a>,
}

impl<'a, T> FileCache<'a, T> {
    pub fn new(loader: impl FnMut(&str) -> Option<T> + 'a) -> Self {
        FileCache {
            content: HashMap::new(),
            loader: Box::new(loader)
        }
    }

    pub fn get(&mut self, name: &str) -> Option<&T> {
        if self.content.contains_key(name) {
            self.content.get(name)
        }
        else {
            let a = (self.loader)(name)?;
            self.content.insert(String::from(name), a);
            self.content.get(name)
        }
    }
}

fn render_geo(geo: &fileparsers::Geo, texture_cache: &mut FileCache<GLuint>) {
    for face in &geo.faces {
        let Vec4(nx, ny, nz, _) = face.normal;

        unsafe {            
            if face.texture_name != "" {
                texture_cache.get(&face.texture_name).map(|tex| 
                    gl::BindTexture(gl::TEXTURE_2D, *tex)
                );
            }
            else {
                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            gl::Begin(gl::TRIANGLE_FAN);
            for v in &face.vertex_refs {
                let Vec3(x, y, z) = geo.vertices[v.vertex_index as usize];
                let (u, v) = v.uv;
                gl::TexCoord2f(u, v);
                gl::Normal3f(nx, ny, nz);
                gl::Vertex3f(-x, y, z);
            }
            gl::End();
        }
    }
}

fn render_graph(root_children: &Vec<GeoNode>, texture_cache: &mut FileCache<GLuint>) {
    for part in root_children {
        
        unsafe {
            gl::PushMatrix();
            gl::Translatef(-part.local_position.0, part.local_position.1, part.local_position.2);
            render_geo(&part.geo, texture_cache);

            render_graph(&part.children, texture_cache);

            gl::PopMatrix();
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    // let geo = virtual_fs::load("E:\\i76\\extracted\\aa2_rmp1.geo")?;
    let sdf: fileparsers::SDF = virtual_fs::load("E:/i76/extracted/bddonut1.sdf")?;
    let graph = render_graph::from(&sdf.sgeo.lod_levels[0].lod_parts)?;
    
    let mut cbk_cache = FileCache::new(|name| {
        virtual_fs::load::<CBK>(&format!("E:/i76/extracted/{}", name)).ok()
    });

    let mut texture_cache = FileCache::new(|name| {
        let vqm_path = format!("E:/i76/extracted/{}.vqm", name);
        let map_path = format!("E:/i76/extracted/{}.map", name);

        let tex = match (Path::new(&vqm_path).exists(), Path::new(&map_path).exists()) {
            (true, _) => {
                let vqm: VQM = virtual_fs::load(&&vqm_path).expect(&format!("Failed to load {}", vqm_path));
                let cbk = cbk_cache.get(&vqm.cbk_filename)?;
                Some(load_gl_texture(vqm.width, vqm.height, &vqm.to_rgba_pixels(cbk, &LUT)))
            },
            (_, true) =>  {
                let map: Map = virtual_fs::load(&&&map_path).expect(&format!("Failed to load {}", map_path));
                Some(load_gl_texture(map.width, map.height, &map.to_rgba_pixels(&LUT)))
            },
            (false, false) => None,
        }?;

        Some(tex)
    });



    // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    // let file = BufReader::new(File::open("E:/i76/music/2.mp3").unwrap());
    // let source = Decoder::new(file).unwrap();
    // stream_handle
    //     .play_raw(source.convert_samples())
    //     .expect("Couldn't play sound");

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, _events) = glfw
        .create_window(640, 480, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    // window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));

    // let mut player = SmackerPlayer::new("E:\\i76\\cutscene\\ANG05F01.smk")
    //     .expect("Failed to open smacker file");

    

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
            render_graph(&graph, &mut texture_cache);
            // player.tick(0.0).expect("Failed to get video frame");

            // gl::BindTexture(gl::TEXTURE_2D, player.texture);
            // gl::Begin(gl::QUADS);
            // gl::TexCoord2f(0.0, 0.0);
            // gl::Vertex3f(-1.0, 1.0, 0.0);

            // gl::TexCoord2f(1.0, 0.0);
            // gl::Vertex3f(1.0, 1.0, 0.0);

            // gl::TexCoord2f(1.0, 1.0);
            // gl::Vertex3f(1.0, -1.0, 0.0);

            // gl::TexCoord2f(0.0, 1.0);
            // gl::Vertex3f(-1.0, -1.0, 0.0);
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
