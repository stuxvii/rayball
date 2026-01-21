use clipboard_rs::Clipboard;
use raylib::{math::rrect, prelude::{*}, rgui::RaylibDrawGui};

use crate::{FLAGS_SPRITESHEET, ICONS_SPRITESHEET, ProgramState, Screens, cfg::layout, net::join::{parse_code, request_room_join}, ui::{flags, state::AppState}};
use crate::net::rooms;
use crate::ui::primitives::Room;
use tokio::sync::mpsc;

pub fn thread_fetch(tx: mpsc::UnboundedSender<Result<Vec<Room>, String>>) {
    tokio::spawn(async move {
        let data = tokio::task::spawn_blocking(move || {
            rooms::fetch_rooms(FLAGS_SPRITESHEET.get().unwrap())
        })
        .await
        .unwrap_or(Err("Thread join failed".to_string()));
        let _ = tx.send(data);
    });
}
pub fn draw_menu(d: &mut RaylibDrawHandle, state: &mut AppState, screen_width:i32, screen_height:i32, dt: f32 ) {
    let time_txt = chrono::Local::now().format(if cfg_val!(atomget MILITARY_TIME) { "%H:%M:%S" } else { "%I:%M:%S" }).to_string();
    d.draw_rectangle(
        0,
        0,
        screen_width,
        layout::BUTTON_HEIGHT as i32,
        clr_val!(SECONDARY_COLOR),
    );
    d.draw_text(
        &time_txt,
        (screen_width / 2) - (state.text_widths.get("clck").unwrap() / 2),
        layout::SPACING as i32 * 2,
        layout::FONT_SIZE,
        clr_val!(PRIMARY_COLOR),
    );

    for (i, btn) in state.navbar_buttons.iter().enumerate() {
        let rect = rrect(
            i as f32 * layout::BUTTON_HEIGHT as f32,
            0,
            layout::BUTTON_HEIGHT,
            layout::BUTTON_HEIGHT,
        );
        if d.gui_button(rect, "") {
            match btn.screen {
                Screens::GithubLink => {
                    open_url("https://github.com/stuxvii/rayball");
                }
                _ => state.current_screen = btn.screen,
            }
        }
        if let Some(txt) = ICONS_SPRITESHEET.get() {
            let mut txt_cntr = Vector2 {
                x: rect.x,
                y: rect.y,
            };
            let negative = Vector2 {
                x: rect.width - btn.rect.width,
                y: rect.height - btn.rect.height,
            };
            txt_cntr.x -= negative.x;
            txt_cntr.y -= negative.y;
            txt_cntr.x += negative.x * 1.5;
            txt_cntr.y += negative.y * 1.5;

            d.draw_texture_rec(txt, btn.rect, txt_cntr, clr_val!(PRIMARY_COLOR));
        }
    }
    
    let flag_coords = flags::get_vector_from_code(&cfg_val!(COUNTRY));
    let flags_rec = Rectangle::new(
        (FLAGS_SPRITESHEET.get().unwrap().width as f32) - flag_coords.x,
        (FLAGS_SPRITESHEET.get().unwrap().height as f32) - flag_coords.y,
        16.0,
        11.0,
    );
    let mut flag_position = Vector2::new(screen_width as f32-flags_rec.width-layout::SPACING, layout::SPACING);
    flag_position.x -= d.measure_text(&cfg_val!(USERNAME), layout::FONT_SIZE) as f32;
    d.draw_text(&cfg_val!(USERNAME), flag_position.x as i32 + flags_rec.width as i32, flag_position.y as i32 + layout::SPACING as i32, layout::FONT_SIZE, clr_val!(PRIMARY_COLOR));
    flag_position.x -= layout::SPACING;
    if let Some(tex) = FLAGS_SPRITESHEET.get() {
        d.draw_texture_rec(tex, flags_rec, flag_position, raylib::color::Color::WHITE);
    }

    match state.current_screen {
        Screens::ServerList => draw_server_list(d, state, screen_width, screen_height, dt),
        Screens::Configuration => draw_configuration(d, state, screen_width, screen_height),
        _ => (),
    }
}

