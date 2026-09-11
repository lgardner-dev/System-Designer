//! Directional illustration only: no execution, throughput or timing semantics.
use super::{Selection, geometry::Path};
use crate::model::Edge;
use eframe::egui::{self, Color32, Id, Painter};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Motion {
    Off,
    Selected,
    #[default]
    All,
}
impl Motion {
    pub fn controls(ui: &mut egui::Ui) -> Self {
        // Session-wide, outside Project and CanvasState: navigation must not
        // turn motion back on after the user has disabled it.
        let id = Id::new("system-designer.direction-lights");
        let mut mode = ui
            .ctx()
            .data(|d| d.get_temp::<Self>(id))
            .unwrap_or_default();
        ui.horizontal_wrapped(|ui| {
            ui.label("Direction lights");
            egui::ComboBox::from_id_salt(id).selected_text(match mode {
                Self::Off => "Off", Self::Selected => "Selected", Self::All => "All",
            }).show_ui(ui, |ui| {
                ui.selectable_value(&mut mode, Self::Off, "Off");
                ui.selectable_value(&mut mode, Self::Selected, "Selected");
                ui.selectable_value(&mut mode, Self::All, "All");
            });
            ui.small("Direction preview — not live execution");
            ui.label("○ receives · ● produces").on_hover_text(
                "Port side is visual only. Hollow ports receive; filled ports produce at this level. Full port names and contracts are available on hover and in the inspector.",
            );
        });
        ui.ctx().data_mut(|d| d.insert_temp(id, mode));
        mode
    }
    pub fn includes(self, edge: &Edge, selected: &Selection) -> bool {
        match self {
            Self::Off => false,
            Self::All => true,
            Self::Selected => match selected {
                Selection::Edge(id) => edge.id == *id,
                Selection::Node(id) => {
                    edge.from.node.as_ref() == Some(id) || edge.to.node.as_ref() == Some(id)
                }
                Selection::Boundary(id) => {
                    (edge.from.node.is_none() && edge.from.port == *id)
                        || (edge.to.node.is_none() && edge.to.port == *id)
                }
                Selection::None => false,
            },
        }
    }
}
fn phase(id: &str) -> f64 {
    let hash = id.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    });
    (hash % 4096) as f64 / 4096.0
}
/// Arc length, not Bezier t: a light does not speed up through a tight bend.
/// A gap lets it disappear at the destination before restarting at the source.
pub(super) fn position(time: f64, length: f32, id: &str) -> Option<f32> {
    if length <= 0.01 || !length.is_finite() || !time.is_finite() {
        return None;
    }
    let period = length as f64 + 100.0;
    let distance = (time * 100.0 + phase(id) * period).rem_euclid(period);
    (distance <= length as f64).then_some(distance as f32)
}
pub(super) fn paint(painter: &Painter, path: &Path, time: f64, id: &str) {
    let Some(distance) = position(time, path.length, id) else {
        return;
    };
    let fade = (distance / 10.0)
        .min((path.length - distance) / 10.0)
        .clamp(0.0, 1.0);
    let color = |alpha: f32| Color32::from_rgba_unmultiplied(150, 222, 255, (alpha * fade) as u8);
    // Short tapered trail; these translucent layers approximate a soft bloom
    // without adding a postprocessing renderer or graphics dependency.
    for i in (1..=8).rev() {
        let at = distance - i as f32 * 3.0;
        if at >= 0.0 {
            painter.circle_filled(path.at(at).0, 1.5, color(150.0 * (1.0 - i as f32 / 9.0)));
        }
    }
    let point = path.at(distance).0;
    for (radius, alpha) in [(11.0, 7.0), (8.0, 14.0), (5.0, 36.0), (3.0, 105.0)] {
        painter.circle_filled(point, radius, color(alpha));
    }
    painter.circle_filled(
        point,
        1.8,
        Color32::from_rgba_unmultiplied(240, 251, 255, (245.0 * fade) as u8),
    );
}
