use async_trait::async_trait;
use ezsockets::ClientConfig;
use tokio::time::timeout;
use std::{collections::HashMap, error::Error, time::Duration};
use url::Url;

use crate::net::xcoder::Encoder;

pub fn parse_code(clip: String) -> Result<std::string::String, Box<dyn Error>> {
    let parsed_url: Url = Url::parse(&clip)?;

    let query_map: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    if let Some(c) = query_map.get("c") {
        Ok(c.to_string())
    } else {
        Err("Didn't find code in URL".into())
    }
}

pub struct Client {
    handle: ezsockets::Client<Self>,
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Vec<u8>;

    async fn on_text(&mut self, text: ezsockets::Utf8Bytes) -> Result<(), ezsockets::Error> {
        println!("received message: {text}");
        Ok(())
    }

    async fn on_binary(&mut self, bytes: ezsockets::Bytes) -> Result<(), ezsockets::Error> {
        println!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        let data: Vec<u8> = call;
        dbg!(data);
        Ok(())
    }

    async fn on_connect(
        &mut self
    ) -> Result<(), ezsockets::Error> {
        println!("connected!");

        let mut plr_state = Encoder::new(None, None);
        plr_state.write_str_len_leb128(&cfg_val!(USERNAME));
        plr_state.write_str_len_leb128(&cfg_val!(COUNTRY));
        plr_state.write_str_len_leb128(&cfg_val!(AVATAR));
        let data: Vec<u8> = plr_state.as_slice();
        self.handle.binary(data)?;
        
        Ok(())
    }

    async fn on_close(&mut self, _frame: Option<ezsockets::CloseFrame>) -> Result<ezsockets::client::ClientCloseMode, ezsockets::Error> {
        println!("FUCK! CLOSED!");
        Ok(ezsockets::client::ClientCloseMode::Reconnect)
    }
}

pub async fn request_room_join(code: String) -> Result<ezsockets::Client<Client>, String> {
    let rq: ClientConfig =
        ClientConfig::new(format!("wss://p2p.haxball.com/client?id={}", code).as_str())
            .header(
                "User-agent",
                "Mozilla/5.0 (rust; raylib; rv:*) rayball client",
            )
            .header("Origin", "https://www.haxball.com");

    let connect_future = ezsockets::connect(|h| Client {handle:h}, rq);

    match timeout(Duration::from_secs(5), connect_future).await {
        Ok((handle, keep_alive)) => {
            tokio::spawn(async move {
                if let Err(e) = keep_alive.await {
                    eprintln!("Connection closed: {e}");
                }
            });
            Ok(handle)
        }
        Err(_) => Err("Connection timed out!".into()),
    }
}
