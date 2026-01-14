use chrono::prelude::*;
use clipboard_rs::{Clipboard, ClipboardContext};
use rayball_rs::cfg::config::*;
use rayball_rs::cfg::{layout, style, config};
use rayball_rs::net::rooms;
use rayball_rs::ui::cursor::CursorTrail;
use rayball_rs::ui::primitives::{Room, SettingData};
use rayball_rs::*;
use raylib::error::Error;
use raylib::prelude::*;
use std::collections::HashMap;
use std::vec;
use tokio::sync::mpsc;

fn thread_fetch(tx: mpsc::UnboundedSender<Result<Vec<Room>, String>>) {
    tokio::spawn(async move {
        let data = tokio::task::spawn_blocking(move || {
            rooms::fetch_rooms(FLAGS_SPRITESHEET.get().unwrap())
        }).await.unwrap_or(Err("Thread join failed".to_string()));
        let _ = tx.send(data);
    });
}

fn get_gui_color(style_property: i32) -> Color {
    let r = ((style_property >> 24) & 0xFF) as u8;
    let g = ((style_property >> 16) & 0xFF) as u8;
    let b = ((style_property >> 8) & 0xFF) as u8;
    let a = (style_property & 0xFF) as u8;

    Color::new(r, g, b, a)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
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

    rl.gui_load_style("./res/style.rgs");
    rl.set_target_fps(cfg_val!(FPS));
    rl.set_window_min_size(320, 360/2);

    clr_val!(SECONDARY_COLOR) = get_gui_color(rl.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BASE_COLOR_NORMAL));

    match load_spritesheets(&mut rl, &rt) {
        Ok(_) => {println!("Loaded textures.")},
        Err(e) => {panic!("Epic fail loading textures: {}.", e)},
    };

    let image_size = 64;
    let mut image = Image::gen_image_color(image_size, image_size, Color::RED);

    let image_size_f: f32 = image_size as f32;
    
    let bg_color1 = rl.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BASE_COLOR_NORMAL);
    let bg_color2 = rl.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BORDER_COLOR_NORMAL); 
    image.draw_line_ex(Vector2 {x: image_size_f, y: 0.}, Vector2 {x: 0., y: image_size_f}, image_size/2, get_gui_color(bg_color2));
    image.draw_line_ex(Vector2 {x: 0., y: 0.}, Vector2 {x: image_size_f, y: image_size_f}, image_size, get_gui_color(bg_color1));
    image.draw_line_ex(Vector2 {x: 0., y: 0.}, Vector2 {x: image_size_f, y: image_size_f}, (image_size_f/2.2) as i32, get_gui_color(bg_color2));

    let checkerboard_bg: Texture2D = rl.load_texture_from_image(&rt, &image).unwrap();

    let mut trail: CursorTrail = CursorTrail::new();
    if cfg_val!(atomget FANCY_CURSOR) {
        rl.hide_cursor();
    }

    let offline_rec = rrect(22, 0, 13, 11);
    struct NavIcon { rect: Rectangle, screen: Screens }
    let navbar_buttons: Vec<NavIcon> = vec![
        NavIcon { rect: rrect(0.0, 0.0, 11.0, 11.0), screen: Screens::ServerList },
        NavIcon { rect: rrect(11.0, 0.0, 11.0, 11.0), screen: Screens::Configuration },
        NavIcon { rect: rrect(44.0, 0.0, 11.0, 11.0), screen: Screens::GithubLink },
    ];

    let mut setting_toggles: Vec<SettingData> = vec![
        SettingData::new("Show Flags".to_string(), &SHOW_FLAG_IMAGES, None),
        SettingData::new("Fancy Cursor".to_string(), &FANCY_CURSOR, Some(|on, d|{if on{d.hide_cursor();}else{d.show_cursor();}})),
        SettingData::new("Scrolling BG".to_string(), &SCROLLING_BACKGROUND, None),
        SettingData::new("Show FPS".to_string(), &SHOW_FPS, None),
        SettingData::new("Center Text".to_string(), &CENTER_TEXT, None),
        SettingData::new("24H Clock".to_string(), &MILITARY_TIME, None),
        SettingData::new("Auto-fetch rooms".to_string(), &AUTO_FETCH, None),
    ];

    let mut rooms_fetch_error: String = "".to_string();
    let mut rooms_list: Vec<Room> = vec![];
    let mut rooms_fetching = false;
    let mut rooms_fetched = false;
    let mut rooms_fetched_once = false;
    let mut scroll_offset: usize = 0;
    let rooms_per_page: usize = 24;
    let scroll_amount: usize = 3;

    let mut current_screen: Screens = Screens::ServerList;
    let mut amount_of_dots_in_loading_text: f32 = 0.;
    let text_widths = HashMap::from([
        ("list", rl.measure_text(&"W".repeat(48), layout::FONT_SIZE)),
        ("clck", rl.measure_text(&"00:00:00", layout::FONT_SIZE)),
        ("erro", rl.measure_text(&"Error!", layout::FONT_SIZE)),
    ]);

    let bpm = 164.;
    let mut bg_scroll: f32 = 0.0;
    let mut bg_scroll_speed: f32;
    bg_scroll_speed = 64.;
    
    let (tx, mut rx) = mpsc::unbounded_channel::<Result<Vec<Room>, String>>();

    let mut time_timer: f32 = 0.;
    let mut time_txt:String = Local::now().format("%H:%M:%S").to_string();
    
    let ctx = ClipboardContext::new().unwrap();
    let mut error_box_showing = false;
    let mut error_box_msg: String = "".to_string();
    
    // easter egg wOW!
    let r: i32 = rl.get_random_value(0..100);
    let yameroing_it = r == 100;
    if yameroing_it {
        bg_scroll_speed = bpm;
    }

    let aud: RaylibAudio = RaylibAudio::init_audio_device().unwrap();
    
    let overbyte = include_bytes!("./res/aud/yamero.wav");
    let overwave = aud.new_wave_from_memory(".wav", overbyte)?;
    let over_snd= aud.new_sound_from_wave(&overwave).unwrap();
    
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&rt);
        let fps = format!("{}", d.get_fps());
        let screen_width = d.get_screen_width();
        let screen_height = d.get_screen_height();
        let dt = d.get_frame_time();

        if !over_snd.is_playing() && yameroing_it {over_snd.play();}
        d.draw_texture_rec(&checkerboard_bg,rrect(-bg_scroll, bg_scroll, screen_width as f32, screen_height as f32), Vector2::zero(), Color::WHITE);
        if cfg_val!(atomget SCROLLING_BACKGROUND) || yameroing_it  {
            bg_scroll = (bg_scroll - bg_scroll_speed * dt) % checkerboard_bg.width as f32;
            if yameroing_it {
                if bg_scroll_speed > 0. {
                    bg_scroll_speed -= (bpm * dt) * (bpm / 60.);
                } else {
                    bg_scroll_speed  = bpm;
                }
            }
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
        
        time_timer += dt;

        if time_timer > 0.5 {
            let mut fmt = "%I:%M:%S";
            if cfg_val!(atomget MILITARY_TIME) {
                fmt = "%H:%M:%S";
            }
            time_txt = Local::now().format(fmt).to_string()
        }

        d.draw_text(&time_txt, (screen_width / 2) - (text_widths.get("clck").unwrap() / 2), layout::BUTTON_HEIGHT as i32 - layout::FONT_SIZE, layout::FONT_SIZE, clr_val!(PRIMARY_COLOR));

        // we can actually start drawing things now

        for (i, btn) in navbar_buttons.iter().enumerate() {
            let rect = rrect(
                    i as f32 * layout::BUTTON_HEIGHT as f32,
                    0,
                    layout::BUTTON_HEIGHT,
                    layout::BUTTON_HEIGHT,
                );
            if d.gui_button(rect, "") {
                match btn.screen {
                    Screens::GithubLink => { open_url("https://github.com/stuxvii/rayball"); }
                    _ => current_screen = btn.screen,
                }
            }
            if let Some(txt) = ICONS_SPRITESHEET.get() {
                let mut txt_cntr = Vector2 { x: rect.x, y: rect.y };
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

        match current_screen {
            Screens::ServerList => {
                if d.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && d.is_key_pressed(KeyboardKey::KEY_V) {
                    let code: String = ctx.get_text()
                        .and_then(|t| Ok(url::Url::parse(&t).map(|url| url.to_string())?))
                        .unwrap_or_else(|e| e.to_string());

                    if code.len() != 11 { // every code should be 11 chars long. if it isn't chances are it's an error
                        error_box_showing = true;
                        error_box_msg = code.clone();
                    }
                }

                let wheel = d.get_mouse_wheel_move();
                if wheel < 0.0 {
                    scroll_offset =
                        (scroll_offset + scroll_amount).min(rooms_list.len() - rooms_per_page);
                } else if wheel > 0.0 {
                    scroll_offset = scroll_offset.saturating_sub(scroll_amount);
                }

                if !(!cfg_val!(atomget AUTO_FETCH) && !rooms_fetched_once) && !rooms_fetching {
                    rooms_fetched_once = true;
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
                        clr_val!(PRIMARY_COLOR),
                    );
                    if let Some(txt) = ICONS_SPRITESHEET.get() {
                        d.draw_texture_rec(txt, offline_rec, Vector2 {x: 0. + layout::SPACING, y: screen_height as f32 - offline_rec.height - layout::SPACING}, raylib::color::Color::WHITE);
                    }
                }

                if !rooms_fetching && d.is_key_pressed(KeyboardKey::KEY_R) {
                    rooms_list = vec![];
                    rooms_fetching = false;
                    rooms_fetched = false;
                    rooms_fetched_once = true;
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
                } else if rooms_fetched_once {
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
                        clr_val!(PRIMARY_COLOR),
                    );
                }
            }
            Screens::Configuration => {
                let setting_toggles_len = setting_toggles.len() as f32;
                for (display_index, btn) in setting_toggles.iter_mut().enumerate() {
                    let mut rect = rrect(
                        screen_width / 2,
                        layout::BUTTON_HEIGHT * display_index as f32,
                        layout::FONT_SIZE,
                        layout::FONT_SIZE,
                    );
                    rect.y += (screen_height/2) as f32;
                    rect.y -= (layout::BUTTON_HEIGHT * setting_toggles_len) / 2.;
                    rect.x -= 40.;

                    let old_val = btn.target.load(std::sync::atomic::Ordering::Relaxed);
                    let mut current_val: bool = old_val;
                    if d.gui_check_box(rect, &btn.text, &mut current_val) {
                        btn.target.store(current_val, std::sync::atomic::Ordering::Relaxed);
                        if let Some(act) = btn.callback {
                            act(current_val, &mut d);
                        }
                    }
                }
            }
            _ => (),
        }

        if error_box_showing {
            let error_rect = rrect(
                screen_width / 2 - 100, 
                screen_height / 2 - 45, 
                200, 
                100
            );

            let title = String::from("error");
            let msg = String::from(error_box_msg.clone());
            let btns = String::from("ok sorry");
            d.gui_unlock();
            let result = d.gui_message_box(error_rect, &title, &msg, &btns);
            d.gui_lock();
            if result >= 0 {
                error_box_showing = false;
                error_box_msg = "".to_string();
            }
        }

        if cfg_val!(atomget FANCY_CURSOR) {
            trail.draw(dt, &mut d);
        }

        if cfg_val!(atomget SHOW_FPS) {
            d.draw_text(&fps, layout::SPACING as i32, screen_height-layout::FONT_SIZE+layout::SPACING as i32, layout::FONT_SIZE, clr_val!(PRIMARY_COLOR));
        }

        if time_timer > 1. {
            time_timer = 0.;
        }
    }

    save_config();
    Ok(())
}
