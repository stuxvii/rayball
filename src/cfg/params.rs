use std::sync::RwLock;

/// Maximum length of the player name string.
pub const MAX_NAME_LENGTH: usize = 25;

/// Settings that can be toggled by the user.
pub struct Config {
    pub show_flag_images: bool,
    pub fancy_cursor: bool,
    pub scrolling_background: bool,
    pub hide_locked: bool,
    pub center_text: bool,
    pub fps: u32,
    pub max_fps: u32,
    pub latitude: f32,
    pub longitude: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_flag_images: true,
            fancy_cursor: true,
            scrolling_background: true,
            hide_locked: false,
            center_text: false,
            fps: 240,
            max_fps: 240,
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

pub static SETTINGS: RwLock<Config> = RwLock::new(Config {
    show_flag_images: true,
    fancy_cursor: true,
    scrolling_background: true,
    hide_locked: false,
    center_text: false,
    fps: 7000,
    max_fps: 240,
    latitude: -34.3559,
    longitude: -58.2255,
});

pub enum Settings {
    ShowFlags = 0,
    UseFancyCursor = 1,
    ScrollingBG = 2,
    HideLocked = 3,
}

pub enum Screens {
    ServerList = 0,
    Configuration = 1,
}