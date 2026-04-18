use crate::net::xcoder::{BinaryDecoder, BinaryEncoder};
use crate::ui::state::AppState;
use async_trait::async_trait;
use ezsockets::ClientConfig;
use flate2::write::DeflateEncoder;
use flate2::{Compression, read::DeflateDecoder};
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::data_channel::RTCDataChannel;
use webrtc::peer_connection::RTCPeerConnection;
use std::io::Read;
use std::{collections::HashMap, error::Error, io::Write, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};
use url::Url;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use webrtc::{
    api::APIBuilder,
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

struct RtcSession {
    peer_connection: Arc<RTCPeerConnection>,
    candidates: Arc<Mutex<Vec<RTCIceCandidateInit>>>,
    channels: Vec<Arc<RTCDataChannel>>,
}

async fn rtc_offer_shenanigans() -> Result<RtcSession, Box<dyn Error + Send + Sync>> {
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;
    let api: webrtc::api::API = APIBuilder::new().with_media_engine(m).build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let peer_connection: Arc<RTCPeerConnection> = Arc::new(api.new_peer_connection(config).await?);
    let candidates = Arc::new(Mutex::new(Vec::new()));

    let c_clone = Arc::clone(&candidates);
    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        let c_clone = Arc::clone(&c_clone);
        Box::pin(async move {
            if let Some(Ok(json)) = candidate.map(|c| c.to_json()) {
                c_clone.lock().await.push(json);
            }
        })
    }));
    
    let channel_configs = vec![
        ("ro", None),
        ("ru", Some(RTCDataChannelInit { ordered: Some(false), max_retransmits: Some(1), ..Default::default() })),
        ("uu", Some(RTCDataChannelInit { ordered: Some(false), max_retransmits: Some(0), ..Default::default() })),
    ];

    let mut channels = Vec::new();
    for (label, init) in channel_configs {
        channels.push(peer_connection.create_data_channel(label, init).await?);
    }

    let offer = peer_connection.create_offer(None).await?;
    peer_connection.set_local_description(offer).await?;
    
    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    let _ = gather_complete.recv().await;
    let _final_sdp = peer_connection.local_description().await.ok_or("...")?.sdp;
    
    Ok(RtcSession { peer_connection, candidates, channels })
}

#[async_trait]
impl ezsockets::ClientExt for Client {
    type Call = Vec<u8>;

    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
        let session: RtcSession = rtc_offer_shenanigans().await?;
        
        for dc in &session.channels {
            let label: String = dc.label().to_owned();
            
            dc.on_open(Box::new(move || {
                println!("Data channel {label} is now open!");
                Box::pin(async {})
            }));

            dc.on_message(Box::new(move |msg| {
                let payload = String::from_utf8_lossy(&msg.data);
                println!("Received message: {payload}");
                Box::pin(async {})
            }));

            match dc.label() {
                "ro" => self.state.ro_datachannel = Some(Arc::clone(dc)),
                "ru" => self.state.ru_datachannel = Some(Arc::clone(dc)),
                "uu" => self.state.uu_datachannel = Some(Arc::clone(dc)),
                _ => {}
            }
        }

        self.state.peer_connection = Some(session.peer_connection);

        if let Some(pc) = &self.state.peer_connection {
            let sdp_offer: String = pc.local_description().await.ok_or("No local description. :(")?.sdp;
            let candidates = session.candidates.lock().await;

            let mut payload = BinaryEncoder::new(true);
            payload.w_u8(0x00);
            payload.w_str(&sdp_offer);
            payload.w_json(&*candidates);
            payload.w_u16(0x0900);
            payload.w_nullable_str(None);

            self.handle.binary(compress_signal_packet(1, &payload.data)).unwrap();
            Ok(())
        } else {
            Err("Peer connection missing".into())
        }
    }

    async fn on_text(&mut self, text: ezsockets::Utf8Bytes) -> Result<(), ezsockets::Error> {
        println!("received message: {text}");
        Ok(())
    }

    async fn on_binary(&mut self, bytes: ezsockets::Bytes) -> Result<(), ezsockets::Error> {
        let decoded_bytes: (u8, Vec<u8>) = decode_signal_packet(&bytes);
        let mut payload: BinaryDecoder<'_> = BinaryDecoder::new(&decoded_bytes.1, false);
        let answer_sdp: String = payload.r_str();
        println!("Received Answer SDP: {}", answer_sdp);
        let candidates_json_list: Vec<IceCandidateJson> = payload.r_json();

        if let Some(peer_connection) = &self.state.peer_connection {
            let desc = RTCSessionDescription::answer(answer_sdp)?;
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
                // "Mozilla/5.0 (X11; Linux x86_64; rv:148.0) Gecko/20100101 Firefox/148.0", 
                "Mozilla/5.0 (raylib; Linux x86_64; rv:147.0) HaxBall Rayball Client",
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
