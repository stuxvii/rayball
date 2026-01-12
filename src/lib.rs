use raylib::prelude::*;
use serde_json::Value;
use std::{collections::{BTreeMap}, fs, sync::{OnceLock, atomic::AtomicBool}};
use crate::cfg::config::{self, *};

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
    ($field:ident) => { *config::$field.lock().unwrap() };
    (atomget $field:ident) => { 
        config::$field.load(std::sync::atomic::Ordering::Relaxed) 
    };
}

#[macro_export]
macro_rules! clr_val {
    ($field:ident) => { *style::$field.lock().unwrap() };
}

fn atomset(a: &AtomicBool, y: bool) {
    a.store(y, std::sync::atomic::Ordering::Relaxed);
}

pub fn save_config() {
    let mut map = BTreeMap::new();

    map.insert("scrolling_bg".to_string(), Value::from(cfg_val!(atomget SCROLLING_BACKGROUND)));
    map.insert("show_flags".to_string(), Value::from(cfg_val!(atomget SHOW_FLAG_IMAGES)));
    map.insert("fancy_cursor".to_string(), Value::from(cfg_val!(atomget FANCY_CURSOR)));
    map.insert("center_text".to_string(), Value::from(cfg_val!(atomget CENTER_TEXT)));
    map.insert("show_fps".to_string(), Value::from(cfg_val!(atomget SHOW_FPS)));
    map.insert("military_time".to_string(), Value::from(cfg_val!(atomget MILITARY_TIME)));
    
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

    let get_num = |key: &str| -> Option<f64> {
        map.get(key)?.as_f64()
    };
    
    if let Some(v) = get_bool("scrolling_bg")  {atomset(&SCROLLING_BACKGROUND, v);}
    if let Some(v) = get_bool("show_flags")    {atomset(&SHOW_FLAG_IMAGES, v);}
    if let Some(v) = get_bool("fancy_cursor")  {atomset(&FANCY_CURSOR, v);}
    if let Some(v) = get_bool("center_text")   {atomset(&CENTER_TEXT, v);}
    if let Some(v) = get_bool("show_fps")      {atomset(&SHOW_FPS, v);}
    if let Some(v) = get_bool("military_time") {atomset(&MILITARY_TIME, v);}
    if let Some(v) = get_num("longitude")       {cfg_val!(LONGITUDE) = v as f32; }
    if let Some(v) = get_num("latitude")        {cfg_val!(LATITUDE) = v as f32; }
    if let Some(v) = get_num("fps")             {cfg_val!(FPS) = v as u32; }

    Ok(())
}

pub mod flags;
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

    pub mod config {
        use std::sync::{Mutex, atomic::AtomicBool};

        pub static SHOW_FLAG_IMAGES: AtomicBool = AtomicBool::new(true);
        pub static FANCY_CURSOR: AtomicBool = AtomicBool::new(true);
        pub static SCROLLING_BACKGROUND: AtomicBool = AtomicBool::new(true);
        pub static CENTER_TEXT: AtomicBool = AtomicBool::new(false);
        pub static SHOW_FPS: AtomicBool = AtomicBool::new(false);
        pub static MILITARY_TIME: AtomicBool = AtomicBool::new(true);
        pub static FPS: Mutex<u32> = Mutex::new(24);
        pub static LATITUDE: Mutex<f32> = Mutex::new(-12.0336);
        pub static LONGITUDE: Mutex<f32> = Mutex::new(-77.0215);
    }

    pub mod style {
        use std::sync::{LazyLock, Mutex};
        use raylib::color::Color;

        pub static ENABLED_COLOR:  LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("00AA00").expect("Invalid hex")));
        pub static PRIMARY_COLOR:  LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("f2ffee").expect("Invalid hex")));
        pub static SECONDARY_COLOR:LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("000000").expect("Invalid hex")));
        pub static HOVER_COLOR:    LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("aaaaaa").expect("Invalid hex")));
        pub static BG_COLOR1:      LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("004400").expect("Invalid hex")));
        pub static BG_COLOR2:      LazyLock<Mutex<Color>> = LazyLock::new(|| Mutex::new(Color::from_hex("005500").expect("Invalid hex")));

        pub static BG_POLOR1: Color = Color::new(255, 160, 255, 255);
        pub static BG_POLOR2: Color = Color::new(220, 90, 200, 255);
    }
}

/// Settings that can be toggled by the user.

pub enum Settings {
    ShowFlags = 0,
    UseFancyCursor = 1,
    ScrollingBG = 2,
    ShowFPS = 3,
    CenterText = 4,
    MilitaryTime = 5,
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
            1 => Screens::Configuration,
            _ => Self::ServerList
        }
    }
}
pub mod ui {
    pub mod cursor;
    pub mod primitives;
}

pub mod net {
    pub mod rooms;
}
