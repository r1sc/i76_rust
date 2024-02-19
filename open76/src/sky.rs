use crate::gl::{self, types::GLuint};

pub struct Sky {
    drift: f32,
}

impl Sky {
    pub fn new() -> Self {
        Self { drift: 0.0 }
    }

    pub fn tick(&mut self, delta: f32) {
        self.drift = (self.drift + 0.1 * delta) % 100.0;
    }

    pub fn draw(&self, texture: GLuint) {
        unsafe {
            gl::Disable(gl::LIGHTING);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::ALPHA_TEST);
            gl::Enable(gl::TEXTURE_2D);

            gl::PushMatrix();

            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::Begin(gl::QUADS);
            gl::TexCoord2f(self.drift, 0.0);
            gl::Vertex3f(-100.0, 1.0, -100.0);
            gl::TexCoord2f(self.drift + 100.0, 0.0);
            gl::Vertex3f(100.0, 1.0, -100.0);
            gl::TexCoord2f(self.drift + 100.0, 100.0);
            gl::Vertex3f(100.0, 1.0, 100.0);
            gl::TexCoord2f(self.drift, 100.0);
            gl::Vertex3f(-100.0, 1.0, 100.0);
            gl::End();

            gl::PopMatrix();
        }
    }
}
