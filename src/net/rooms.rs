use haversine::{Location, distance};
use raylib::prelude::*;

use crate::{cfg::{self, params::SETTINGS}, ui::primitives::Room};

pub fn fetch_rooms(
    flag_texture: &Texture2D,
) -> Vec<Room> {
    let mut rooms = Vec::new();
    
    let response = match minreq::get("http://haxball.com/rs/api/list").send() {
        Ok(res) => res.as_bytes().to_vec(),
        Err(e) => {
            println!("Couldn't form a connection to HaxBall's servers! Error: {}", e);

            let empty_room: Room = Room::new(
                "You're offline!".to_string(),
                "user_offline".to_string(),
                "na".to_string(),
                Vector2::new(0.0, 0.0),
                Rectangle::new(0.0, 0.0, 0.0, 0.0),
                "".to_string(),
                -1.0,
                "".to_string(),
                true,
            );
            rooms.push(empty_room);
            return rooms;
        }
    };

    let mut pos = 1;
    let buf = response;

    while pos + 1 < buf.len() {
        if buf[pos] != 0x00 {
            break;
        }
        pos += 1;

        let id_len = buf[pos] as usize;
        pos += 1;
        let id = String::from_utf8_lossy(&buf[pos..pos + id_len]).into_owned();
        pos += id_len + 1;

        let block_len = buf[pos] as usize;
        pos += 1;
        let next_entry = pos + block_len;

        pos += 2;

        let name_len = buf[pos] as usize;
        pos += 1;
        let name = String::from_utf8_lossy(&buf[pos..pos + name_len]).into_owned();
        pos += name_len;

        let c_len = buf[pos] as usize;
        pos += 1;
        let country = String::from_utf8_lossy(&buf[pos..pos + c_len]).into_owned();
        pos += c_len;

        let x = read_f32(&buf, pos);
        pos += 4;
        let y = read_f32(&buf, pos);
        pos += 4;

        let flag_coords = cfg::flags::get_vector_from_code(&country);

        let flags_rec = Rectangle::new(
            (flag_texture.width as f32) - flag_coords.x,
            (flag_texture.height as f32) - flag_coords.y,
            16.0,
            11.0,
        );

        let locked = buf[pos] != 0;
        pos += 1;
        let max_players = buf[pos];
        pos += 1;
        let players = buf[pos];

        let label = format!("{}/{}", players, max_players);

        let distance_km: f64 = distance(
            Location {
                latitude: SETTINGS.read().unwrap().latitude as f64,
                longitude: SETTINGS.read().unwrap().longitude as f64,
            },
            Location {
                latitude: x as f64,
                longitude: y as f64,
            },
            haversine::Units::Kilometers,
        );
        let distance_text = format!("{}km", distance_km.round());

        let room: Room = Room::new(
            name,
            id,
            country,
            Vector2::new(players.into(), max_players.into() ),
            flags_rec,
            label.to_string(),
            distance_km,
            distance_text,
            locked,
        );
        rooms.push(room);
        pos = next_entry;
    }

    rooms.sort_by(|a, b| a.distance_km.partial_cmp(&b.distance_km).unwrap());
    //rooms.reverse();
    rooms
}

fn read_f32(data: &[u8], pos: usize) -> f32 {
    if pos + 4 > data.len() {
        return 0.0;
    }
    let bytes = [data[pos], data[pos + 1], data[pos + 2], data[pos + 3]];
    f32::from_le_bytes(bytes)
}
