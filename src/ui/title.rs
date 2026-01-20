use raylib::prelude::*;
use crate::cfg::{config, layout};
use crate::*;
use crate::ui::state::AppState;

pub fn draw_ask_info(d: &mut RaylibDrawHandle, state: &mut AppState, program_name: &str, screen_width: i32, screen_height: i32, dt: f32) {
    let amplitude;
    if state.logo_letter_amp_timer < state.logo_letter_amp_tween.duration() {
        state.logo_letter_amp_timer += dt;
        amplitude = state.logo_letter_amp_tween.apply(state.logo_letter_amp_timer);
    } else {
        amplitude = 4.;
    }
    
    for (i, c) in program_name.chars().enumerate() {
        let spacing = 25.;
        let mut x = i as f32 * spacing;
        x += screen_width as f32 / 2.;
        x += spacing / 2.;
        x -= (program_name.len() as f32 / 2.) * spacing;
        let y = spacing + (d.get_time() as f32 * 4.0 + i as f32).sin() * amplitude;

        for i in 0..4 {
            d.draw_text_pro(
                d.get_font_default(),
                &String::from(c),
                Vector2::new(x + i as f32, y + i as f32),
                Vector2::zero(),
                y - spacing,
                40.,
                0.,
                clr_val!(SECONDARY_COLOR),
            );
        }
        d.draw_text_pro(
            d.get_font_default(),
            &String::from(c),
            Vector2::new(x, y),
            Vector2::zero(),
            y - spacing,
            40.,
            0.,
            clr_val!(PRIMARY_COLOR),
        );
    }

    let go_rect = rrect((screen_width / 2) - 64, screen_height - 48, 128, 32);

    let mut username_rec = rrect(
        screen_width / 2 - *state.text_widths.get("usnm").unwrap() / 2,
        0,
        *state.text_widths.get("usnm").unwrap(),
        layout::BUTTON_HEIGHT,
    );
    username_rec.y -= layout::BUTTON_HEIGHT * 2.;
    username_rec.y += go_rect.y;

    let mut username: std::sync::MutexGuard<'_, String> = config::USERNAME.lock().unwrap();
    let mut username_hover: bool = false;

    if username_rec.check_collision_point_rec(d.get_mouse_position()) {
        username_hover = true;
        if d.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            username.pop();
        }
        while let Some(c) = d.get_char_pressed() {
            if username.chars().count() < 24 {
                username.push(c);
            }
        }
    }
    d.draw_rectangle_rec(username_rec, clr_val!(SECONDARY_COLOR));
    
    if username.is_empty() {
        d.draw_text(
            "enter your name.",
            username_rec.x as i32 + 2,
            username_rec.y as i32 + 2,
            layout::FONT_SIZE,
            clr_val!(TERNARY_COLOR),
        );
    }

    if username_hover {
        d.draw_rectangle_lines_ex(
            username_rec,
            layout::SPACING,
            clr_val!(TERNARY_COLOR),
        );
        d.draw_rectangle(username_rec.x as i32 + d.measure_text(&username, layout::FONT_SIZE) + 2, username_rec.y as i32 + 2, 4, layout::FONT_SIZE-1, clr_val!(PRIMARY_COLOR));
    }

    d.draw_text(
        &username,
        username_rec.x as i32 + 2,
        username_rec.y as i32 + 2,
        layout::FONT_SIZE,
        clr_val!(PRIMARY_COLOR),
    );

    if !username.is_empty() && (d.gui_button(go_rect, "Let's get it on!") || d.is_key_pressed(KeyboardKey::KEY_ENTER) ){
        state.program_state = ProgramState::Menu;
    }
}