use std::sync::Arc;

use render76::glow::{self, HasContext};

mod egui_glfw_input;

pub struct Gui {
    gl: Arc<glow::Context>,
    painter: egui_glow::Painter,
    egui_ctx: egui::Context,
    egui_input: egui_glfw_input::EguiInputState,
    full_output: egui::FullOutput,
}

impl Gui {
    pub fn new(gl: Arc<glow::Context>, window_width: u32, window_height: u32) -> Self {
        let painter =
            egui_glow::Painter::new(gl.clone(), "", Some(egui_glow::ShaderVersion::Es300))
                .expect("Failed to create egui_glow::Painter");

        let egui_ctx = egui::Context::default();
        egui_ctx.set_pixels_per_point(1.0);
        egui_ctx.set_visuals(egui::Visuals::light());

        let egui_input = egui_glfw_input::EguiInputState::new(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(window_width as f32, window_height as f32),
            )),
            ..egui::RawInput::default()
        });

        Self {
            gl,
            painter,
            egui_ctx,
            egui_input,
            full_output: Default::default(),
        }
    }

    pub fn handle_event(&mut self, event: glfw::WindowEvent) {
        self.egui_input.handle_event(event.clone());
    }

    pub fn run(&mut self, run_ui: impl FnOnce(&egui::Context)) {
        self.full_output = self.egui_ctx.run(self.egui_input.input.take(), run_ui);
    }

    pub fn render(&mut self, window_width: u32, window_height: u32) {
        let egui::FullOutput {
            pixels_per_point,
            platform_output: _,
            shapes,
            textures_delta,
            viewport_output: _,
        } = self.full_output.clone();

        // EGUI rendering
        for (id, image_delta) in textures_delta.set {
            self.painter.set_texture(id, &image_delta);
        }

        let clipped_primitives = self.egui_ctx.tessellate(shapes, pixels_per_point);

        self.painter.paint_primitives(
            [window_width, window_height],
            pixels_per_point,
            &clipped_primitives,
        );

        for id in textures_delta.free {
            self.painter.free_texture(id);
        }

        unsafe {
            self.gl.finish();
        }
    }
}
