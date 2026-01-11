use chrono::prelude::*;
use rayball_rs::cfg::config::*;
use rayball_rs::cfg::{layout, style, config};
use rayball_rs::net::rooms;
use rayball_rs::ui::cursor::CursorTrail;
use rayball_rs::ui::primitives::{IconButton, Room, SettingToggle};
use rayball_rs::*;
use raylib::prelude::*;
use std::collections::HashMap;
use std::fs::write;
use std::sync::mpsc::{self, Sender};
use std::{thread, vec};
use tinyjson::JsonValue;

fn save_config() {
    let mut map = HashMap::new();

    map.insert("scrolling_bg".to_string(), cfg_val!(atomget SCROLLING_BACKGROUND).into());
    map.insert("show_flags".to_string(), cfg_val!(atomget SHOW_FLAG_IMAGES).into());
    map.insert("fancy_cursor".to_string(), cfg_val!(atomget FANCY_CURSOR).into());
    map.insert("center_text".to_string(), cfg_val!(atomget CENTER_TEXT).into());
    map.insert("show_fps".to_string(), cfg_val!(atomget SHOW_FPS).into());
    map.insert("military_time".to_string(), cfg_val!(atomget MILITARY_TIME).into());
    
    map.insert("longitude".to_string(), JsonValue::Number(cfg_val!(LONGITUDE) as f64));
    map.insert("latitude".to_string(), JsonValue::Number(cfg_val!(LATITUDE) as f64));
    map.insert("fps".to_string(), JsonValue::Number(cfg_val!(FPS) as f64));

    let root = JsonValue::Object(map);
    let _ = write("./rb.cfg", root.stringify().unwrap());
}

fn thread_fetch(tx: Sender<Result<Vec<Room>, String>>) {
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let data: Result<Vec<Room>, String> = rooms::fetch_rooms(FLAGS_SPRITESHEET.get().unwrap());
        let _ = tx_clone.send(data);
    });
}

