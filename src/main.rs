use chrono::prelude::*;
use clipboard_rs::ClipboardContext;
use futures::task::noop_waker_ref;
use rayball_rs::cfg::config::*;
use rayball_rs::cfg::layout;
use rayball_rs::net::join::request_room_join;
use rayball_rs::net::xcoder::BinaryEncoder;
use rayball_rs::ui::cursor::CursorTrail;
use rayball_rs::ui::joining;
use rayball_rs::ui::{menu, title};
use rayball_rs::ui::primitives::{Room, SettingData};
use rayball_rs::ui::state::{AppState, NavIcon};
use rayball_rs::*;
use raylib::ease::Tween;
use raylib::error::Error;
use raylib::prelude::*;
use std::collections::HashMap;
use std::task::Context;
use std::vec;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    tracing_subscriber::fmt::init();

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

    let args: Vec<String> = std::env::args().collect();
    
    if let Some(code) = args.get(1) {
        match request_room_join(code.to_string()).await {
            Ok(_) => {
                println!("Waiting for messages (Ctrl+C to exit)...");
                tokio::signal::ctrl_c().await.unwrap();
                println!("Exiting...");
            },
            Err(e) => println!("Connection failed: {e}"),
        }
        return Ok(());
    }

    let program_name = "RayBall";
    let (mut rl, rt) = raylib::init()
        .resizable()
        .title(program_name)
        .size(640, 480)
        .build();

    rl.gui_load_style("./style.rgs");
    rl.set_target_fps(cfg_val!(FPS));
    rl.set_window_min_size(490, 360);

    clr_val!(SECONDARY_COLOR) =
        get_gui_color(rl.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BASE_COLOR_NORMAL));
    clr_val!(PRIMARY_COLOR) =
        get_gui_color(rl.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::TEXT_COLOR_NORMAL));
    clr_val!(TERNARY_COLOR) = get_gui_color(rl.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BORDER_COLOR_NORMAL),
    );

    match load_spritesheets(&mut rl, &rt) {
        Ok(_) => {
            println!("Loaded textures.")
        }
        Err(e) => {
            panic!("Epic fail loading textures: {}.", e)
        }
    };

    let checkerboard_bg: Texture2D = generate_checkerboard(&mut rl, &rt);

    let mut trail: CursorTrail = CursorTrail::new();
    if cfg_val!(atomget FANCY_CURSOR) {
        rl.hide_cursor();
    }

    let waker = noop_waker_ref();
    let cx: Context<'_> = Context::from_waker(waker);

    let mut bg_scroll: f32 = 0.0;
    let bg_scroll_speed: f32 = 32.;

    let (tx, rx) = mpsc::unbounded_channel::<Result<Vec<Room>, String>>();

    let mut time_timer: f32 = 0.;
    let mut time_txt: String = Local::now().format("%H:%M:%S").to_string();

    let mut state: AppState = AppState {
        navbar_buttons: vec![
            NavIcon { rect: rrect(0.0, 0.0, 11.0, 11.0), screen: Screens::ServerList },
            NavIcon { rect: rrect(11.0, 0.0, 11.0, 11.0), screen: Screens::Configuration },
            NavIcon { rect: rrect(44.0, 0.0, 11.0, 11.0), screen: Screens::GithubLink },
        ],
        setting_toggles: vec![
            SettingData::new("Fancy Cursor".to_string(), &FANCY_CURSOR, Some(|on, d| { if on { d.hide_cursor(); } else { d.show_cursor(); } })),
            SettingData::new("Scrolling BG".to_string(), &SCROLLING_BACKGROUND, None),
            SettingData::new("Show FPS".to_string(), &SHOW_FPS, None),
            SettingData::new("24H Clock".to_string(), &MILITARY_TIME, None),
            SettingData::new("Auto-fetch rooms".to_string(), &AUTO_FETCH, None),
            SettingData::new("Skip title".to_string(), &SKIP_TITLE, None),
        ],
        errors: vec![],
        rooms_fetch_error: "".to_string(),
        rooms_list: vec![],
        rooms_fetching: false,
        rooms_fetched: false,
        rooms_fetched_once: false,
        scroll_offset: 0,
        rooms_per_page: 24,
        scroll_amount: 3,
        current_screen: Screens::ServerList,
        amount_of_dots_in_loading_text: 0.,
        text_widths: HashMap::from([
            ("list", rl.measure_text(&"W".repeat(48), layout::FONT_SIZE)),
            ("clck", rl.measure_text("00:00:00", layout::FONT_SIZE)),
            ("erro", rl.measure_text("Error!", layout::FONT_SIZE)),
            ("usnm", rl.measure_text(&"W".repeat(25), layout::FONT_SIZE)),
        ]),
        tx,
        rx,
        cx,
        ws_client: None,
        join_task: None,
        clipboard_ctx: ClipboardContext::new().unwrap(),
        program_state: if cfg_val!(atomget SKIP_TITLE) { ProgramState::Menu } else { ProgramState::AskInfo },
        state: BinaryEncoder::new(256, true),
        logo_letter_amp_timer: 0.,
        logo_letter_amp_tween: Tween::new(ease::circ_out, 32., 4., 200.),
    };

    while !rl.window_should_close() {
        let mut d: RaylibDrawHandle<'_> = rl.begin_drawing(&rt);
        let fps: String = format!("{}", d.get_fps());
        let screen_width: i32 = d.get_screen_width();
        let screen_height: i32 = d.get_screen_height();
        let dt: f32 = d.get_frame_time();

        d.draw_texture_rec(
            &checkerboard_bg,
            rrect(
                -bg_scroll,
                bg_scroll,
                screen_width as f32,
                screen_height as f32,
            ),
            Vector2::zero(),
            Color::WHITE,
        );

        if cfg_val!(atomget SCROLLING_BACKGROUND) {
            bg_scroll = (bg_scroll - bg_scroll_speed * dt) % checkerboard_bg.width as f32;
        }

        time_timer += dt;

        match state.program_state {
            ProgramState::Menu => {
                d.draw_rectangle(0, 0, screen_width, layout::BUTTON_HEIGHT as i32, clr_val!(SECONDARY_COLOR));
                menu::draw_menu(&mut d, &mut state, screen_width, screen_height, dt);
            }
            ProgramState::Joining => {
                joining::draw_joining(&mut d, &mut state, screen_width, screen_height);
            }
            ProgramState::AskInfo => {
                title::draw_ask_info(&mut d, &mut state, program_name, screen_width, screen_height, dt);
            }
            _ => (),
        }
        if state.errors.is_empty() {
            for i in (0..state.errors.len()).rev() {
                let er_box = &mut state.errors[i];
                let idx = i as i32;

                let text = if er_box.fade {
                    format!(
                        "error: {} | closes in {}",
                        er_box.text,
                        (er_box.creation + 5) - Local::now().timestamp()
                    )
                } else {
                    format!("error: {} | click to close", er_box.text)
                };

                let mut error_rect = rrect(
                    screen_width - d.measure_text(text.as_str(), layout::BUTTON_HEIGHT as i32),
                    idx * 32,
                    d.measure_text(text.as_str(), layout::BUTTON_HEIGHT as i32),
                    32,
                );
                error_rect.y += layout::BUTTON_HEIGHT;
                let mut result = d.gui_button(error_rect, text.as_str());

                if !result && er_box.fade {
                    result = Local::now().timestamp() > (er_box.creation + 5);
                }

                if result {
                    state.errors.remove(i);
                }
            }
        }

        if cfg_val!(atomget FANCY_CURSOR) {
            trail.draw(dt, &mut d);
        }

        if cfg_val!(atomget SHOW_FPS) {
            d.draw_text(
                &fps,
                layout::SPACING as i32,
                screen_height - layout::FONT_SIZE + layout::SPACING as i32,
                layout::FONT_SIZE,
                clr_val!(PRIMARY_COLOR),
            );
        }

        if time_timer > 1. {
            let mut fmt = "%I:%M:%S";
            if cfg_val!(atomget MILITARY_TIME) {
                fmt = "%H:%M:%S";
            }
            time_txt = Local::now().format(fmt).to_string();
            time_timer = 0.;
        }
    }

    save_config();
    Ok(())
}
