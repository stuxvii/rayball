use crate::cfg::layout;
use crate::*;
use crate::ui::state::AppState;

pub fn draw_joining(
    d: &mut RaylibDrawHandle<'_>,
    state: &mut AppState,
    screen_width: i32,
    screen_height: i32,
) {
    if let Some(task) = &mut state.join_task && let Ok(handle) = task.try_recv() {
        state.ws_client = Some(handle);
        state.program_state = ProgramState::InGame;
        state.join_task = None;
    }

    d.draw_text("Johnning The Roome", (d.measure_text("Johnning The Roome", layout::FONT_SIZE)/2)-screen_width/2, screen_height/2, layout::FONT_SIZE, clr_val!(PRIMARY_COLOR));
}
