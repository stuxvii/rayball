use crate::net::xcoder::BinaryEncoder;
use async_trait::async_trait;
use base64::write::EncoderWriter;
use ezsockets::ClientConfig;
use flate2::Compression;
use std::{collections::HashMap, error::Error, io::Write, sync::Arc};
use tokio::sync::{Mutex, Notify};
use url::Url;

use webrtc::{
    api::{APIBuilder, media_engine::MediaEngine},
    data_channel::data_channel_init::RTCDataChannelInit,
    ice_transport::ice_server::RTCIceServer,
    peer_connection::configuration::RTCConfiguration,
};

pub struct Client {
    handle: ezsockets::Client<Self>,
}

fn encode_signal_packet(op_code: u8, payload_buffer: &[u8]) -> Vec<u8> {
    let mut encoder: flate2::write::DeflateEncoder<Vec<u8>> =
        flate2::write::DeflateEncoder::new(Vec::new(), Compression::default());

    encoder.write_all(payload_buffer).unwrap();
    let compressed: Vec<u8> = encoder.finish().expect("Failed to compress");

    let mut packet: Vec<u8> = Vec::with_capacity(1 + compressed.len());
    packet.push(op_code);
    packet.extend_from_slice(&compressed);
    packet
}

fn hex_to_u8_manual(hex: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    // Check if the hex string length is even
    if hex.len() % 2 != 0 {
        // return Err(ParseIntError); // Simplified error for demo; see note below
    }

    // Iterate over the hex string in chunks of 2 characters
    let bytes: Result<Vec<u8>, _> = hex
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            // Convert chunk (2 bytes) to a &str
            let hex_chunk = std::str::from_utf8(chunk).expect("Invalid UTF-8 in hex string");
            // Parse hex chunk to u8 (base 16)
            u8::from_str_radix(hex_chunk, 16)
        })
        .collect();

    bytes
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

        // this shit is ridiculous
        let finished_gathering: Arc<Notify> = Arc::new(Notify::new());
        let notify_clone: Arc<Notify> = Arc::clone(&finished_gathering);
        
        let candidate_buffer: Arc<Mutex<BinaryEncoder>> =
            Arc::new(Mutex::new(BinaryEncoder::new(0, false)));
        let cb_clone = Arc::clone(&candidate_buffer);

        peer_connection.on_ice_candidate(Box::new(move |candidate| {
            let inner_buffer = Arc::clone(&cb_clone);
            let inner_notify = Arc::clone(&notify_clone);

            Box::pin(async move {
                match candidate {
                    Some(cand) => {
                        let json_obj = cand.to_json().unwrap();
                        let x = encode_signal_packet(4, json_obj.candidate.as_bytes());
                        inner_buffer.lock().await.append_bytes(&x);
                    }
                    None => {
                        inner_notify.notify_one();
                    }
                }
            })
        }));
        
        peer_connection.create_data_channel("ro", None).await?;
        peer_connection
            .create_data_channel(
                "ru",
                Some(RTCDataChannelInit {
                    ordered: Some(false),
                    max_retransmits: Some(1),
                    ..Default::default()
                }),
            )
            .await?;

        peer_connection
            .create_data_channel(
                "uu",
                Some(RTCDataChannelInit {
                    ordered: Some(false),
                    max_retransmits: Some(0),
                    ..Default::default()
                }),
            )
            .await?;

        let offer: webrtc::peer_connection::sdp::session_description::RTCSessionDescription =
            peer_connection.create_offer(None).await?;
        peer_connection.set_local_description(offer.clone()).await?;
        
        finished_gathering.notified().await;
        let guard = candidate_buffer.lock().await;

        let offer_sdp: String = offer.sdp;

        let mut handshake: BinaryEncoder = BinaryEncoder::new(16, true);
        handshake.w_u16(9);
        handshake.w_nullable_str_len(None);

        let mut payload: BinaryEncoder = BinaryEncoder::new(256, true);
        payload.w_u8(0);
        payload.w_str_len(&offer_sdp);
        payload.w_str_len("[]");
        payload.append_bytes(&handshake.data);

        let r = self.handle.binary(encode_signal_packet(1, &guard.data));

        // let text = "01bd944d6bdc3010867328942eec7f107b2c913b922d4b1ad883b31f742109a1bba187528a6cc989a9bfb09d649bd24bff6dff45b59ba409851e7248b10d2faf06e699d18c0f7ebeba9ec278d44cabe6b6284b1304c1e6fd6afdc5bfebf959f2619350ad03204c84c064a843c541ca5000030264754a5667118160ff8c47fd948e47c3d41f8d47669a17f585ebdaaea807ec2f0de522264aa26238d3385be09261bcc044239f2124389f63b240e0b858220f5124a8bce935602291cd70ae512f71ae7016a216b81498449824c805f2234c96b85cecd25e74cd558b47e7a7f3e3c51d489139dab443d1d43d0e5d917d2dddceaefac2d2de55a61e8a0c3f9eacc9dbf1a89a9ab62d8bccecc28926e7f3b377f3cdf1fadd7ab63923372eed868c5a3398ecd2d4b52bc7a36cfa771bccb477b5ed5c76bdd36e3b54a6a5a62c9b1b5a155b671f98da1b8b1900e791764ec72a4e63885d08792a25b7228bb5740fb15779672e90dbdc01cbd51ebfb0789f6cf0059b6c684ddfef0d2f69db74030a807d4865b6b4727d6f2e1ced8b5b870c642823a678381efd7afde9fb2433b52d7c5d6e828f1afdc5ef3a4038e39c2909200973b9342e026a621dd288bb88a632cda8cbe3d479eed41911944d664a2284569c0cdf5a72d9f4c3e470d2dbf6e4b8a8ddaab66e3b41b8730aeb73823fbeea5d579bca2d7da995ab076f3f143cf971f82f46fe84910b2ea29068a9a43490529b1a4d239903d5cc2b151bc6988e2c17fa9e31128ac72fce1879c6cd6cc708c267d49ef1197dd47ff8c890b55e3be26fbbb8762f852b9ee00a1e45523fa7a5ff1d97dd4f008b959f3819723fa52a0e380f840c9822a18805eca1fa2e2fb7a433d6760feb4abadda6107829b8f0112e06c163bf425c01f8dfa0ff52860a6c8e7e881f07f1e5203f1fbc39f80d";
        // let bin = hex_to_u8_manual(text).unwrap();
        // let r = self.handle.binary(bin);
        r.unwrap();
        println!("sent data!");

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
                "Mozilla/5.0 (X11; Linux x86_64; rv:147.0) Gecko/17.38 Firefox/147.0.0.0.0.0.0",
            )
            .header("Origin", "https://www.haxball.com")
            .header("Sec-Fetch-Dest", "empty")
            .header("Sec-Fetch-Mode", "websocket")
            .header("Sec-Fetch-Site", "same-site");

    let (handle, future) = ezsockets::connect(|h| Client { handle: h }, rq).await;
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
