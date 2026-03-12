use raylib::prelude::*;
use clipboard_rs::ClipboardContext;
use webrtc::peer_connection::RTCPeerConnection;
use crate::net::xcoder::BinaryEncoder;
use crate::ui::primitives::{Room, SettingData};
use crate::*;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct NavIcon {
    pub rect: Rectangle,
    pub screen: Screens,
}
pub struct AppState {
    pub navbar_buttons: Vec<NavIcon>,
    pub setting_toggles: Vec<SettingData>,
    pub errors: Vec<Alert>,
    pub rooms_fetch_error: String,
    pub rooms_list: Vec<Room>,
    pub rooms_fetching: bool,
    pub rooms_fetched: bool,
    pub rooms_fetched_once: bool,
    pub scroll_offset: usize,
    pub rooms_per_page: usize,
    pub scroll_amount: usize,
    pub current_screen: Screens,
    pub amount_of_dots_in_loading_text: f32,
    pub text_widths: HashMap<&'static str, i32>,
    pub tx: mpsc::UnboundedSender<Result<Vec<Room>, String>>,
    pub rx: mpsc::UnboundedReceiver<Result<Vec<Room>, String>>,
    pub clipboard_ctx: ClipboardContext,
    pub program_state: ProgramState,
    pub state: BinaryEncoder,
    pub logo_letter_amp_timer: f32,
    pub logo_letter_amp_tween: raylib::ease::Tween,
    pub ro_datachannel: Option<Arc<webrtc::data_channel::RTCDataChannel>>,
    pub ru_datachannel: Option<Arc<webrtc::data_channel::RTCDataChannel>>,
    pub uu_datachannel: Option<Arc<webrtc::data_channel::RTCDataChannel>>,
    pub peer_connection: Option<Arc<RTCPeerConnection>>,
}

impl AppState {
    pub fn push_error(&mut self, text: String, fade: bool) {
        self.errors.push(Alert::new(text, fade));
    }
    pub fn change_state(&mut self, state: ProgramState) {
        self.program_state = state;
    }
}