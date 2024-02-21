use egui::*;

pub struct EguiInputState {
    pub pointer_pos: Pos2,
    pub input: RawInput,
    pub modifiers: Modifiers,
}

impl EguiInputState {
    pub fn new(input: RawInput) -> Self {
        EguiInputState {
            pointer_pos: Pos2::new(0f32, 0f32),
            input,
            modifiers: Modifiers::default(),
        }
    }

    pub fn handle_event(
        &mut self,
        event: glfw::WindowEvent
    ) {
        use glfw::WindowEvent::*;

        match event {
            FramebufferSize(width, height) => {
                self.input.screen_rect = Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width as f32, height as f32),
                ));
            }

            MouseButton(mouse_btn, glfw::Action::Press, _) => {
                self.input.events.push(egui::Event::PointerButton {
                    pos: self.pointer_pos,
                    button: match mouse_btn {
                        glfw::MouseButtonLeft => egui::PointerButton::Primary,
                        glfw::MouseButtonRight => egui::PointerButton::Secondary,
                        glfw::MouseButtonMiddle => egui::PointerButton::Middle,
                        _ => unreachable!(),
                    },
                    pressed: true,
                    modifiers: self.modifiers,
                })
            }

            MouseButton(mouse_btn, glfw::Action::Release, _) => {
                self.input.events.push(egui::Event::PointerButton {
                    pos: self.pointer_pos,
                    button: match mouse_btn {
                        glfw::MouseButtonLeft => egui::PointerButton::Primary,
                        glfw::MouseButtonRight => egui::PointerButton::Secondary,
                        glfw::MouseButtonMiddle => egui::PointerButton::Middle,
                        _ => unreachable!(),
                    },
                    pressed: false,
                    modifiers: self.modifiers,
                })
            }

            CursorPos(x, y) => {
                self.pointer_pos = pos2(x as f32, y as f32);
                self.input
                    .events
                    .push(egui::Event::PointerMoved(self.pointer_pos))
            }

            Key(keycode, _scancode, glfw::Action::Release, keymod) => {
                use glfw::Modifiers as Mod;
                if let Some(key) = translate_virtual_key_code(keycode) {
                    self.modifiers = Modifiers {
                        alt: (keymod & Mod::Alt == Mod::Alt),
                        ctrl: (keymod & Mod::Control == Mod::Control),
                        shift: (keymod & Mod::Shift == Mod::Shift),

                        // TODO: GLFW doesn't seem to support the mac command key
                        //       mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,
                        command: (keymod & Mod::Control == Mod::Control),

                        ..Default::default()
                    };

                    self.input.events.push(Event::Key {
                        key,
                        pressed: false,
                        modifiers: self.modifiers,
                        repeat: false,
                        physical_key: None
                    });
                }
            }

            Key(keycode, _scancode, action, keymod) => {
                use glfw::Modifiers as Mod;
                if let Some(key) = translate_virtual_key_code(keycode) {
                    self.modifiers = Modifiers {
                        alt: (keymod & Mod::Alt == Mod::Alt),
                        ctrl: (keymod & Mod::Control == Mod::Control),
                        shift: (keymod & Mod::Shift == Mod::Shift),

                        // TODO: GLFW doesn't seem to support the mac command key
                        //       mac_cmd: keymod & Mod::LGUIMOD == Mod::LGUIMOD,
                        command: (keymod & Mod::Control == Mod::Control),

                        ..Default::default()
                    };

                    if self.modifiers.command && key == egui::Key::X {
                        self.input.events.push(egui::Event::Cut);
                    } else if self.modifiers.command && key == egui::Key::C {
                        self.input.events.push(egui::Event::Copy);
                    } else if self.modifiers.command && key == egui::Key::V {
                        // if let Some(clipboard_ctx) = state.clipboard.as_mut() {
                        //     state.input.events.push(egui::Event::Text(
                        //         clipboard_ctx
                        //             .get_contents()
                        //             .unwrap_or_else(|_| "".to_string()),
                        //     ));
                        // }
                    } else {
                        self.input.events.push(Event::Key {
                            key,
                            pressed: action == glfw::Action::Press,
                            modifiers: self.modifiers,
                            repeat: action == glfw::Action::Repeat,
                            physical_key: None
                        });
                    }
                }
            }

            Char(c) => {
                self.input.events.push(Event::Text(c.to_string()));
            }

            Scroll(x, y) => {
                self.input
                    .events
                    .push(Event::Scroll(vec2(x as f32, y as f32)));
            }

            _ => {}
        }
    }
}