fn main() {
    
    match load_settings() {
        Ok(_) => {
            println!("Successfully loaded configuration!");
        }
        Err(e) => {
            println!(
                "Unable to load config! Will need to generate a new one. Error is: {}",
                e
            );
            save_config();
        }
    }

    let (mut rl, rt) = raylib::init()
        .resizable()
        .title("rayball")
        .size(640, 480)
        .build();
    
    rl.set_target_fps(cfg_val!(FPS));
    rl.set_window_min_size(480/2, 360/2);
    
    match load_spritesheets(&mut rl, &rt) {
        Ok(_) => {println!("Loaded textures.")},
        Err(e) => {println!("Epic fail loading textures: {}.", e)},
    };

    let image_size = 64;
    let mut image = Image::gen_image_color(image_size, image_size, Color::RED);

    let image_size_f: f32 = image_size as f32;
    
    image.draw_line_ex(Vector2 {x: image_size_f, y: 0.}, Vector2 {x: 0., y: image_size_f}, image_size/2, style::BG_COLOR2);
    image.draw_line_ex(Vector2 {x: 0., y: 0.}, Vector2 {x: image_size_f, y: image_size_f}, image_size, style::BG_COLOR1);
    image.draw_line_ex(Vector2 {x: 0., y: 0.}, Vector2 {x: image_size_f, y: image_size_f}, (image_size_f/2.2) as i32, style::BG_COLOR2);

    let checkerboard_bg: Texture2D = rl.load_texture_from_image(&rt, &image).unwrap();

    let mut trail: CursorTrail = CursorTrail::new();
    if cfg_val!(atomget FANCY_CURSOR) {
        rl.hide_cursor();
    }

    let offline_rec = rrect(22, 0, 13, 11);
    let navbar_buttons: Vec<IconButton> = vec![
        IconButton::new(rrect(0.0, 0.0, 11.0, 11.0), Screens::ServerList),
        IconButton::new(rrect(11.0, 0.0, 11.0, 11.0), Screens::Configuration),
        IconButton::new(rrect(44.0, 0.0, 11.0, 11.0), Screens::GithubLink),
    ];

    let mut setting_toggles: Vec<SettingToggle> = vec![
        SettingToggle::new("Show Flags".to_string(), &SHOW_FLAG_IMAGES, None),
        SettingToggle::new("Fancy Cursor".to_string(), &FANCY_CURSOR, Some(|on, d| {
        if on {
            println!("HIDE THE CURSOR BITCH");
            d.hide_cursor();
        } else {
            println!("SHOW THE CURSOR BITCH");
            d.show_cursor();
        }
    })),
        SettingToggle::new("Scrolling BG".to_string(), &SCROLLING_BACKGROUND, None),
        SettingToggle::new("Show FPS".to_string(), &SHOW_FPS, None),
        SettingToggle::new("Center Text".to_string(), &CENTER_TEXT, None),
        SettingToggle::new("24H Clock".to_string(), &MILITARY_TIME, None),
    ];

    let mut rooms_fetch_error: String = "".to_string();
    let mut rooms_list: Vec<Room> = vec![];
    let mut rooms_fetching = false;
    let mut rooms_fetched = false;
    let mut scroll_offset: usize = 0;
    let rooms_per_page: usize = 24;
    let scroll_amount: usize = 3;

    let mut current_screen: Screens = Screens::ServerList;
    let mut amount_of_dots_in_loading_text: f32 = 0.;
    let text_widths = HashMap::from([
        ("list", rl.measure_text(&"W".repeat(48), layout::FONT_SIZE)),
        ("clck", rl.measure_text(&"00:00:00", layout::FONT_SIZE)),
    ]);

    let bpm = 163.;
    let mut bg_scroll: f32 = 0.0;
    let mut bg_scroll_speed = 64.;
    
    let (tx, rx) = mpsc::channel::<Result<Vec<Room>, String>>();

    let mut time_timer: f32 = 0.;
    let mut time_txt:String = Local::now().format("%H:%M:%S").to_string();

    let yameroing_it = false;
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&rt);
        let fps = format!("{}", d.get_fps());
        let screen_width = d.get_screen_width();
        let screen_height = d.get_screen_height();
        let dt = d.get_frame_time();

        d.draw_texture_rec(&checkerboard_bg,rrect(-bg_scroll, bg_scroll, screen_width as f32, screen_height as f32), Vector2::zero(), Color::WHITE);
        

        match cfg_val!(atomget SCROLLING_BACKGROUND) {
            true => {
                bg_scroll -= bg_scroll_speed * dt;
                if bg_scroll <= (-checkerboard_bg.width) as f32 {
                    bg_scroll = 0.0
                };
                if yameroing_it {
                    if bg_scroll_speed > 0. {
                        bg_scroll_speed -= (bpm * dt) * (bpm / 60.);
                    } else {
                        bg_scroll_speed  = bpm;
                    }
                }
            }
            false => (),
        }

        d.draw_rectangle(
            0,
            0,
            screen_width,
            layout::BUTTON_HEIGHT as i32,
            Color {
                r: 0,
                g: 0,
                b: 0,
                a: 191,
            },
        );

        // we can actually start drawing things now
        
        time_timer += dt;

        if time_timer > 0.5 { // wouldn't want to just hammer this func
            if cfg_val!(atomget MILITARY_TIME) {
                time_txt = Local::now().format("%H:%M:%S").to_string();
            } else {
                time_txt = Local::now().format("%I:%M:%S").to_string();
            }
            time_timer = 0.;
        }

        d.draw_text(&time_txt, (screen_width / 2) - (text_widths.get("clck").unwrap() / 2), layout::BUTTON_HEIGHT as i32 - layout::FONT_SIZE, layout::FONT_SIZE, style::PRIMARY_COLOR);
        
        for (i, btn) in navbar_buttons.clone().into_iter().enumerate() {
            let state = btn.draw(
                &mut d,
                rrect(
                    i as f32 * layout::BUTTON_HEIGHT as f32,
                    0,
                    layout::BUTTON_HEIGHT,
                    layout::BUTTON_HEIGHT,
                ),
            );
            if let Some(new_screen) = state {
                match new_screen {
                    Screens::GithubLink => {
                        open_url("https://github.com/stuxvii/rayball");
                    }
                    _ => current_screen = new_screen,
                }
            };
        }

        match current_screen {
            Screens::ServerList => {
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
                    match data {
                        Ok(v) => {
                            rooms_list = v;
                            rooms_fetch_error = String::new();
                        },
                        Err(e) => {
                            rooms_list = vec![];
                            rooms_fetch_error = e;
                        }                    
                    };
                    rooms_fetched = true;
                    amount_of_dots_in_loading_text = 0.;
                }

                if rooms_list.len() == 0 && rooms_fetched {
                    let mut txt_x = screen_width / 2;
                    let txt_y = screen_height / 2;
                    txt_x -= d.measure_text(&rooms_fetch_error, layout::FONT_SIZE) / 2;
                    d.draw_text(
                        &rooms_fetch_error,
                        txt_x,
                        txt_y,
                        layout::FONT_SIZE,
                        style::PRIMARY_COLOR,
                    );
                    if let Some(txt) = ICONS_SPRITESHEET.get() {
                        d.draw_texture_rec(txt, offline_rec, Vector2 {x: 0. + layout::SPACING, y: screen_height as f32 - offline_rec.height - layout::SPACING}, raylib::color::Color::WHITE);
                    }
                }

                if rooms_fetched {
                    let mut room_list_x = screen_width / 2;
                    let mut room_list_y = screen_height / 2;
                    room_list_x -= text_widths.get("list").unwrap() / 2;
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
                            width: *text_widths.get("list").unwrap() as f32,
                            height: layout::BUTTON_HEIGHT,
                        };
                        if room.draw(&mut d, rect) {};
                    }
                                        
                    if d.is_key_pressed(KeyboardKey::KEY_R) {
                        rooms_list = vec![];
                        rooms_fetching = false;
                        rooms_fetched = false;
                    }
                } else {
                    amount_of_dots_in_loading_text += 10. * dt;
                    let datextitself = format!(
                        "Fetching rooms{}",
                        ".".repeat(amount_of_dots_in_loading_text as usize)
                    );
                    let mut txt_x = screen_width / 2;
                    let txt_y = screen_height / 2;
                    txt_x -= d.measure_text(&datextitself, layout::FONT_SIZE) / 2;
                    d.draw_text(
                        &datextitself,
                        txt_x,
                        txt_y,
                        layout::FONT_SIZE,
                        style::PRIMARY_COLOR,
                    );
                }
            }
            Screens::Configuration => {
                let setting_toggles_len = setting_toggles.len() as f32;
                for (display_index, btn) in setting_toggles.iter_mut().enumerate() {
                    let btn_width = 128;
                    let mut rect = rrect(
                        screen_width / 2,
                        layout::BUTTON_HEIGHT * display_index as f32,
                        btn_width,
                        layout::FONT_SIZE,
                    );
                    rect.y += (screen_height/2) as f32;
                    rect.y -= (layout::BUTTON_HEIGHT * setting_toggles_len) / 2.;
                    rect.x -= (btn_width / 2) as f32;
                    let clicked = btn.draw(rect, &mut d);
                    if clicked {
                        let on = btn.target.load(std::sync::atomic::Ordering::Relaxed);
                        btn.target.store(on, std::sync::atomic::Ordering::Relaxed);
                        
                        if let Some(act) = btn.callback {
                            act(on, &mut d);
                        }
                    }
                }
            }
            _ => (),
        }

        if cfg_val!(atomget FANCY_CURSOR) {
            trail.draw(dt, &mut d);
        }

        // It is very much intentional that the FPS draw over the Fancy Cusror (That typo in cursor is also very much intended).
        if cfg_val!(atomget SHOW_FPS) {
            let fps_width = d.measure_text(&fps, layout::FONT_SIZE);
            d.draw_rectangle(screen_width-fps_width, screen_height-layout::FONT_SIZE+layout::SPACING as i32, fps_width, layout::BUTTON_HEIGHT as i32, Color::new(0, 0, 0, 127));
            d.draw_text(&fps, screen_width-fps_width-layout::SPACING as i32, screen_height-layout::FONT_SIZE+layout::SPACING as i32, layout::FONT_SIZE, style::PRIMARY_COLOR);
        }
    }

    save_config();
}
