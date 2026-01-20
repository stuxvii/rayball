use chrono::Local;
use raylib::prelude::*;
use serde_json::Value;
use std::{collections::{BTreeMap}, fs, sync::{OnceLock, atomic::AtomicBool}};
use crate::cfg::config::*;

pub static ICONS_SPRITESHEET: OnceLock<Texture2D> = OnceLock::new();
pub static FLAGS_SPRITESHEET: OnceLock<Texture2D> = OnceLock::new();

/// Maximum length of the player name string.
pub const MAX_NAME_LENGTH: usize = 25;

pub fn load_spritesheets(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), Box<dyn std::error::Error>> {
    let icons_sp_file = include_bytes!("./res/img/icons.png");
    let icons_im_data = Image::load_image_from_mem(".png", icons_sp_file)?;
    let flags_sp_file = include_bytes!("./res/img/flags.png");
    let flags_im_data = Image::load_image_from_mem(".png", flags_sp_file)?;

    let _ = ICONS_SPRITESHEET
        .set(rl.load_texture_from_image(thread, &icons_im_data).unwrap())
        .or(Err("Failed to set ICONS_SPRITESHEET"));
    let _ = FLAGS_SPRITESHEET
        .set(rl.load_texture_from_image(thread, &flags_im_data).unwrap())
        .or(Err("Failed to set FLAGS_SPRITESHEET"));

    Ok(())
}


#[macro_export]
macro_rules! cfg_val {
    ($field:ident) => { *crate::cfg::config::$field.lock().unwrap() };
    (atomget $field:ident) => { 
        crate::cfg::config::$field.load(std::sync::atomic::Ordering::Relaxed) 
    };
}

fn atomset(a: &AtomicBool, y: bool) {
    a.store(y, std::sync::atomic::Ordering::Relaxed);
}

pub fn get_gui_color(style_property: i32) -> Color {
    let r = ((style_property >> 24) & 0xFF) as u8;
    let g = ((style_property >> 16) & 0xFF) as u8;
    let b = ((style_property >> 8) & 0xFF) as u8;
    let a = (style_property & 0xFF) as u8;

    Color::new(r, g, b, a)
}

pub fn generate_checkerboard(rl: &mut RaylibHandle, rt: &RaylibThread) -> Texture2D {
    let image_size = 64;
    let mut image = Image::gen_image_color(image_size, image_size, Color::RED);

    let image_size_f: f32 = image_size as f32;

    let bg_color1 = rl.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BASE_COLOR_NORMAL);
    let bg_color2 = rl.gui_get_style(GuiControl::DEFAULT, GuiControlProperty::BORDER_COLOR_NORMAL);
    
    image.draw_line_ex(
        Vector2 {
            x: image_size_f,
            y: 0.,
        },
        Vector2 {
            x: 0.,
            y: image_size_f,
        },
        image_size / 2,
        get_gui_color(bg_color2),
    );
    image.draw_line_ex(
        Vector2 { x: 0., y: 0. },
        Vector2 {
            x: image_size_f,
            y: image_size_f,
        },
        image_size,
        get_gui_color(bg_color1),
    );
    image.draw_line_ex(
        Vector2 { x: 0., y: 0. },
        Vector2 {
            x: image_size_f,
            y: image_size_f,
        },
        (image_size_f / 2.2) as i32,
        get_gui_color(bg_color2),
    );

    rl.load_texture_from_image(rt, &image).unwrap()
}

#[macro_export]
macro_rules! clr_val {
    ($field:ident) => { *crate::cfg::style::$field.lock().unwrap() };
}

pub fn save_config() {
    let mut map = BTreeMap::new();

    map.insert("scrolling_bg".to_string(), Value::from(cfg_val!(atomget SCROLLING_BACKGROUND)));
    map.insert("fancy_cursor".to_string(), Value::from(cfg_val!(atomget FANCY_CURSOR)));
    map.insert("show_fps".to_string(), Value::from(cfg_val!(atomget SHOW_FPS)));
    map.insert("military_time".to_string(), Value::from(cfg_val!(atomget MILITARY_TIME)));
    map.insert("auto_fetch".to_string(), Value::from(cfg_val!(atomget AUTO_FETCH)));
    map.insert("skip_title".to_string(), Value::from(cfg_val!(atomget SKIP_TITLE)));
    
    map.insert("username".to_string(), Value::from(cfg_val!(USERNAME).as_str()));
    map.insert("country".to_string(), Value::from(cfg_val!(COUNTRY).as_str()));
    map.insert("longitude".to_string(), Value::from(cfg_val!(LONGITUDE) as f64));
    map.insert("latitude".to_string(), Value::from(cfg_val!(LATITUDE) as f64));
    map.insert("fps".to_string(), Value::from(cfg_val!(FPS) as f64));

    if let Ok(json_string) = serde_json::to_string_pretty(&map) {
        let _ = std::fs::write("./rb.cfg", json_string);
    }
}

