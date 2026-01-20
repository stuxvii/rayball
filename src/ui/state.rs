use raylib::prelude::*;
use clipboard_rs::ClipboardContext;
use tokio_tungstenite::WebSocketStream;
use crate::ui::primitives::{Room, SettingData};
use crate::*;
use std::collections::HashMap;
use tokio::sync::mpsc;
use futures::future::BoxFuture;

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
    pub websocket_future: Option<BoxFuture<'static, (WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>)>>,
    pub logo_letter_amp_timer: f32,
    pub logo_letter_amp_tween: raylib::ease::Tween,
}

impl AppState {
    pub fn push_error(&mut self, text: String, fade: bool) {
        self.errors.push(Alert::new(text, fade));
    }
}