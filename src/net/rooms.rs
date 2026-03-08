use haversine::{Location, distance};
use minreq::Response;
use raylib::prelude::*;

use crate::{
    net::xcoder::BinaryDecoder,
    ui::{flags, primitives::Room},
};

pub fn fetch_rooms(flag_texture: &Texture2D) -> Result<Vec<Room>, std::string::String> {
    let mut rooms = Vec::new();

    let fetch: Result<Response, minreq::Error> =
        minreq::get("http://haxball.com/rs/api/list").send();
    let response: Vec<u8>;
    match fetch {
        Ok(res) => {
            response = res.as_bytes().to_vec();
        }
        Err(e) => return Err(format!("{}", e)),
    };

    let mut decoder = BinaryDecoder::new(&response, true);
    let user_location = Vector2 {
        x: cfg_val!(LONGITUDE),
        y: cfg_val!(LATITUDE),
    };

    decoder.set_position(1);

    while decoder.position() + 1 < response.len() as u64 {
        if response[decoder.position() as usize] != 0x00 {
            break;
        }
        decoder.r_u8();

        let id_len = decoder.r_u8() as usize;
        let id = String::from_utf8_lossy(&decoder.read_bytes(id_len)).into_owned();
        decoder.r_u8();

        let block_len = decoder.r_u8() as usize;
        let next_entry_pos = decoder.position() + block_len as u64;

        decoder.set_position(decoder.position() + 2);

        let name_len = decoder.r_u8() as usize;
        let name = String::from_utf8_lossy(&decoder.read_bytes(name_len)).into_owned();

        let c_len = decoder.r_u8() as usize;
        let country = String::from_utf8_lossy(&decoder.read_bytes(c_len)).into_owned();

        let x = decoder.r_f32();
        let y = decoder.r_f32();

        let flag_coords = flags::get_vector_from_code(&country);
        let flags_rec = Rectangle::new(
            flag_texture.width as f32 - flag_coords.x,
            flag_texture.height as f32 - flag_coords.y,
            16.0,
            11.0,
        );

        let locked = decoder.r_u8() != 0;
        let max_players = decoder.r_u8();
        let players = decoder.r_u8();

        let distance_km = distance(
            Location {
                latitude: user_location.y as f64,
                longitude: user_location.x as f64,
            },
            Location {
                latitude: x as f64,
                longitude: y as f64,
            },
            haversine::Units::Kilometers,
        );

        rooms.push(Room::new(
            name,
            id,
            country,
            Vector2::new(players.into(), max_players.into()),
            flags_rec,
            format!("{}/{}", players, max_players),
            distance_km,
            format!("{}km", distance_km.round()),
            locked,
        ));

        decoder.set_position(next_entry_pos);
    }

    rooms.sort_by(|a, b| f64::total_cmp(&a.distance_km, &b.distance_km));

    Ok(rooms)
}
