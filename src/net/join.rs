use std::{collections::HashMap, error::Error};
use url::Url;
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite::client::IntoClientRequest};

pub fn parse_code(clip: String) -> Result<std::string::String, Box<dyn Error>> {
    let parsed_url_result = Url::parse(&clip);
    let parsed_url: Url;
    match parsed_url_result {
        Ok(u) => {
            parsed_url = u;
        },
        Err(e) => {
            return Err(e.into())
        }
    };
    let query_map: HashMap<String, String> = parsed_url
        .query_pairs()
        .into_owned()
        .collect();
    if let Some(c) = query_map.get("c") {
        return Ok(c.to_string());
    } else {
        return Err("Didn't find code in URL".into())
    }
}

pub async fn request_room_join(code: String) -> (WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>) {
    let mut rq = format!("wss://p2p.haxball.com/client?id={}", code).as_str().into_client_request().unwrap();
    // custom headers so that the server lets us go through :pray:
    rq.headers_mut().insert("Origin", "https://www.haxball.com".parse().unwrap());
    // Hey basro. Or mario Carbajal. Feel free to block this specific user agent or whatever if that fits you. No biggie.
    rq.headers_mut().insert("User-agent", "Mozilla/5.0 (raygui; raylib; rv:*) AcidBox's Rust H*xball Client That Uses Mother Fuckig Raylib".parse().unwrap());
    connect_async(rq).await.unwrap()
}