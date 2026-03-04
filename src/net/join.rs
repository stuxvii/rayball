use crate::net::idkey::IdKey;
use crate::net::xcoder::{BinaryDecoder, BinaryEncoder};
use crate::prelude::USERNAME;
use async_trait::async_trait;
use ezsockets::ClientConfig;
use flate2::write::DeflateEncoder;
use flate2::{Compression, read::DeflateDecoder};
use std::io::Read;
use std::{collections::HashMap, error::Error, io::Write, sync::Arc};
use tokio::sync::{Mutex, MutexGuard, mpsc};
use url::Url;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use webrtc::{
    api::{APIBuilder, media_engine::MediaEngine},
    data_channel::data_channel_init::RTCDataChannelInit,
    ice_transport::{
        ice_candidate::RTCIceCandidateInit, ice_gatherer_state::RTCIceGathererState,
        ice_server::RTCIceServer,
    },
    peer_connection::configuration::RTCConfiguration,
};

pub struct Client {
    handle: ezsockets::Client<Self>,
    peer_connection: Option<Arc<webrtc::peer_connection::RTCPeerConnection>>,
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

fn encode_signal_packet(op_code: u8, payload_buffer: &[u8]) -> Vec<u8> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(6));
    encoder.write_all(payload_buffer).unwrap();
    let compressed = encoder.finish().expect("Failed to compress");

    let mut packet = Vec::with_capacity(1 + compressed.len());
    packet.push(op_code);
    packet.extend_from_slice(&compressed);
    packet
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Vec<u8>;

    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
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

        let (done_tx, mut done_rx) = mpsc::channel::<()>(1);

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

        peer_connection.on_ice_gathering_state_change(Box::new(move |state| {
            let done_tx = done_tx.clone();
            Box::pin(async move {
                if state == RTCIceGathererState::Complete {
                    let _ = done_tx.send(()).await;
                }
            })
        }));

        peer_connection.gathering_complete_promise().await;

        let dc_ro = peer_connection.create_data_channel("ro", None).await?;
        let dc_uu = peer_connection
            .create_data_channel(
                "uu",
                Some(RTCDataChannelInit {
                    ordered: Some(false),
                    max_retransmits: Some(0),
                    ..Default::default()
                }),
            )
            .await?;
        let dc_ru = peer_connection
            .create_data_channel(
                "ru",
                Some(RTCDataChannelInit {
                    ordered: Some(false),
                    max_retransmits: Some(1),
                    ..Default::default()
                }),
            )
            .await?;

        let dc_uu_label = dc_uu.label().to_owned();
        dc_uu.on_open(Box::new( move || {
        println!("Data channel {dc_uu_label} is now open!!!");
        Box::pin(async{})
    }));

        dc_uu.on_message(Box::new(move |msg| {
            let payload = String::from_utf8_lossy(&msg.data);
            println!("Received message: {}", payload);
            Box::pin(async {})
        }));

        let dc_ru_label = dc_ru.label().to_owned();
        dc_ru.on_open(Box::new( move || {
        println!("Data channel {dc_ru_label} is now open!!!");
        Box::pin(async{})
    }));

        dc_ru.on_message(Box::new(move |msg| {
            let payload = String::from_utf8_lossy(&msg.data);
            println!("Received message: {}", payload);
            Box::pin(async {})
        }));

        let dc_ro_label = dc_ro.label().to_owned();
        let dc_clone = Arc::clone(&dc_ro);
        dc_ro.on_open(Box::new(move || {
            println!("Data channel {dc_ro_label} is now open!");
            let dc_clone = Arc::clone(&dc_clone);
            Box::pin(async move {
                let id_key = IdKey::generate();
                
                let mut player_data = BinaryEncoder::new(256, true);
                player_data.w_u8(0x01);
                player_data.w_nullable_str(Some(&cfg_val!(USERNAME)));
                player_data.w_nullable_str(Some(&cfg_val!(AVATAR)));
                player_data.w_str(&cfg_val!(COUNTRY));
                player_data.w_str(&id_key.get());
                let data = bytes::Bytes::copy_from_slice(player_data.bytes());
                println!("Sending player data through dc_{dc_ro_label}");
                if let Err(e) = dc_clone.send(&data).await {
                    println!("Failed to send player data: {:?}", e);
                }
            })
        }));

        dc_ro.on_message(Box::new(move |msg| {
            let payload = String::from_utf8_lossy(&msg.data);
            println!("Received message: {}", payload);
            Box::pin(async {})
        }));

        let offer = peer_connection.create_offer(None).await?;
        peer_connection.set_local_description(offer.clone()).await?;
        self.peer_connection = Some(peer_connection);
        let _ = done_rx.recv().await;

        let offer_sdp: String = offer.sdp;

        let final_candidates: MutexGuard<'_, Vec<RTCIceCandidateInit>> = candidates.lock().await;
        let json_string: String = serde_json::to_string(&*final_candidates)?;

        let mut payload: BinaryEncoder = BinaryEncoder::new(256, true);
        payload.w_u8(0x00);
        payload.w_str(&offer_sdp);
        payload.w_str(&json_string);
        payload.w_u16(0x0900);
        payload.w_u8(0x00);

        let data = encode_signal_packet(1, &payload.data);
        self.handle.binary(data).unwrap();

        Ok(())
    }

    async fn on_text(&mut self, text: ezsockets::Utf8Bytes) -> Result<(), ezsockets::Error> {
        println!("received message: {text}");
        Ok(())
    }

    async fn on_binary(&mut self, bytes: ezsockets::Bytes) -> Result<(), ezsockets::Error> {
        let decoded_bytes = decode_signal_packet(&bytes);

        let mut payload = BinaryDecoder::new(&decoded_bytes.1, false);
        let offer_sdp = payload.r_str();
        let candidates_json_list: Vec<IceCandidateJson> = payload.r_json();

        println!("Candidates: {:?}", candidates_json_list);
        if let Some(peer_connection) = &self.peer_connection {
            println!("SDP Answer being set: {}", offer_sdp);
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
        println!("FUCK! CLOSED!");
        Ok(ezsockets::client::ClientCloseMode::Close)
    }
}

pub async fn request_room_join(code: String) -> Result<ezsockets::Client<Client>, String> {
    let rq: ClientConfig =
        ClientConfig::new(format!("wss://p2p.haxball.com/client?id={}", code).as_str())
            // ehh good enough
            .header(
                "User-agent",
                "Mozilla/5.0 (raylib; Linux x86_64; rv:147.0) HaxBall Rayball Client",
            )
            .header("Origin", "https://www.haxball.com");

    let (handle, future) = ezsockets::connect(
        |h| Client {
            handle: h,
            peer_connection: None,
        },
        rq,
    )
    .await;
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