fn draw_server_list(d: &mut RaylibDrawHandle, state: &mut AppState, screen_width: i32, screen_height: i32, dt: f32) {
    if d.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && d.is_key_pressed(KeyboardKey::KEY_V) {
        let clip: String = state.clipboard_ctx.get_text().unwrap();
        match parse_code(clip) {
            Ok(c) => {
                state.websocket_future = Some(Box::pin(request_room_join(c)));
                state.program_state = ProgramState::Joining;
            }
            Err(e) => {
                state.push_error(e.to_string(), true);
            }
        }
    }

    let wheel = d.get_mouse_wheel_move();
    state.rooms_per_page = ((screen_height as f32 / layout::BUTTON_HEIGHT) - layout::BUTTON_HEIGHT) as usize;

    if !(!cfg_val!(atomget AUTO_FETCH) && !state.rooms_fetched_once) && !state.rooms_fetching {
        state.rooms_fetched_once = true;
        thread_fetch(state.tx.clone());
        state.rooms_fetching = true
    }

    if let Ok(data) = state.rx.try_recv() {
        match data {
            Ok(v) => {
                state.rooms_list = v;
                state.rooms_fetch_error = String::new();
            }
            Err(e) => {
                state.rooms_list = vec![];
                state.rooms_fetch_error = e;
            }
        };
        state.rooms_fetched = true;
        state.amount_of_dots_in_loading_text = 0.;
    }

    if state.rooms_list.len() == 0 && state.rooms_fetched {
        let mut txt_x = screen_width / 2;
        let txt_y = screen_height / 2;
        txt_x -= d.measure_text(&state.rooms_fetch_error, layout::FONT_SIZE) / 2;
        d.draw_text(&state.rooms_fetch_error, txt_x, txt_y, layout::FONT_SIZE, clr_val!(PRIMARY_COLOR));
        
        let offline_rec = rrect(22, 0, 13, 11);
        if let Some(txt) = ICONS_SPRITESHEET.get() {
            d.draw_texture_rec(
                txt,
                offline_rec,
                Vector2 {
                    x: 0. + layout::SPACING,
                    y: screen_height as f32 - offline_rec.height - layout::SPACING,
                },
                raylib::color::Color::WHITE,
            );
        }
    }

    if !state.rooms_fetching && d.is_key_pressed(KeyboardKey::KEY_R) {
        state.rooms_list = vec![];
        state.rooms_fetching = false;
        state.rooms_fetched = false;
        state.rooms_fetched_once = true;
    }

    if state.rooms_fetched {
        let mut room_list_x = screen_width / 2;
        let mut room_list_y = screen_height / 2;
        room_list_x -= state.text_widths.get("list").unwrap() / 2;
        room_list_x -= layout::FLAG_SIZE.x as i32;
        room_list_x -= layout::DISTANCE_WIDTH;
        room_list_x += layout::FONT_SIZE;
        room_list_x += 6;
        room_list_x -= layout::SPACING as i32 * 3;
        room_list_y -= (layout::BUTTON_HEIGHT as i32 * state.rooms_per_page as i32) / 2;

        if wheel < 0.0 {
            state.scroll_offset = (state.scroll_offset + state.scroll_amount).min(state.rooms_list.len().saturating_sub(state.rooms_per_page));
        } else if wheel > 0.0 {
            state.scroll_offset = state.scroll_offset.saturating_sub(state.scroll_amount);
        }

        let mut join_id = None;

        let visible_rooms = state.rooms_list
            .iter_mut()
            .enumerate()
            .skip(state.scroll_offset)
            .take(state.rooms_per_page);

        for (display_index, (_, room)) in visible_rooms.enumerate() {
            let y = display_index as f32 * (layout::BUTTON_HEIGHT + layout::SPACING);
            let rect = Rectangle {
                x: room_list_x as f32,
                y: room_list_y as f32 + y,
                width: *state.text_widths.get("list").unwrap() as f32,
                height: layout::BUTTON_HEIGHT,
            };
            if room.draw(d, rect) {
                join_id = Some(room.id.clone());
            };
        }

        if let Some(id) = join_id {
            state.websocket_future = Some(Box::pin(request_room_join(id)));
            state.program_state = ProgramState::Joining;
        }

        if d.is_key_pressed(KeyboardKey::KEY_R) {
            state.rooms_list = vec![];
            state.rooms_fetching = false;
            state.rooms_fetched = false;
        }
    } else if state.rooms_fetched_once {
        state.amount_of_dots_in_loading_text += 10. * dt;
        let datextitself = format!(
            "Fetching rooms{}",
            ".".repeat(state.amount_of_dots_in_loading_text as usize)
        );
        let mut txt_x = screen_width / 2;
        let txt_y = screen_height / 2;
        txt_x -= d.measure_text(&datextitself, layout::FONT_SIZE) / 2;
        d.draw_text(&datextitself, txt_x, txt_y, layout::FONT_SIZE, clr_val!(PRIMARY_COLOR));
    }
}

fn draw_configuration(d: &mut RaylibDrawHandle, state: &mut AppState, screen_width: i32, screen_height: i32) {
    let setting_toggles_len = state.setting_toggles.len() as f32;
    for (display_index, btn) in state.setting_toggles.iter_mut().enumerate() {
        let mut rect = rrect(
            screen_width / 2,
            layout::BUTTON_HEIGHT * display_index as f32,
            layout::FONT_SIZE,
            layout::FONT_SIZE,
        );
        rect.y += (screen_height / 2) as f32;
        rect.y -= (layout::BUTTON_HEIGHT * setting_toggles_len) / 2.;
        rect.x -= 40.;

        let old_val = btn.target.load(std::sync::atomic::Ordering::Relaxed);
        let mut current_val: bool = old_val;
        if d.gui_check_box(rect, &btn.text, &mut current_val) {
            btn.target.store(current_val, std::sync::atomic::Ordering::Relaxed);
            if let Some(act) = btn.callback {
                act(current_val, d);
            }
        }
    }
}