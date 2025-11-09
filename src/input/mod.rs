use std::collections::HashSet;

use glam::{Vec2, vec2};
use winit::event::{DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct Input {
    pressed_keys: HashSet<KeyCode>,
    mouse_delta: Vec2,
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            mouse_delta: Vec2::ZERO,
        }
    }

    pub fn submit_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => self.handle_key_event(event),
            _ => {}
        }
    }

    pub fn submit_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.mouse_delta += vec2(delta.0 as f32, delta.1 as f32);
            }
            _ => {}
        }
    }

    pub fn is_key_pressed(&self, keycode: KeyCode) -> bool {
        self.pressed_keys.contains(&keycode)
    }

    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    pub fn reset_mouse_delta(&mut self) {
        self.mouse_delta = Vec2::ZERO;
    }

    fn handle_key_event(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(keycode) = event.physical_key else {
            return;
        };

        match event.state {
            ElementState::Pressed => {
                self.pressed_keys.insert(keycode);
            }
            ElementState::Released => {
                self.pressed_keys.remove(&keycode);
            }
        }
    }
}
