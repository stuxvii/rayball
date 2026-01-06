use raylib::prelude::*;
use crate::cfg::style;

const MAX_TRAIL_LENGTH: usize = 3;
const TRAIL_DECAY: f32 = 0.025;
const CURSOR_SIZE: f32 = 5.0;
const IDLE_THRESHOLD: f32 = 10.0;
const FADE_SPEED: f32 = 2.0;

pub struct CursorTrail {
    positions: [Vector2; MAX_TRAIL_LENGTH],
    last_mouse_pos: Vector2,
    timer: f32,
    idle_timer: f32,
}

impl Default for CursorTrail {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorTrail {
    pub fn new() -> Self {
        Self {
            positions: [Vector2::new(0.0, 0.0); MAX_TRAIL_LENGTH],
            last_mouse_pos: Vector2::new(0.0, 0.0),
            timer: 0.0,
            idle_timer: 0.0,
        }
    }

    pub fn draw(&mut self, dt: f32, d: &mut RaylibDrawHandle) {
        let mouse_position = d.get_mouse_position();

        if mouse_position == self.last_mouse_pos {
            self.idle_timer += dt;
        } else {
            self.idle_timer = 0.0;
            self.last_mouse_pos = mouse_position;
        }

        let alpha_factor = (1.0 - (self.idle_timer - IDLE_THRESHOLD) * FADE_SPEED).clamp(0.0, 1.0);

        if alpha_factor <= 0.0 {
            return;
        }

        self.timer += dt;
        if self.timer > TRAIL_DECAY {
            for i in (1..MAX_TRAIL_LENGTH).rev() {
                self.positions[i] = self.positions[i - 1];
            }
            self.positions[0] = mouse_position;
            self.timer = 0.0;
        }

        for (i, pos) in self.positions.iter().enumerate() {
            if pos.x != 0.0 || pos.y != 0.0 {
                let ratio = (MAX_TRAIL_LENGTH - i) as f32 / MAX_TRAIL_LENGTH as f32;
                let mut trail_color = style::PRIMARY_COLOR;

                let target_alpha = (ratio * 127.0 * alpha_factor) as u8;
                trail_color.a = target_alpha;
                
                let trail_radius = CURSOR_SIZE * ratio;
                d.draw_circle_v(pos, trail_radius, trail_color);
            }
        }

        
        let mut cursor_color = style::PRIMARY_COLOR;
        cursor_color.a = (255.0 * alpha_factor) as u8;
        d.draw_circle_v(mouse_position, CURSOR_SIZE, cursor_color);
    }
}