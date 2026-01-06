use rayball_rs::cfg::params::SETTINGS;
use rayball_rs::cfg::{layout, style};
use rayball_rs::net::rooms;
use rayball_rs::ui::cursor::CursorTrail;
use rayball_rs::ui::primitives::{IconButton, Room};
use rayball_rs::{FLAGS_SPRITESHEET, load_settings, load_spritesheets};
use raylib::ffi::SetTextureWrap;
use raylib::prelude::*;
use std::sync::mpsc::{self, Sender};
use std::thread;

fn thread_fetch(tx: Sender<Vec<Room>>) {
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let data = rooms::fetch_rooms(FLAGS_SPRITESHEET.get().unwrap());
        let _ = tx_clone.send(data);
    });
}

fn main() {
    let (mut rl, rt) = raylib::init()
        .resizable()
        .title("rayball")
        .size(640, 480)
        .build();

    load_spritesheets(&mut rl, &rt);
    let res = load_settings();
    match res {
        Ok(_) => {
            println!("Successfully loaded configuration!");
        }
        Err(e) => {
            println!(
                "Unable to load config! Will need to generate a new one. Error is: {}",
                e
            );
        }
    }

    let mut image = Image::gen_image_color(64, 64, Color::BLANK);
    for y in 0..64 {
        for x in 0..64 {
            let tile_index = (x / 32 + y / 32) % 2;
            let colour = if tile_index == 0 {
                style::BG_COLOR1
            } else {
                style::BG_COLOR2
            };
            image.draw_pixel(x, y, colour);
        }
    }

    let checkerboard_bg: Texture2D = rl.load_texture_from_image(&rt, &image).unwrap();
    unsafe {
        SetTextureWrap(*checkerboard_bg, 0);
    }
    let mut bg_scroll: f32 = 0.0;

    rl.set_target_fps(SETTINGS.read().unwrap().fps);
    rl.set_window_min_size(640, 480);
    let mut trail: CursorTrail = CursorTrail::new();
    if SETTINGS.read().unwrap().fancy_cursor {
        rl.hide_cursor();
    }

    let list_width = rl.measure_text(&"W".repeat(50), layout::FONT_SIZE);

    let rooms_per_page: usize = 24;
    let mut scroll_offset: usize = 0;
    let scroll_amount: usize = 3;

    let navbar_buttons: Vec<IconButton> = vec![
        IconButton {
            icon_rec: rrect(0.0, 0.0, 11.0, 11.0),
        },
        IconButton {
            icon_rec: rrect(11.0, 0.0, 11.0, 11.0),
        },
        IconButton {
            icon_rec: rrect(44.0, 0.0, 11.0, 11.0),
        },
    ];

    let (tx, rx) = mpsc::channel::<Vec<Room>>();
    let mut rooms_fetched = false;
    let mut rooms_fetching = false;
    let mut rooms_list: Vec<Room> = vec![];
    let mut current_screen: u8 = 0;
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&rt);
        let screen_width = d.get_screen_width();
        let screen_height = d.get_screen_height();
        let dt = d.get_frame_time();

        match SETTINGS.read().unwrap().scrolling_background {
            true => {
                bg_scroll -= 25. * dt;
                if bg_scroll <= (-checkerboard_bg.width) as f32 {
                    bg_scroll = 0.0
                };
            }
            false => (),
        }

        d.draw_texture_rec(
            &checkerboard_bg,
            Rectangle {
                x: bg_scroll,
                y: 0.,
                width: screen_width as f32,
                height: screen_height as f32,
            },
            Vector2 { x: 0., y: 0. },
            Color::WHITE,
        );

        // we can actually start drawing things now

        d.draw_rectangle(
            0,
            0,
            screen_width,
            layout::BUTTON_HEIGHT as i32,
            style::SECONDARY_COLOR,
        );

        for (i, mut btn) in navbar_buttons.clone().into_iter().enumerate() {
            let state: bool = btn.draw(
                &mut d,
                rrect(
                    i as f32 * layout::BUTTON_HEIGHT as f32,
                    0,
                    layout::BUTTON_HEIGHT,
                    layout::BUTTON_HEIGHT,
                ),
            );

            if state {
                match i {
                    2 => {
                        open_url("https://github.com/stuxvii/rayball");
                    }
                    _ => current_screen = i as u8
                }
            };
        }

        match current_screen {
            0 => {
                let wheel = d.get_mouse_wheel_move();
                if wheel < 0.0 {
                    scroll_offset =
                        (scroll_offset + scroll_amount).min(rooms_list.len() - rooms_per_page);
                } else if wheel > 0.0 {
                    scroll_offset = scroll_offset.saturating_sub(scroll_amount);
                }

                if !rooms_fetching {
                    thread_fetch(tx.clone());
                    rooms_fetching = true
                }

                if let Ok(data) = rx.try_recv() {
                    rooms_list = data;
                    rooms_fetched = true;
                }

                if rooms_fetched {
                    let mut room_list_x = screen_width / 2;
                    let mut room_list_y = screen_height / 2;
                    room_list_x -= list_width / 2;
                    room_list_x -= layout::FLAG_SIZE.x as i32;
                    room_list_x -= layout::DISTANCE_WIDTH;
                    room_list_x += layout::FONT_SIZE;
                    room_list_x -= layout::SPACING as i32 * 3;
                    room_list_y -= (layout::BUTTON_HEIGHT as i32 * rooms_per_page as i32) / 2;

                    let visible_rooms = rooms_list
                        .iter_mut()
                        .enumerate()
                        .skip(scroll_offset)
                        .take(rooms_per_page);

                    for (display_index, (_, room)) in visible_rooms.enumerate() {
                        let y = display_index as f32 * (layout::BUTTON_HEIGHT + layout::SPACING);
                        let rect = Rectangle {
                            x: room_list_x as f32,
                            y: room_list_y as f32 + y,
                            width: list_width as f32,
                            height: layout::BUTTON_HEIGHT,
                        };
                        if room.draw(&mut d, rect) {};
                    }
                } else {
                    let mut txt_x = screen_width / 2;
                    let txt_y = screen_height / 2;
                    txt_x -= d.measure_text("Fetching rooms...", layout::FONT_SIZE) / 2;
                    d.draw_text(
                        "Fetching rooms...",
                        txt_x,
                        txt_y,
                        layout::FONT_SIZE,
                        style::PRIMARY_COLOR,
                    );
                }
            }
            1 => {

            }
            _ => (),
        }

        d.draw_fps(100, 0);

        if SETTINGS.read().unwrap().fancy_cursor {
            trail.draw(dt, &mut d);
        }
    }
}
