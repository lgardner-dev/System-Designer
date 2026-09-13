use super::*;
use crate::ui::canvas::{
    geometry::Path,
    motion::{self, Motion},
};
use egui::{
    Align2, Color32, FontId, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use std::collections::BTreeMap;
const ACCENT: Color32 = Color32::from_rgb(99, 195, 219);
#[derive(Clone)]
pub(super) enum Gesture {
    Move {
        id: String,
        start: Pos2,
        initial: Position,
        at: Position,
    },
    Connect {
        id: String,
        output: bool,
        start: Pos2,
    },
    Pan {
        start: Pos2,
        initial: Vec2,
    },
}
/// Ray intersection with the actual shape, not its invisible bounding corners.
pub(super) fn perimeter(rect: Rect, kind: StepKind, toward: Pos2) -> (Pos2, Vec2) {
    let d = toward - rect.center();
    let d = if d.length_sq() < 0.001 { Vec2::X } else { d };
    let a = rect.width() * 0.5;
    let b = rect.height() * 0.5;
    let t = match kind {
        StepKind::Decision => 1.0 / (d.x.abs() / a + d.y.abs() / b),
        StepKind::Entry | StepKind::Outcome => 1.0 / ((d.x / a).powi(2) + (d.y / b).powi(2)).sqrt(),
        _ => 1.0 / (d.x.abs() / a).max(d.y.abs() / b),
    };
    let v = d * t;
    let normal = match kind {
        StepKind::Decision => vec2(d.x.signum() / a, d.y.signum() / b).normalized(),
        StepKind::Entry | StepKind::Outcome => vec2(v.x / (a * a), v.y / (b * b)).normalized(),
        _ => {
            if (v.x.abs() - a).abs() < 0.01 {
                vec2(d.x.signum(), 0.0)
            } else {
                vec2(0.0, d.y.signum())
            }
        }
    };
    (rect.center() + v, normal)
}
fn route(a: Rect, ak: StepKind, b: Rect, bk: StepKind, same: bool) -> Path {
    let (from, na, to, nb) = if same {
        (a.right_center(), Vec2::X, a.center_top(), -Vec2::Y)
    } else {
        let (from, na) = perimeter(a, ak, b.center());
        let (to, nb) = perimeter(b, bk, a.center());
        (from, na, to, nb)
    };
    Path::between(from, na, to, nb)
}
fn shape(painter: &egui::Painter, r: Rect, kind: StepKind, fill: Color32, stroke: Stroke) {
    match kind {
        StepKind::Decision => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    r.center_top(),
                    r.right_center(),
                    r.center_bottom(),
                    r.left_center(),
                ],
                fill,
                stroke,
            ));
        }
        StepKind::Entry | StepKind::Outcome => {
            let points = (0..64)
                .map(|i| {
                    let angle = i as f32 * std::f32::consts::TAU / 64.0;
                    r.center()
                        + vec2(
                            angle.cos() * r.width() * 0.5,
                            angle.sin() * r.height() * 0.5,
                        )
                })
                .collect();
            painter.add(egui::Shape::convex_polygon(points, fill, stroke));
        }
        _ => {
            painter.rect(r, 3.0, fill, stroke, StrokeKind::Inside);
            if kind == StepKind::Call {
                for x in [r.left() + 10.0, r.right() - 10.0] {
                    painter.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], stroke);
                }
            }
        }
    }
}
fn short(s: &str, n: usize) -> String {
    if s.chars().count() > n {
        format!(
            "{}…",
            s.chars().take(n.saturating_sub(1)).collect::<String>()
        )
    } else {
        s.into()
    }
}
impl Designer {
    pub(in crate::ui) fn flow_view(&mut self, ui: &mut egui::Ui) {
        let p = self.store.snapshot();
        let owner = self.owner.clone();
        let Some(f) = p.behavior.get(&owner) else {
            ui.add_space(45.0);
            ui.heading("Control flow not specified");
            ui.label("Describe the work here, then extract responsibilities when their purpose and boundary are clear.");
            if ui.button("Start a flow").clicked() {
                self.dialog = Some(Dialog::Flow(FlowDialog::Start));
            }
            if ui.button("Open Interfaces").clicked() {
                self.go_scope(owner, Layer::Interfaces);
            }
            return;
        };
        ui.horizontal_wrapped(|ui| {
            ui.menu_button("+ Step", |ui| {
                for kind in [
                    StepKind::Action,
                    StepKind::Decision,
                    StepKind::Merge,
                    StepKind::Call,
                    StepKind::Entry,
                    StepKind::Outcome,
                ] {
                    if ui.button(kind.label()).clicked() {
                        ui.close();
                        self.new_step(kind);
                    }
                }
            });
            if ui.button("Connect steps…").clicked() {
                self.transition_dialog(None);
            }
            if ui.button("Extract selected steps…").clicked() {
                self.extraction_dialog(false);
            }
            if ui.button("Suggest a responsibility…").clicked() {
                self.extraction_dialog(true);
            }
        });
        let mut lights = Motion::Off;
        ui.horizontal_wrapped(|ui| {
            if ui.button("Fit").clicked() {
                self.canvas.fit_requested = true;
            }
            ui.checkbox(&mut self.flow.focus, "Focus selection");
            lights = Motion::controls(ui);
            ui.small("Drag ○ out -> ○ in, or reverse. Ctrl/Shift-click selects a region.");
        });
        if let Some((scope, component)) = self
            .flow
            .success
            .clone()
            .filter(|(o, c)| o == &owner && p.node(c).is_some())
        {
            ui.horizontal_wrapped(|ui| {
                ui.colored_label(
                    ACCENT,
                    format!("Created {}", behavior::name(&p, &component)),
                );
                if ui.button("Inspect interfaces").clicked() {
                    self.show_component_interfaces(&component);
                }
                if ui.button("Enter component").clicked() {
                    self.go_scope(component, Layer::Flow);
                }
                if ui.small_button("Dismiss").clicked() {
                    self.flow.success = None;
                }
            });
            if self.owner != scope || self.layer != Layer::Flow {
                return;
            }
        }
        let (area, response) = ui.allocate_exact_size(
            ui.available_size().max(vec2(200.0, 150.0)),
            Sense::click_and_drag(),
        );
        let painter = ui.painter_at(area);
        painter.rect_filled(area, 0.0, Color32::from_rgb(17, 22, 30));
        let mut positions = behavior::positions(f);
        if let Some(saved) = p.flow_layout.get(&owner) {
            positions.extend(saved.iter().map(|(k, v)| (k.clone(), *v)));
        }
        if let Some(Gesture::Move { id, at, .. }) = &self.flow.gesture {
            positions.insert(id.clone(), *at);
        }
        let rects: BTreeMap<_, _> = f
            .steps
            .iter()
            .map(|s| {
                let pos = positions.get(&s.id).copied().unwrap_or_default();
                let size = if s.kind == StepKind::Decision {
                    vec2(260.0, 150.0)
                } else {
                    vec2(260.0, 112.0)
                };
                (
                    s.id.clone(),
                    Rect::from_min_size(pos2(pos.x as f32, pos.y as f32), size),
                )
            })
            .collect();
        let paths: Vec<_> = f
            .transitions
            .iter()
            .filter_map(|t| {
                Some((
                    t,
                    route(
                        *rects.get(&t.from)?,
                        f.step(&t.from)?.kind,
                        *rects.get(&t.to)?,
                        f.step(&t.to)?.kind,
                        t.from == t.to,
                    ),
                ))
            })
            .collect();
        let mut bounds = rects.values().fold(Rect::NOTHING, |r, c| r.union(*c));
        for (_, path) in &paths {
            bounds = bounds.union(path.bounds);
        }
        if !bounds.is_finite() {
            bounds = Rect::from_min_size(Pos2::ZERO, vec2(600.0, 400.0));
        }
        bounds = bounds.expand(45.0);
        if self.canvas.fit_requested {
            self.canvas.zoom = ((area.width() - 30.0) / bounds.width())
                .min((area.height() - 30.0) / bounds.height())
                .clamp(0.2, 1.0);
            self.canvas.pan = area.size() * 0.5 - bounds.center().to_vec2() * self.canvas.zoom;
            self.canvas.fit_requested = false;
        }
        let cursor = ui.input(|i| i.pointer.interact_pos());
        if response.hovered() && ui.is_enabled() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.01
                && let Some(cursor) = cursor
            {
                let world = (cursor - area.min - self.canvas.pan) / self.canvas.zoom;
                self.canvas.zoom = (self.canvas.zoom * (scroll * 0.002).exp()).clamp(0.2, 2.5);
                self.canvas.pan = cursor - area.min - world * self.canvas.zoom;
            }
        }
        let z = self.canvas.zoom;
        let pan = self.canvas.pan;
        let screen_rect = |r: Rect| {
            Rect::from_min_max(
                area.min + pan + r.min.to_vec2() * z,
                area.min + pan + r.max.to_vec2() * z,
            )
        };
        let screen: BTreeMap<_, _> = rects
            .iter()
            .map(|(id, r)| (id.clone(), screen_rect(*r)))
            .collect();
        #[cfg(test)]
        {
            self.flow.rects = screen.clone();
        }
        let screen_paths: Vec<_> = paths
            .iter()
            .map(|(t, path)| (*t, path.screen(area.min, pan, z)))
            .collect();
        let active = |t: &Transition| {
            !self.flow.focus
                || (self.flow.selection.steps.is_empty()
                    && self.flow.selection.transition.is_none())
                || self.flow.selection.steps.contains(&t.from)
                || self.flow.selection.steps.contains(&t.to)
                || self.flow.selection.transition.as_ref() == Some(&t.id)
        };
        let mut animate = false;
        for (t, path) in &screen_paths {
            if !path.bounds.expand(14.0).intersects(area) {
                continue;
            }
            let selected = self.flow.selection.transition.as_ref() == Some(&t.id);
            let emphasized = active(t);
            let color = if !emphasized {
                Color32::from_gray(54)
            } else if selected {
                Color32::YELLOW
            } else {
                ACCENT
            };
            painter.add(egui::Shape::line(
                path.points.clone(),
                Stroke::new(if selected { 3.0_f32 } else { 1.8_f32 }, color),
            ));
            let (tip, direction) = path.at(path.length);
            let side = vec2(-direction.y, direction.x);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    tip,
                    tip - direction * 11.0 + side * 5.0,
                    tip - direction * 11.0 - side * 5.0,
                ],
                color,
                Stroke::NONE,
            ));
            let label = if let Some(outcome) = &t.outcome {
                let name = f
                    .step(&t.from)
                    .and_then(|s| s.target.as_ref())
                    .and_then(|id| p.behavior.get(id))
                    .and_then(|f| f.step(outcome))
                    .map(|s| s.name.as_str())
                    .unwrap_or(outcome);
                if t.condition.is_empty() {
                    name.into()
                } else {
                    format!("{} · {}", name, t.condition)
                }
            } else {
                t.condition.clone()
            };
            if !label.is_empty() {
                let pos = path.at(path.length * 0.5).0 + vec2(0.0, -12.0);
                painter.text(
                    pos,
                    Align2::CENTER_BOTTOM,
                    short(&label, 36),
                    FontId::proportional((12.0 * z).max(10.0)),
                    color,
                );
            }
            let lit = lights == Motion::All
                || (lights == Motion::Selected
                    && (selected
                        || self.flow.selection.steps.contains(&t.from)
                        || self.flow.selection.steps.contains(&t.to)));
            if emphasized && lit && self.dialog.is_none() {
                motion::paint(&painter, path, ui.input(|i| i.time), &t.id);
                animate = true;
            }
        }
        for s in &f.steps {
            let r = screen[&s.id];
            if !r.intersects(area) {
                continue;
            }
            let selected = self.flow.selection.steps.contains(&s.id);
            let fill = if selected {
                Color32::from_rgb(31, 67, 79)
            } else {
                Color32::from_rgb(32, 40, 53)
            };
            let color = if selected { Color32::YELLOW } else { ACCENT };
            shape(
                &painter,
                r,
                s.kind,
                fill,
                Stroke::new(if selected { 2.5_f32 } else { 1.5_f32 }, color),
            );
            let clipped = painter.with_clip_rect(r.shrink(12.0 * z).intersect(area));
            let title_size = (15.0 * z).clamp(9.0, 20.0);
            let n = if s.kind == StepKind::Decision { 23 } else { 29 };
            clipped.text(
                r.center() - vec2(0.0, 12.0 * z),
                Align2::CENTER_CENTER,
                short(&s.name, n),
                FontId::proportional(title_size),
                Color32::WHITE,
            );
            let subtitle = if let Some(target) = &s.target {
                behavior::name(&p, target).to_owned()
            } else {
                s.kind.label().to_owned()
            };
            clipped.text(
                r.center() + vec2(0.0, 13.0 * z),
                Align2::CENTER_CENTER,
                short(&subtitle, 32),
                FontId::proportional((11.0 * z).max(8.0)),
                Color32::from_gray(172),
            );
            if s.kind != StepKind::Entry {
                painter.circle_filled(r.left_center(), 6.0, Color32::from_rgb(156, 204, 157));
                painter.text(
                    r.left_center() + vec2(10.0, 12.0),
                    Align2::LEFT_TOP,
                    "in",
                    FontId::proportional(9.0),
                    Color32::GRAY,
                );
            }
            if s.kind != StepKind::Outcome {
                painter.circle_filled(r.right_center(), 6.0, ACCENT);
                painter.text(
                    r.right_center() + vec2(-10.0, 12.0),
                    Align2::RIGHT_TOP,
                    "out",
                    FontId::proportional(9.0),
                    Color32::GRAY,
                );
            }
        }
        if animate {
            ui.ctx().request_repaint_after(Duration::from_millis(33));
        }
        if let (Some(Gesture::Connect { id, output, .. }), Some(cursor)) =
            (&self.flow.gesture, cursor)
            && let Some(r) = screen.get(id)
        {
            let a = if *output {
                r.right_center()
            } else {
                r.left_center()
            };
            let path = if *output {
                Path::between(a, Vec2::X, cursor, -Vec2::X)
            } else {
                Path::between(cursor, Vec2::X, a, -Vec2::X)
            };
            painter.add(egui::Shape::line(
                path.points,
                Stroke::new(2.0_f32, Color32::YELLOW),
            ));
        }
        if !ui.is_enabled() {
            return;
        }
        let Some(cursor) = cursor else {
            return;
        };
        let handle = f.steps.iter().find_map(|s| {
            let r = screen[&s.id];
            if s.kind != StepKind::Outcome && cursor.distance(r.right_center()) < 11.0 {
                Some((s.id.clone(), true))
            } else if s.kind != StepKind::Entry && cursor.distance(r.left_center()) < 11.0 {
                Some((s.id.clone(), false))
            } else {
                None
            }
        });
        if area.contains(cursor) && ui.input(|i| i.pointer.any_pressed()) {
            if ui.input(|i| i.pointer.button_pressed(PointerButton::Middle)) {
                self.flow.gesture = Some(Gesture::Pan {
                    start: cursor,
                    initial: pan,
                });
            } else if ui.input(|i| i.pointer.button_pressed(PointerButton::Primary)) {
                if let Some((id, output)) = &handle {
                    self.flow.gesture = Some(Gesture::Connect {
                        id: id.clone(),
                        output: *output,
                        start: cursor,
                    });
                } else if let Some(s) = f.steps.iter().rev().find(|s| {
                    let r = screen[&s.id];
                    r.contains(cursor)
                        && (cursor - r.center()).length()
                            <= (perimeter(r, s.kind, cursor).0 - r.center()).length() + 0.1
                }) {
                    let additive =
                        ui.input(|i| i.modifiers.ctrl || i.modifiers.shift || i.modifiers.command);
                    self.flow.selection.step(s.id.clone(), additive);
                    if !additive {
                        let initial = positions[&s.id];
                        self.flow.gesture = Some(Gesture::Move {
                            id: s.id.clone(),
                            start: cursor,
                            initial,
                            at: initial,
                        });
                    }
                } else if let Some((t, _)) = screen_paths
                    .iter()
                    .filter(|(_, path)| path.distance(cursor) < 7.0)
                    .min_by(|(_, a), (_, b)| a.distance(cursor).total_cmp(&b.distance(cursor)))
                {
                    self.flow.selection = FlowSelection {
                        transition: Some(t.id.clone()),
                        ..Default::default()
                    };
                } else {
                    self.flow.selection = FlowSelection::default();
                    self.flow.gesture = Some(Gesture::Pan {
                        start: cursor,
                        initial: pan,
                    });
                }
            }
        }
        if ui.input(|i| i.pointer.any_down()) {
            match &mut self.flow.gesture {
                Some(Gesture::Move {
                    start, initial, at, ..
                }) => {
                    let delta = (cursor - *start) / z;
                    *at = Position {
                        x: (initial.x + delta.x as f64).max(0.0),
                        y: (initial.y + delta.y as f64).max(0.0),
                    };
                }
                Some(Gesture::Pan { start, initial }) => {
                    self.canvas.pan = *initial + (cursor - *start)
                }
                _ => {}
            }
            if self.flow.has_gesture() {
                ui.ctx().request_repaint();
            }
        }
        if ui.input(|i| i.pointer.any_released()) {
            match self.flow.gesture.take() {
                Some(Gesture::Move {
                    id,
                    start,
                    initial,
                    at,
                }) => {
                    if cursor.distance(start) < 4.0
                        && ui.input(|i| i.pointer.button_double_clicked(PointerButton::Primary))
                    {
                        if let Some(s) = f.step(&id) {
                            self.dialog = Some(Dialog::Flow(FlowDialog::Step(s.clone())));
                        }
                    } else if at != initial {
                        self.publish(
                            "Move flow step",
                            edit::candidate(&p, |q| {
                                q.flow_layout.entry(owner).or_default().insert(id, at);
                                Ok(())
                            }),
                        );
                    }
                }
                Some(Gesture::Connect { id, output, start }) => {
                    if cursor.distance(start) > 4.0
                        && let Some((target, target_output)) = handle
                        && output != target_output
                    {
                        self.transition_dialog(Some(if output {
                            (id, target)
                        } else {
                            (target, id)
                        }));
                    }
                }
                _ => {}
            }
        }
    }
}
