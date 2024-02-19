use glow::HasContext;

pub struct ShaderProgram {
    program: glow::Program,
}

impl ShaderProgram {
    pub fn load(gl: &glow::Context, vertex_src: &str, frag_src: &str) -> Self {
        let build_shader = |src: &str, kind: u32| -> glow::Shader {
            unsafe {
                let shader = gl.create_shader(kind).unwrap();
                gl.shader_source(shader, src);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!(
                        "{} shader compile error: {}",
                        if kind == glow::VERTEX_SHADER {
                            "Vertex"
                        } else {
                            "Fragment"
                        },
                        gl.get_shader_info_log(shader)
                    );
                }
                shader
            }
        };

        unsafe {
            let program = gl.create_program().unwrap();
            let vertex_shader = build_shader(vertex_src, glow::VERTEX_SHADER);
            let fragment_shader = build_shader(frag_src, glow::FRAGMENT_SHADER);
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("Program link error: {}", gl.get_program_info_log(program));
            }
            Self { program }
        }
    }

    pub fn use_program(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.program));
        }
    }

    pub fn get_uniform_location(
        &self,
        gl: &glow::Context,
        name: &str,
    ) -> Option<glow::UniformLocation> {
        unsafe { gl.get_uniform_location(self.program, name) }
    }
}
