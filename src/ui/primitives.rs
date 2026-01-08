use raylib::math::rrect;
use raylib::prelude::RaylibDraw;
use raylib::prelude::RaylibDrawHandle;
use raylib::prelude::*;

use crate::FLAGS_SPRITESHEET;
use crate::ICONS_SPRITESHEET;
use crate::Screens;
use crate::Settings;
use crate::cfg::config;
use crate::cfg::layout;
use crate::cfg::style;

pub struct Interaction;

impl Interaction {
    pub fn check(rect: Rectangle, d: &RaylibDrawHandle) -> (bool, bool) {
        let m = d.get_mouse_position();
        let mouse_over = rect.check_collision_point_rec(m);
        let clicked = d.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        (mouse_over, clicked)
    }

    pub fn resolve_color(mouse_over: bool, toggled: bool) -> Color {
        if toggled {
            style::GREEN_COLOR
        } else if mouse_over {
            style::HOVER_COLOR
        } else {
            style::SECONDARY_COLOR
        }
    }
}

pub trait ButtonContent {
    fn draw_content(&self, rect: Rectangle, d: &mut RaylibDrawHandle, mouse_over: bool);
}

impl ButtonContent for IconButton {
    fn draw_content(&self, rect: Rectangle, d: &mut RaylibDrawHandle, mouse_over: bool) {
        let bg_color = Interaction::resolve_color(mouse_over, false);
        d.draw_rectangle_rec(rect, bg_color);

        if let Some(txt) = ICONS_SPRITESHEET.get() {
            let mut txt_cntr = Vector2 {
                x: rect.x,
                y: rect.y,
            };
            let negative = Vector2 {
                x: rect.width - self.icon_rec.width,
                y: rect.height - self.icon_rec.height,
            };
            
            txt_cntr.x -= negative.x;
            txt_cntr.y -= negative.y;
            txt_cntr.x += negative.x * 1.5;
            txt_cntr.y += negative.y * 1.5;

            d.draw_texture_rec(txt, self.icon_rec, txt_cntr, style::PRIMARY_COLOR);
        }
    }
}

impl ButtonContent for Button {
    fn draw_content(&self, rect: Rectangle, d: &mut RaylibDrawHandle, mouse_over: bool) {
        let bg_color = Interaction::resolve_color(mouse_over, false);
        d.draw_rectangle_rec(rect, bg_color);

        let mut text_x = rect.x as i32;
        if *config::CENTER_TEXT.lock().unwrap() {
            text_x += (rect.width as i32 / 2) - (d.measure_text(&self.text, layout::FONT_SIZE) / 2);
        } else {
            text_x += layout::SPACING as i32 * 2;
        }

        let text_y = (rect.y + layout::SPACING + rect.height / 2.0 - (layout::FONT_SIZE as f32 / 2.0)) as i32;
        
        d.draw_text(
            &self.text,
            text_x,
            text_y,
            layout::FONT_SIZE,
            style::PRIMARY_COLOR,
        );
    }
}

