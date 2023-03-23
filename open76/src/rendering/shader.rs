use glow::HasContext;

pub struct ShaderProgram {
    program: glow::Program,
}

impl ShaderProgram {
    pub fn new(gl: &glow::Context, src: &str) -> Self {
        let build_shader = move |kind: u32| -> glow::Shader {
            let define_str = match kind {
                glow::VERTEX_SHADER => "#define VS",
                glow::FRAGMENT_SHADER => "#define FS",
                _ => panic!("Expected vertex/fragment"),
            };
            unsafe {
                let shader = gl.create_shader(kind).unwrap();
                gl.shader_source(shader, &format!("{}\n{}", define_str, src));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("Shader compile error: {}", gl.get_shader_info_log(shader));
                }
                shader
            }
        };

        unsafe {
            let program = gl.create_program().unwrap();
            let vertex_shader = build_shader(glow::VERTEX_SHADER);
            let fragment_shader = build_shader(glow::FRAGMENT_SHADER);
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
}
