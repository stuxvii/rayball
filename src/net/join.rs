use crate::net::xcoder::{BinaryDecoder, BinaryEncoder};
use crate::ui::state::AppState;
use async_trait::async_trait;
use ezsockets::ClientConfig;
use flate2::write::DeflateEncoder;
use flate2::{Compression, read::DeflateDecoder};
use std::io::Read;
use std::{collections::HashMap, error::Error, io::Write, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};
use url::Url;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use webrtc::{
    api::{APIBuilder, media_engine::MediaEngine},
    ice_transport::{ice_candidate::RTCIceCandidateInit, ice_server::RTCIceServer},
    peer_connection::configuration::RTCConfiguration,
};

pub struct Client {
    handle: ezsockets::Client<Self>,
    state: AppState,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct IceCandidateJson {
    candidate: String,
    sdp_mid: String,
    sdp_m_line_index: u16,
    username_fragment: Option<String>,
}

fn decode_signal_packet(packet: &[u8]) -> (u8, Vec<u8>) {
    if packet.is_empty() {
        panic!("Packet is too short to contain an opcode");
    }

    let op_code = packet[0];
    let compressed_payload = &packet[1..];

    let mut decoder = DeflateDecoder::new(compressed_payload);
    let mut decompressed_payload = Vec::new();

    decoder
        .read_to_end(&mut decompressed_payload)
        .expect("Failed to decompress payload");

    (op_code, decompressed_payload)
}

fn compress_signal_packet(op_code: u8, payload_buffer: &[u8]) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(6));
    encoder.write_all(payload_buffer).unwrap();
    let compressed = encoder.finish().expect("Failed to compress");

    let mut packet = Vec::with_capacity(1 + compressed.len());
    packet.push(op_code);
    packet.extend_from_slice(&compressed);
    packet
}

async fn rtc_offer_shenanigans() -> Result<
    (
        Arc<webrtc::peer_connection::RTCPeerConnection>,
        Arc<Mutex<Vec<RTCIceCandidateInit>>>,
        Arc<webrtc::data_channel::RTCDataChannel>,
        Arc<webrtc::data_channel::RTCDataChannel>,
        Arc<webrtc::data_channel::RTCDataChannel>,
    ),
    Box<dyn Error + Send + std::marker::Sync>,