impl ButtonContent for Room {
    fn draw_content(&self, rect: Rectangle, d: &mut RaylibDrawHandle, mouse_over: bool) {
        let bg_color = Interaction::resolve_color(mouse_over, false);
        d.draw_rectangle_rec(rect, bg_color);
        let mut txt_x: f32 = rect.x;
        if *config::CENTER_TEXT.lock().unwrap() {
            txt_x = rect.x + rect.width / 2.;
            txt_x -= (d.measure_text(&self.text, layout::FONT_SIZE) / 2 )as f32;
        }
        d.draw_text(&self.text, txt_x as i32, (rect.y + layout::SPACING*2.) as i32, layout::FONT_SIZE, style::PRIMARY_COLOR);

        let pcl_offset = rect.width + rect.x + layout::SPACING;
        let player_count_rec = rrect(pcl_offset, rect.y, layout::PLAYER_COUNT_WIDTH, rect.height);

        d.draw_rectangle_rec(player_count_rec, style::SECONDARY_COLOR);
        d.draw_text(
            &self.player_label,
            (player_count_rec.x + layout::SPACING) as i32,
            (rect.y + layout::SPACING) as i32,
            layout::FONT_SIZE,
            style::PRIMARY_COLOR,
        );

        let flag_bg_rect = rrect(
            player_count_rec.x + player_count_rec.width + layout::SPACING,
            rect.y,
            layout::FLAG_SIZE.y + layout::DISTANCE_WIDTH as f32,
            rect.height,
        );
        d.draw_rectangle_rec(flag_bg_rect, style::SECONDARY_COLOR);

        if *config::SHOW_FLAG_IMAGES.lock().unwrap(){
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
            (rect.y + layout::SPACING * 2.0) as i32,
            layout::FONT_SIZE,
            style::PRIMARY_COLOR,
        );

        let lock_bg_rect = rrect(
            rect.x + rect.width + layout::SPACING * 3.0 + layout::PLAYER_COUNT_WIDTH as f32 + flag_bg_rect.width,
            rect.y,
            layout::FONT_SIZE as f32 + layout::SPACING,
            rect.height,
        );
        d.draw_rectangle_rec(lock_bg_rect, style::SECONDARY_COLOR);

        let lock_text_rec = Rectangle {
            x: 35.0,
            y: 0.0,
            width: 9.0,
            height: 11.0,
        };

        if self.locked {
            if let Some(tex) = ICONS_SPRITESHEET.get() {
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
        }
    }
}

#[derive(Debug)]
pub struct Button {
    pub text: String,
}

impl Button {
    pub fn new(text: String) -> Button {
        Button { text }
    }

    pub fn draw(&self, rect: Rectangle, d: &mut RaylibDrawHandle) -> bool {
        let (mouse_over, clicked) = Interaction::check(rect, d);
        self.draw_content(rect, d, mouse_over);
        mouse_over && clicked
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IconButton {
    pub icon_rec: Rectangle,
    pub target: Screens,
}

impl IconButton {
    pub fn new(icon_rec: Rectangle, target: Screens) -> IconButton {
        Self {
            icon_rec,
            target,
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle, rect: Rectangle) -> Option<Screens> {
        let (mouse_over, clicked) = Interaction::check(rect, d);
        self.draw_content(rect, d, mouse_over);
        if mouse_over && clicked {
            Some(self.target)
        } else {
            None
        }
    }
}

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
        let (mouse_over, clicked) = Interaction::check(rect, d);
        self.draw_content(rect, d, mouse_over);
        clicked && mouse_over
    }
}

pub struct ToggleButton {
    pub text: String,
    pub toggled: bool,
    pub target: Settings,

}

impl ToggleButton {
    pub fn new(text: String, toggled: bool, target: Settings) -> ToggleButton {
        ToggleButton {
            text,
            toggled,
            target
        }
    }

    pub fn draw(&mut self, rect: Rectangle, d: &mut RaylibDrawHandle) -> bool {
        let (mouse_over, clicked) = Interaction::check(rect, d);

        if clicked && mouse_over {
            self.toggled = !self.toggled;
        }
        self.render(rect, mouse_over, d);
        clicked && mouse_over
    }

    fn render(&self, rect: Rectangle, mouse_over: bool, d: &mut RaylibDrawHandle) {
        let bg = Interaction::resolve_color(mouse_over, self.toggled);
        d.draw_rectangle_rec(rect, bg);

        let mut text_x = rect.x as i32;
        if *config::CENTER_TEXT.lock().unwrap() {
            text_x += (rect.width as i32 / 2) - (d.measure_text(&self.text, layout::FONT_SIZE) / 2);
        } else {
            text_x += layout::SPACING as i32 * 2;
        }

        let text_y = (rect.y + layout::SPACING + rect.height / 2.0 - (layout::FONT_SIZE as f32 / 2.0)) as i32;

        d.draw_text(
            &self.text,
            text_x,
            text_y,
            layout::FONT_SIZE,
            style::PRIMARY_COLOR,
        );
    }
}