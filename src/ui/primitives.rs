use raylib::math::rrect;
use raylib::prelude::RaylibDraw;
use raylib::prelude::RaylibDrawHandle;
use raylib::prelude::*;

use crate::FLAGS_SPRITESHEET;
use crate::cfg::layout;
use crate::cfg::params::SETTINGS;
use crate::cfg::style;

#[derive(Debug)]
pub struct Button {
    pub txt: String,
}

impl Button {
    pub fn new(txt: String) -> Button {
        Button {
            txt: txt.to_string(),
        }
    }

    pub fn draw(&self, rect: Rectangle, d: &mut RaylibDrawHandle) -> bool {
        let m = d.get_mouse_position();

        let mouse_over = unsafe { ffi::CheckCollisionPointRec(m.into(), rect.into()) };
        let clicked = d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        Self::render(self, rect, mouse_over, d);
        mouse_over && clicked
    }

    fn render(&self, rect: Rectangle, mouse_over: bool, d: &mut RaylibDrawHandle) {
        let bg = if mouse_over {
            style::HOVER_COLOR
        } else {
            style::SECONDARY_COLOR
        };

        let mut text_x: i32 = rect.x as i32;

        if SETTINGS.read().unwrap().center_text {
            text_x += rect.width as i32 / 2;
            text_x -= d.measure_text(&self.txt, layout::FONT_SIZE) / 2;
        }

        d.draw_rectangle_rec(rect, bg);
        d.draw_text(
            &self.txt,
            text_x + layout::SPACING as i32 * 2,
            (rect.y + layout::SPACING + rect.height / 2. - (layout::FONT_SIZE as f32 / 2.)) as i32,
            layout::FONT_SIZE,
            style::PRIMARY_COLOR,
        );
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IconButton {
    pub icon_rec: Rectangle,
}

impl IconButton {
    pub fn new(
        icon_rec: Rectangle,
    ) -> IconButton {
        Self {
            icon_rec
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, rect: Rectangle) -> bool {
        let m = d.get_mouse_position();

        let mouse_over = unsafe { ffi::CheckCollisionPointRec(m.into(), rect.into()) };
        let clicked = d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        Self::render(self, rect, mouse_over, d);
        mouse_over && clicked
    }

    fn render(&self, rect: Rectangle, mouse_over: bool, d: &mut RaylibDrawHandle) {
        let txt = crate::ICONS_SPRITESHEET.get().unwrap();
        let mut txt_cntr: Vector2 = Vector2 { x: rect.x, y: rect.y };
        let negative: Vector2 = Vector2 { x: rect.width - self.icon_rec.width, y: rect.height - self.icon_rec.height };
        txt_cntr -= negative;
        txt_cntr.x += negative.x*1.5;
        txt_cntr.y += negative.y*1.5;
        let bg = if mouse_over {
            style::HOVER_COLOR
        } else {
            style::SECONDARY_COLOR
        };

        d.draw_rectangle_rec(rect, bg);
        d.draw_texture_rec(txt, self.icon_rec, txt_cntr, style::PRIMARY_COLOR);
    }
}

#[derive(Debug)]
pub struct Room {
    pub base_button: Button,
    pub id: String,
    pub country: String,
    pub player_info: Vector2,
    pub map_rec: Rectangle,
    pub player_label: String,
    pub distance_km: f64,
    pub distance_text: String,
    pub locked: bool,
}

impl Room {
    pub fn new(
        text: String,
        id: String,
        country: String,
        player_info: Vector2,
        map_rec: Rectangle,
        player_label: String,
        distance_km: f64,
        distance_text: String,
        locked: bool,
    ) -> Self {
        Self {
            base_button: Button::new(text),
            id,
            country,
            player_info,
            map_rec,
            player_label,
            distance_km,
            distance_text,
            locked,
        }
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, rect: Rectangle) -> bool {
        let pcl_offset: f32 = rect.width + rect.x + layout::SPACING;
        let player_count_rec: Rectangle =
            rrect(pcl_offset, rect.y, layout::PLAYER_COUNT_WIDTH, rect.height);

        d.draw_rectangle_rec(player_count_rec, style::SECONDARY_COLOR);
        d.draw_text(
            &self.player_label,
            (player_count_rec.x + layout::SPACING) as i32,
            (rect.y + layout::SPACING) as i32,
            layout::FONT_SIZE,
            style::PRIMARY_COLOR,
        );

        let flag_bg_rect: Rectangle = rrect(
            player_count_rec.x + player_count_rec.width + layout::SPACING,
            rect.y,
            layout::FLAG_SIZE.y + layout::DISTANCE_WIDTH as f32,
            rect.height,
        );
        d.draw_rectangle_rec(flag_bg_rect, style::SECONDARY_COLOR);

        if SETTINGS.read().unwrap().show_flag_images {
            let position = Vector2 {
                x: flag_bg_rect.x + layout::SPACING,
                y: rect.y + layout::SPACING,
            };
            if let Some(tex) = FLAGS_SPRITESHEET.get() {
                d.draw_texture_rec(tex, self.map_rec, position, raylib::color::Color::WHITE);
            }
        } else {
            d.draw_text(
                &self.country,
                (flag_bg_rect.x + layout::SPACING) as i32,
                (rect.y + layout::SPACING) as i32,
                layout::FONT_SIZE,
                style::PRIMARY_COLOR,
            );
        }

        
        d.draw_text(
            &self.distance_text,
            (layout::FLAG_SIZE.x + flag_bg_rect.x) as i32,
            (rect.y + layout::SPACING * 2.) as i32,
            layout::FONT_SIZE,
            style::PRIMARY_COLOR,
        );
        
        let lock_bg_rect = rrect(
            rect.x
                + rect.width
                + layout::SPACING * 3.0
                + layout::PLAYER_COUNT_WIDTH as f32
                + flag_bg_rect.width,
            rect.y,
            layout::FONT_SIZE as f32 + layout::SPACING,
            rect.height,
        );
        d.draw_rectangle_rec(lock_bg_rect, style::SECONDARY_COLOR);

        let lock_text_rec = Rectangle {
            x: 35.,
            y: 0.,
            width: 9.,
            height: 11.,
        };
        if self.locked
            && let Some(tex) = crate::ICONS_SPRITESHEET.get()
        {
            d.draw_texture_rec(
                tex,
                lock_text_rec,
                Vector2 {
                    y: lock_bg_rect.y + layout::SPACING,
                    x: lock_bg_rect.x + layout::SPACING,
                },
                raylib::color::Color::WHITE,
            );
        }

        self.base_button.draw(rect, d)
    }
}
