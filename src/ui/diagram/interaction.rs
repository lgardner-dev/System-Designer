use crate::model::Position;
use eframe::egui::{self, Pos2, Response};
/// Only the response's own layer may begin a gesture. Captured gestures do not
/// use this gate: they continue outside the viewport until release/cancellation.
pub(in crate::ui) fn owns_press(ui: &egui::Ui, response: &Response) -> bool {
    ui.is_enabled()
        && response.contains_pointer()
        && !egui::Popup::is_any_open(ui.ctx())
        && ui.input(|i| i.focused)
}
pub(in crate::ui) fn cancelled(ui: &egui::Ui) -> bool {
    !ui.is_enabled() || ui.input(|i| !i.focused)
}
pub(in crate::ui) fn moved(initial: Position, start: Pos2, cursor: Pos2, zoom: f32) -> Position {
    let delta = (cursor - start) / zoom;
    Position {
        x: initial.x + f64::from(delta.x),
        y: initial.y + f64::from(delta.y),
    }
}
