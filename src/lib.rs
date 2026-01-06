use raylib::prelude::*;
use tinyjson::JsonValue;
use std::{collections::HashMap, fs, sync::OnceLock};

pub static ICONS_SPRITESHEET: OnceLock<Texture2D> = OnceLock::new();
pub static FLAGS_SPRITESHEET: OnceLock<Texture2D> = OnceLock::new();

pub fn load_spritesheets(rl: &mut RaylibHandle, thread: &RaylibThread) {
    ICONS_SPRITESHEET
        .set(rl.load_texture(thread, "src/res/img/icons.png").unwrap())
        .expect("Failed to set ICONS_SPRITESHEET");
    FLAGS_SPRITESHEET
        .set(rl.load_texture(thread, "src/res/img/flags.png").unwrap())
        .expect("Failed to set FLAGS_SPRITESHEET");
}

pub fn load_settings() -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string("rb.conf")?;
    let json: JsonValue = contents.parse()?;
    let object: &HashMap<String, JsonValue> = json.get().ok_or("Couldn't get JSON values? whatever just do some defaults")?;
    println!("{:?}", object);
    Ok(())
}

pub mod cfg {
    pub mod params;
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

    pub mod style {
        use raylib::color::Color;

        pub static PRIMARY_COLOR: Color = Color::RAYWHITE;
        pub static SECONDARY_COLOR: Color = Color::BLACK;
        pub static HOVER_COLOR: Color = Color::GRAY;
        pub static GREEN_COLOR: Color = Color::GREEN;
        pub static BG_COLOR1: Color = Color::new(0, 32, 0, 255);
        pub static BG_COLOR2: Color = Color::new(0, 64, 0, 255);
    }

    pub mod flags;
}

pub mod ui {
    pub mod cursor;
    pub mod primitives;
}

pub mod net {
    pub mod rooms;
}
