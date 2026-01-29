use crate::ui::state::AppState;
use crate::net::xcoder::Encoder;
use crate::cfg::layout;
use crate::*;
use futures::SinkExt;
use tokio_tungstenite::{tungstenite::Message, *};
use std::task::{Context, Poll};

async fn send_data_to_ws(fut: (WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>>))  -> Result<Message, tokio_tungstenite::tungstenite::Error> {
    let (mut write, read) = fut;
    let mut plr_state: Encoder = Encoder::new(None, None);
    plr_state.write_str_len_leb128(&cfg_val!(USERNAME));
    write.send(Message::binary(plr_state.as_slice())).await?;
    dbg!(plr_state);
    if let Some(msg) = read.into_body() {
        Ok(Message::binary(msg))
    } else {
        Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
    }
}

pub fn draw_joining(d: &mut RaylibDrawHandle, state: &mut AppState, cx: &mut Context, screen_width: i32, screen_height: i32) {
    if let Some(ref mut wssf) = state.websocket_future {
        match wssf.as_mut().poll(cx) {
            Poll::Ready(fut) => {
                
                tokio::spawn( async move {
                    match send_data_to_ws(fut).await {
                        Ok(m) => println!("{m}"),
                        Err(e) => println!("{e}")
                    };
                });
                state.program_state = ProgramState::InGame;
            }
            Poll::Pending => {}
        }
    } else {
        state.push_error("A WebSocket Future was expected, but wasn't present".to_owned(), false);
        state.program_state = ProgramState::Menu;
    };

    let text = "Connecting to master...";
    let x: i32 = (screen_width / 2) - (d.measure_text(text, layout::FONT_SIZE) / 2);
    let y: i32 = screen_height / 2;
    let font_size: i32 = layout::FONT_SIZE;
    let color: Color = clr_val!(PRIMARY_COLOR);
    d.draw_text(text, x, y, font_size, color);
}