pub fn load_settings() -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("rb.cfg")?;
    let json: Value = serde_json::from_str(&contents)?;

    let map = json.as_object().ok_or("Config is not a JSON object")?;

    let get_bool = |key: &str| -> Option<bool> {
        map.get(key)?.as_bool()
    };

    let get_str = |key: &str| -> Option<String> {
        map.get(key)?.as_str().map(|s| s.to_owned())
    };

    let get_num = |key: &str| -> Option<f64> {
        map.get(key)?.as_f64()
    };
    
    if let Some(v) = get_bool("scrolling_bg")  {atomset(&SCROLLING_BACKGROUND, v);}
    if let Some(v) = get_bool("fancy_cursor")  {atomset(&FANCY_CURSOR, v);}
    if let Some(v) = get_bool("show_fps")      {atomset(&SHOW_FPS, v);}
    if let Some(v) = get_bool("military_time") {atomset(&MILITARY_TIME, v);}
    if let Some(v) = get_bool("auto_fetch")    {atomset(&AUTO_FETCH, v);}
    if let Some(v) = get_bool("skip_title")    {atomset(&SKIP_TITLE, v);}
    if let Some(v) = get_str("username")     {cfg_val!(USERNAME) = v; }
    if let Some(v) = get_str("country")      {cfg_val!(COUNTRY) = v; }
    if let Some(v) = get_num("longitude")       {cfg_val!(LONGITUDE) = v as f32; }
    if let Some(v) = get_num("latitude")        {cfg_val!(LATITUDE) = v as f32; }
    if let Some(v) = get_num("fps")             {cfg_val!(FPS) = v as u32; }

    cfg_val!(USERNAME).truncate(24);
    cfg_val!(COUNTRY).truncate(2);

    Ok(())
}

pub mod cfg {
    /// Layout constants.
    pub mod layout {
        use raylib::math::Vector2;

        pub static FONT_SIZE: i32 = 10;
        pub static SPACING: f32 = 1.;
        pub static BUTTON_HEIGHT: f32 = 13.;
        pub static FLAG_SIZE: Vector2 = Vector2 { x: 19.0, y: 11.0 };
        pub static PLAYER_COUNT_WIDTH: i32 = 33;
        pub static DISTANCE_WIDTH: i32 = 48;
    }

    /// Settings that can be toggled by the user.
    pub mod config {
        use std::sync::{LazyLock, Mutex, atomic::AtomicBool};

        pub static FANCY_CURSOR: AtomicBool = AtomicBool::new(true);
        pub static SCROLLING_BACKGROUND: AtomicBool = AtomicBool::new(true);
        pub static SHOW_FPS: AtomicBool = AtomicBool::new(false);
        pub static MILITARY_TIME: AtomicBool = AtomicBool::new(true);
        pub static AUTO_FETCH: AtomicBool = AtomicBool::new(true);
        pub static SKIP_TITLE: AtomicBool = AtomicBool::new(false);
        pub static FPS: Mutex<u32> = Mutex::new(60);
        pub static LATITUDE: Mutex<f32> = Mutex::new(-32.5323);
        pub static LONGITUDE: Mutex<f32> = Mutex::new(-68.5040);
        pub static USERNAME: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("".to_string()));
        pub static COUNTRY: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("ar".to_string()));
    }

    pub mod style {
        use std::sync::{LazyLock, Mutex};
        use raylib::color::Color;

        pub static ENABLED_COLOR:  LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("00AA00").expect("Invalid hex")));
        pub static PRIMARY_COLOR:  LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("f2ffee").expect("Invalid hex")));
        pub static SECONDARY_COLOR:LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("000000").expect("Invalid hex")));
        pub static TERNARY_COLOR:  LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("BABABA").expect("Invalid hex")));
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProgramState {
    Menu = 0,
    Joining = 1,
    InGame = 2,
    AskInfo = 3,
}

impl From<usize> for ProgramState {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Menu,
            1 => Self::Joining,
            2 => Self::InGame,
            3 => Self::AskInfo,
            _ => Self::Menu
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Screens {
    ServerList = 0,
    Configuration = 1,
    GithubLink = 2,
}

impl From<usize> for Screens {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::ServerList,
            1 => Self::Configuration,
            _ => Self::ServerList
        }
    }
}

pub struct Alert { 
    pub text: String,
    pub fade: bool,
    pub creation: i64 
}
impl Alert {
    pub fn new(text: String, fade: bool) -> Alert {
        Alert {
            text,
            fade,
            creation: Local::now().timestamp()
        }
    }
}
pub mod ui {
    pub mod cursor;
    pub mod primitives;
    pub mod title;
    pub mod state;
    pub mod flags;
    pub mod menu;
    pub mod joining;
}

pub mod net {
    pub mod rooms;
    pub mod join;
}
