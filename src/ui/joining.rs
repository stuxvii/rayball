use raylib::prelude::*;
use futures::AsyncWriteExt;
use futures::io::sink;
use crate::cfg::layout;
use crate::*;
use std::task::{Context, Poll};
use crate::ui::state::AppState;

pub fn draw_joining(d: &mut RaylibDrawHandle, state: &mut AppState, cx: &mut Context, screen_width: i32, screen_height: i32) {
    if let Some(ref mut wssf) = state.websocket_future {
        match wssf.as_mut().poll(cx) {
            Poll::Ready(fut) => {
                let (wss, resp) = fut;
                println!("{:?}{:#?}", wss, resp);
                state.program_state = ProgramState::InGame;
                
                tokio::spawn(async move {
                    let mut writer = sink();
                    let _ = writer.write(&[2]).await;
                });
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