pub fn translate_virtual_key_code(key: glfw::Key) -> Option<egui::Key> {
    use glfw::Key::*;

    Some(match key {
        Left => Key::ArrowLeft,
        Up => Key::ArrowUp,
        Right => Key::ArrowRight,
        Down => Key::ArrowDown,

        Escape => Key::Escape,
        Tab => Key::Tab,
        Backspace => Key::Backspace,
        Space => Key::Space,

        Enter => Key::Enter,

        Insert => Key::Insert,
        Home => Key::Home,
        Delete => Key::Delete,
        End => Key::End,
        PageDown => Key::PageDown,
        PageUp => Key::PageUp,

        A => Key::A,
        B => Key::B,
        C => Key::C,
        D => Key::D,
        E => Key::E,
        F => Key::F,
        G => Key::G,
        H => Key::H,
        I => Key::I,
        J => Key::J,
        K => Key::K,
        L => Key::L,
        M => Key::M,
        N => Key::N,
        O => Key::O,
        P => Key::P,
        Q => Key::Q,
        R => Key::R,
        S => Key::S,
        T => Key::T,
        U => Key::U,
        V => Key::V,
        W => Key::W,
        X => Key::X,
        Y => Key::Y,
        Z => Key::Z,

        _ => {
            return None;
        }
    })
}

// pub fn translate_cursor(cursor_icon: egui::CursorIcon) -> glfw::StandardCursor {
//     match cursor_icon {
//         CursorIcon::Default => glfw::StandardCursor::Arrow,

//         CursorIcon::PointingHand => glfw::StandardCursor::Hand,

//         CursorIcon::ResizeHorizontal => glfw::StandardCursor::HResize,
//         CursorIcon::ResizeVertical => glfw::StandardCursor::VResize,
//         // TODO: GLFW doesnt have these specific resize cursors, so we'll just use the HResize and VResize ones instead
//         CursorIcon::ResizeNeSw => glfw::StandardCursor::HResize,
//         CursorIcon::ResizeNwSe => glfw::StandardCursor::VResize,

//         CursorIcon::Text => glfw::StandardCursor::IBeam,
//         CursorIcon::Crosshair => glfw::StandardCursor::Crosshair,

//         CursorIcon::Grab | CursorIcon::Grabbing => glfw::StandardCursor::Hand,

//         // TODO: Same for these
//         CursorIcon::NotAllowed | CursorIcon::NoDrop => glfw::StandardCursor::Arrow,
//         CursorIcon::Wait => glfw::StandardCursor::Arrow,
//         _ => glfw::StandardCursor::Arrow,
//     }
// }

// pub fn init_clipboard() -> Option<ClipboardContext> {
//     match ClipboardContext::new() {
//         Ok(clipboard) => Some(clipboard),
//         Err(err) => {
//             eprintln!("Failed to initialize clipboard: {}", err);
//             None
//         }
//     }
// }

// pub fn copy_to_clipboard(egui_state: &mut EguiInputState, copy_text: String) {
//     if let Some(clipboard) = egui_state.clipboard.as_mut() {
//         let result = clipboard.set_contents(copy_text);
//         if result.is_err() {
//             dbg!("Unable to set clipboard content.");
//         }
//     }
// }
