use std::sync::atomic::AtomicBool;

use raylib::math::rrect;
use raylib::prelude::RaylibDraw;
use raylib::prelude::RaylibDrawHandle;
use raylib::prelude::*;

use crate::FLAGS_SPRITESHEET;
use crate::ICONS_SPRITESHEET;
use crate::cfg::layout;

#[derive(Debug)]
pub struct Room {
    pub text: String,
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
    #[allow(clippy::too_many_arguments)]
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
            text,
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
        let clicked = d.gui_button(rect, "");

        let txt_x: f32 = rect.x + layout::SPACING;

        d.draw_text(
            &self.text, 
            txt_x as i32, 
            (rect.y + layout::SPACING * 2.) as i32, 
            layout::FONT_SIZE, 
            clr_val!(PRIMARY_COLOR)
        );

        let pcl_offset = rect.width + rect.x + layout::SPACING;
        let player_count_rec = rrect(pcl_offset, rect.y, layout::PLAYER_COUNT_WIDTH, rect.height);

        d.draw_rectangle_rec(player_count_rec, clr_val!(SECONDARY_COLOR));
        d.draw_text(
            &self.player_label,
            (player_count_rec.x + layout::SPACING) as i32,
            (rect.y + (layout::BUTTON_HEIGHT - layout::FONT_SIZE as f32) - layout::SPACING) as i32,
            layout::FONT_SIZE,
            clr_val!(PRIMARY_COLOR),
        );

        let flag_bg_rect = rrect(
            player_count_rec.x + player_count_rec.width + layout::SPACING + layout::SPACING,
            rect.y,
            layout::FLAG_SIZE.y + layout::DISTANCE_WIDTH as f32,
            rect.height,
        );
        d.draw_rectangle_rec(flag_bg_rect, clr_val!(SECONDARY_COLOR));

        let position = Vector2 {
            x: flag_bg_rect.x + layout::SPACING,
            y: rect.y + layout::SPACING,
        };
        if let Some(tex) = FLAGS_SPRITESHEET.get() {
            d.draw_texture_rec(tex, self.map_rec, position, raylib::color::Color::WHITE);
        }

        d.draw_text(
            &self.distance_text,
            (layout::FLAG_SIZE.x + flag_bg_rect.x) as i32,
            (rect.y + layout::SPACING * 2.0) as i32,
            layout::FONT_SIZE,
            clr_val!(PRIMARY_COLOR),
        );

        let lock_bg_rect = rrect(
            rect.x + rect.width + layout::SPACING * 4.0 + layout::PLAYER_COUNT_WIDTH as f32 + flag_bg_rect.width,
            rect.y,
            layout::FONT_SIZE as f32 + layout::SPACING,
            rect.height,
        );
        d.draw_rectangle_rec(lock_bg_rect, clr_val!(SECONDARY_COLOR));

        let lock_text_rec = Rectangle {
            x: 35.0,
            y: 0.0,
            width: 9.0,
            height: 11.0,
        };

        if self.locked && let Some(tex) = ICONS_SPRITESHEET.get() {
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

        clicked
    }
}

pub struct SettingToggle {
    pub text: String,
    pub target: &'static AtomicBool,
    pub callback: Option<fn(bool, &mut RaylibDrawHandle)>,
}

impl SettingToggle {
    pub fn new(text: String, target: &'static AtomicBool, callback: Option<fn(bool, &mut RaylibDrawHandle)>,) -> SettingToggle {
        SettingToggle {
            text,
            target,
            callback
        }
    }
}

pub struct SettingData {
    pub text: String,
    pub target: &'static AtomicBool,
    pub callback: Option<fn(bool, &mut RaylibDrawHandle)>,
}

impl SettingData {
    pub fn new(text: String, target: &'static AtomicBool, callback: Option<fn(bool, &mut RaylibDrawHandle)>) -> Self {
        Self { text, target, callback }
    }
}