> {
    let config: RTCConfiguration = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let api: webrtc::api::API = APIBuilder::new()
        .with_media_engine(MediaEngine::default())
        .build();

    let peer_connection: Arc<webrtc::peer_connection::RTCPeerConnection> =
        Arc::new(api.new_peer_connection(config).await?);

    let candidates = Arc::new(Mutex::new(Vec::new()));
    let c_clone = Arc::clone(&candidates);

    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        let c_clone = Arc::clone(&c_clone);
        Box::pin(async move {
            if let Some(cand) = candidate {
                if let Ok(json) = cand.to_json() {
                    c_clone.lock().await.push(json);
                }
            }
        })
    }));

    // Register data channels
    let ro = peer_connection.create_data_channel("ro", None).await?;
    let ru = peer_connection
        .create_data_channel(
            "ru",
            Some(RTCDataChannelInit {
                ordered: Some(false),
                max_retransmits: Some(1),
                ..Default::default()
            }),
        )
        .await?;
    let uu = peer_connection
        .create_data_channel(
            "uu",
            Some(RTCDataChannelInit {
                ordered: Some(false),
                max_retransmits: Some(0),
                ..Default::default()
            }),
        )
        .await?;

    let offer = peer_connection.create_offer(None).await?;
    peer_connection.set_local_description(offer.clone()).await?;

    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    gather_complete.recv().await;

    Ok((peer_connection, candidates, ro, ru, uu))
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Vec<u8>;

    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
        let result: (
            Arc<webrtc::peer_connection::RTCPeerConnection>,
            Arc<Mutex<Vec<RTCIceCandidateInit>>>,
            Arc<webrtc::data_channel::RTCDataChannel>,
            Arc<webrtc::data_channel::RTCDataChannel>,
            Arc<webrtc::data_channel::RTCDataChannel>,
        ) = rtc_offer_shenanigans().await?;

        let (peer_connection, candidates) = (result.0, result.1);
        self.state.ro_datachannel = Some(result.2);
        self.state.ru_datachannel = Some(result.3);
        self.state.uu_datachannel = Some(result.4);
        self.state.peer_connection = Some(peer_connection);

        if let None = self.state.uu_datachannel {
            return Err("UU DataChannel was not initialized.".into());
        }
        if let None = self.state.ru_datachannel {
            return Err("RU DataChannel was not initialized.".into());
        }
        if let None = self.state.ro_datachannel {
            return Err("RO DataChannel was not initialized.".into());
        }

        let uu_datachanel = self.state.uu_datachannel.as_mut().unwrap();
        let ru_datachanel = self.state.ru_datachannel.as_mut().unwrap();
        let ro_datachanel = self.state.ro_datachannel.as_mut().unwrap();

        let dc_uu_label = uu_datachanel.label().to_owned();
        uu_datachanel.on_open(Box::new(move || {
            println!("Data channel {dc_uu_label} is now open!!!");
            Box::pin(async {})
        }));

        uu_datachanel.on_message(Box::new(move |msg| {
            let payload = String::from_utf8_lossy(&msg.data);
            println!("Received message: {}", payload);
            Box::pin(async {})
        }));

        let dc_ru_label = ru_datachanel.label().to_owned();
        ru_datachanel.on_open(Box::new(move || {
            println!("Data channel {dc_ru_label} is now open!!!");
            Box::pin(async {})
        }));

        ru_datachanel.on_message(Box::new(move |msg| {
            let payload = String::from_utf8_lossy(&msg.data);
            println!("Received message: {}", payload);
            Box::pin(async {})
        }));

        let dc_ro_label = ro_datachanel.label().to_owned();
        let dc_clone = Arc::clone(&ro_datachanel);
        ro_datachanel.on_open(Box::new(move || {
            println!("Data channel {dc_ro_label} is now open!");
            Box::pin(async move {})
        }));

        ro_datachanel.on_message(Box::new(move |msg| {
            let payload = String::from_utf8_lossy(&msg.data);
            let dc_clone = Arc::clone(&dc_clone);
            println!("Received message: {}", payload);
            Box::pin(async move {
                let mut user_info = BinaryEncoder::new(false);
                user_info.w_str(&cfg_val!(USERNAME));
                user_info.w_str(&cfg_val!(COUNTRY));
                user_info.w_nullable_str(Some(&cfg_val!(AVATAR)));

                let mut data = BinaryEncoder::new(false);
                data.w_u8(0);
                // Here, ideally i would write a byte that is the hex code indicating the full length of the idkey.x + signing challenge
                // let mut cryptography_data = BinaryEncoder::new(false);
                // cryptography_data.w_str(&cfg_val!(IDKEY).x)
                // cryptography_data.w_u8(crypto_challenge.bytes().len() as u8)
                // cryptography_data.append_bytes(crypto_challenge.bytes()) // append_byte doesnt manage specifying the length of the added data, must manage that ourselves
                // Then just write all that.
                // And write the length of the user_info along with the user info.
                // data.w_u8(user_info.bytes().len() as u8);
                // data.append_bytes(user_info.bytes());

                let data = bytes::Bytes::copy_from_slice(data.bytes());
                // println!("Sending player data through dc_{dc_ro_label}");
                if let Err(e) = dc_clone.send(&data).await {println!("Failed to send player data: {:?}", e)}
            })
        }));

        if let Some(p) = &self.state.peer_connection {
            let final_sdp: RTCSessionDescription =
                p.local_description().await.ok_or("No local description")?;
            let offer_sdp: String = final_sdp.sdp;

            let final_candidates: MutexGuard<'_, Vec<RTCIceCandidateInit>> =
                candidates.lock().await;

            let mut payload: BinaryEncoder = BinaryEncoder::new(true);
            payload.w_u8(0x00);
            payload.w_str(&offer_sdp);
            payload.w_json(&*final_candidates);
            payload.w_u16(0x0900);
            payload.w_u8(0x00);

            self.handle.binary(compress_signal_packet(1, &payload.data)).unwrap();

            Ok(())
        } else {
            Err("Something went Exceptionally Wrong!".into())
        }
    }

    async fn on_text(&mut self, text: ezsockets::Utf8Bytes) -> Result<(), ezsockets::Error> {
        println!("received message: {text}");
        Ok(())
    }

    async fn on_binary(&mut self, bytes: ezsockets::Bytes) -> Result<(), ezsockets::Error> {
        let decoded_bytes = decode_signal_packet(&bytes);
        let mut payload = BinaryDecoder::new(&decoded_bytes.1, false);
        let offer_sdp = payload.r_str();
        println!("{}", offer_sdp);
        let candidates_json_list: Vec<IceCandidateJson> = payload.r_json();

        if let Some(peer_connection) = &self.state.peer_connection {
            let desc = RTCSessionDescription::answer(offer_sdp)?;
            peer_connection.set_remote_description(desc).await?;

            for c in candidates_json_list {
                peer_connection
                    .add_ice_candidate(RTCIceCandidateInit {
                        candidate: c.candidate,
                        sdp_mid: Some(c.sdp_mid),
                        sdp_mline_index: Some(c.sdp_m_line_index),
                        username_fragment: c.username_fragment,
                    })
                    .await?;
            }
        }

        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        let data: Vec<u8> = call;
        dbg!(data);
        Ok(())
    }

    async fn on_close(
        &mut self,
        _frame: Option<ezsockets::CloseFrame>,
    ) -> Result<ezsockets::client::ClientCloseMode, ezsockets::Error> {
        println!("CLOSED!");
        Ok(ezsockets::client::ClientCloseMode::Close)
    }
}

pub async fn request_room_join(
    code: String,
    state: AppState,
) -> Result<ezsockets::Client<Client>, String> {
    let rq: ClientConfig =
        ClientConfig::new(format!("wss://p2p.haxball.com/client?id={}", code).as_str())
            // ehh good enough
            .header(
                "User-agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:148.0) Gecko/20100101 Firefox/148.0", // "Mozilla/5.0 (raylib; Linux x86_64; rv:147.0) HaxBall Rayball Client",
            )
            .header("Host", "p2p.haxball.com")
            .header("Origin", "https://www.haxball.com");

    let (handle, future) = ezsockets::connect(|h| Client { handle: h, state }, rq).await;
    tokio::spawn(async move {
        if let Err(e) = future.await {
            eprintln!("Connection closed: {e}");
        }
    });

    Ok(handle)
}

pub fn parse_code(clip: String) -> Result<std::string::String, Box<dyn Error>> {
    let parsed_url: Url = Url::parse(&clip)?;

    let query_map: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    if let Some(c) = query_map.get("c") {
        Ok(c.to_string())
    } else {
        Err("Didn't find code in URL".into())
    }
}
