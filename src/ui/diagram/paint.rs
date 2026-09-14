use super::{motion, routes::Path};
use eframe::egui::{self, Color32, FontId, Rect, Stroke, vec2};
pub(in crate::ui) const ACCENT: Color32 = Color32::from_rgb(91, 183, 217);
pub(in crate::ui) const BACKGROUND: Color32 = Color32::from_rgb(14, 18, 24);
#[derive(Clone, Copy)]
pub(in crate::ui) struct EdgeStyle {
    pub selected: bool,
    pub hovered: bool,
    pub emphasized: bool,
}
impl EdgeStyle {
    pub fn color(self) -> Color32 {
        if self.selected || self.hovered {
            ACCENT
        } else if self.emphasized {
            Color32::from_rgb(111, 129, 151)
        } else {
            Color32::from_rgb(46, 56, 70)
        }
    }
}
pub(in crate::ui) fn edge(
    painter: &egui::Painter,
    path: &Path,
    style: EdgeStyle,
    caption: &str,
    zoom: f32,
) {
    let color = style.color();
    painter.add(egui::Shape::line(
        path.points.clone(),
        Stroke::new(
            if style.selected {
                3.0_f32
            } else if style.emphasized {
                1.8
            } else {
                1.0
            },
            color,
        ),
    ));
    if path.length > 8.0 {
        let (head, tangent) = path.at((path.length - 7.0).max(0.0));
        painter.arrow(
            head - tangent * 13.0,
            tangent * 13.0,
            Stroke::new(1.5_f32, color),
        );
    }
    if style.emphasized && !caption.is_empty() {
        let galley = painter.layout_no_wrap(
            caption.into(),
            FontId::proportional((11.0 * zoom).max(9.0)),
            color,
        );
        let rect = Rect::from_center_size(
            path.at(path.length * 0.5).0 - vec2(0.0, 11.0),
            galley.size() + vec2(10.0, 4.0),
        );
        painter.rect_filled(rect, 3, BACKGROUND);
        painter.galley(rect.min + vec2(5.0, 2.0), galley, color);
    }
}
pub(in crate::ui) fn handle(
    painter: &egui::Painter,
    point: egui::Pos2,
    output: bool,
    active: bool,
    color: Color32,
) {
    painter.circle_filled(point, 5.0, if output { color } else { BACKGROUND });
    if !output {
        painter.circle_stroke(point, 5.0, Stroke::new(1.5_f32, color));
    }
    if active {
        painter.circle_stroke(point, 9.0, Stroke::new(1.0_f32, color));
    }
}
pub(in crate::ui) fn light(ui: &egui::Ui, path: &Path, enabled: bool, id: &str) -> bool {
    let active = enabled
        && ui.is_enabled()
        && ui.input(|i| i.focused && !i.viewport().minimized.unwrap_or(false));
    if active {
        motion::paint(ui.painter(), path, ui.input(|i| i.time), id);
    }
    active
}
pub(in crate::ui) fn repaint(ui: &egui::Ui, active: bool) {
    if active {
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(33));
    }
}
