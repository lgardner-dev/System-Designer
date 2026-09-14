use super::*;
use crate::ui::canvas::{geometry::Path, motion::Motion};
use crate::ui::diagram::{
    interaction,
    paint::{self, ACCENT, EdgeStyle},
    viewport::Transform,
};
use egui::{
    Align2, Color32, FontId, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use std::collections::BTreeMap;
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
pub(super) fn perimeter(rect: Rect, kind: StepKind, toward: Pos2) -> (Pos2, Vec2) {
    scene::outline(kind).perimeter(rect, toward)
}
#[cfg(test)]
pub(super) fn routes<'a>(
    f: &'a Flow,
    rects: &BTreeMap<String, Rect>,
) -> Vec<(&'a Transition, Path)> {
    scene::routes(f, rects, &scene::anchors(f, rects))
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
        let mut positions = behavior::effective_positions(f, p.flow_layout.get(&owner));
        if let Some(Gesture::Move { id, at, .. }) = &self.flow.gesture {
            positions.insert(id.clone(), *at);
        }
        let rects: BTreeMap<_, _> = f
            .steps
            .iter()
            .map(|s| {
                let pos = positions.get(&s.id).copied().unwrap_or_default();
                let (width, height) = behavior::footprint(s.kind);
                let size = vec2(width as f32, height as f32);
                (
                    s.id.clone(),
                    Rect::from_min_size(pos2(pos.x as f32, pos.y as f32), size),
                )
            })
            .collect();
        let anchors = if let Some((before, frozen)) = &self.flow.frozen {
            let mut anchors = frozen.clone();
            for ((id, _), anchor) in &mut anchors {
                anchor.point += rects[id].min - before[id].min;
            }
            anchors
        } else {
            scene::anchors(f, &rects)
        };
        let paths = scene::routes(f, &rects, &anchors);
        let mut bounds = rects.values().fold(Rect::NOTHING, |r, c| r.union(*c));
        for (_, path) in &paths {
            bounds = bounds.union(path.bounds);
        }
        if !bounds.is_finite() {
            bounds = Rect::from_min_size(Pos2::ZERO, vec2(600.0, 400.0));
        }
        bounds = bounds.expand(45.0);
        let mut transform = Transform {
            area,
            pan: self.canvas.pan,
            zoom: self.canvas.zoom,
        };
        if self.canvas.fit_requested {
            transform.fit(bounds);
            self.canvas.fit_requested = false;
        }
        let cursor = ui.input(|i| i.pointer.interact_pos());
        if interaction::owns_press(ui, &response) {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.01
                && let Some(cursor) = cursor
            {
                transform.zoom_at(cursor, scroll);
            }
        }
        self.canvas.pan = transform.pan;
        self.canvas.zoom = transform.zoom;
        let z = transform.zoom;
        let pan = transform.pan;
        let screen_rect = |r| transform.rect(r);
        let screen: BTreeMap<_, _> = rects
            .iter()
            .map(|(id, r)| (id.clone(), screen_rect(*r)))
            .collect();
        #[cfg(test)]
        {
            self.flow.rects = screen.clone();
            self.flow.handles = anchors
                .iter()
                .map(|(id, a)| (id.clone(), transform.screen(a.point)))
                .collect();
        }
        let screen_paths: Vec<_> = paths
            .iter()
            .map(|(t, path)| (*t, path.screen(area.min, pan, z)))
            .collect();
        #[cfg(test)]
        {
            self.flow.paths = screen_paths
                .iter()
                .map(|(t, path)| (t.id.clone(), path.clone()))
                .collect();
        }
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
            paint::edge(
                &painter,
                path,
                EdgeStyle {
                    selected,
                    emphasized,
                    hovered: response.hovered() && cursor.is_some_and(|p| path.distance(p) < 7.0),
                },
                &short(&label, 36),
                z,
            );
            let lit = lights == Motion::All
                || (lights == Motion::Selected
                    && (selected
                        || self.flow.selection.steps.contains(&t.from)
                        || self.flow.selection.steps.contains(&t.to)));
            animate |= paint::light(ui, path, emphasized && lit && self.dialog.is_none(), &t.id);
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
            let color = ACCENT;
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
            for output in [false, true] {
                if let Some(anchor) = anchors.get(&(s.id.clone(), output)) {
                    let point = transform.screen(anchor.point);
                    paint::handle(&painter, point, output, false, color);
                    painter.text(
                        point + anchor.normal * 14.0,
                        Align2::CENTER_CENTER,
                        if output { "out" } else { "in" },
                        FontId::proportional(9.0),
                        Color32::GRAY,
                    );
                }
            }
        }
        paint::repaint(ui, animate);
        let connecting = match &self.flow.gesture {
            Some(Gesture::Connect { id, output, .. }) => Some((id.clone(), *output)),
            _ => self.flow.pending.clone(),
        };
        let handle = cursor.and_then(|cursor| {
            anchors
                .iter()
                .filter(|(_, a)| transform.screen(a.point).distance(cursor) <= 12.0)
                .min_by(|(ka, a), (kb, b)| {
                    transform
                        .screen(a.point)
                        .distance(cursor)
                        .total_cmp(&transform.screen(b.point).distance(cursor))
                        .then(ka.cmp(kb))
                })
                .map(|(key, _)| key.clone())
        });
        if let (Some(key), Some(cursor)) = (&connecting, cursor)
            && let Some(a) = anchors.get(key)
        {
            let target = handle.as_ref().and_then(|key| anchors.get(key));
            let end = target.map(|b| transform.screen(b.point)).unwrap_or(cursor);
            let normal = target.map(|b| b.normal).unwrap_or(-a.normal);
            let path = if key.1 {
                Path::between(transform.screen(a.point), a.normal, end, normal)
            } else {
                Path::between(end, normal, transform.screen(a.point), a.normal)
            };
            paint::edge(
                &painter,
                &path,
                EdgeStyle {
                    selected: true,
                    hovered: false,
                    emphasized: true,
                },
                "",
                z,
            );
        }
        if interaction::cancelled(ui) {
            self.flow.cancel();
            return;
        }
        let Some(cursor) = cursor else {
            if ui.input(|i| !i.pointer.any_down()) {
                self.flow.cancel();
            }
            return;
        };
        if interaction::owns_press(ui, &response) && ui.input(|i| i.pointer.any_pressed()) {
            if ui.input(|i| i.pointer.button_pressed(PointerButton::Middle)) {
                self.flow.gesture = Some(Gesture::Pan {
                    start: cursor,
                    initial: pan,
                });
            } else if ui.input(|i| i.pointer.button_pressed(PointerButton::Primary)) {
                if let Some((id, output)) = &handle {
                    self.flow.frozen = Some((rects.clone(), anchors.clone()));
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
                        self.flow.pending = None;
                        self.flow.frozen = Some((rects.clone(), anchors.clone()));
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
                    self.flow.pending = None;
                    self.flow.selection = FlowSelection::default();
                    self.flow.gesture = Some(Gesture::Pan {
                        start: cursor,
                        initial: pan,
                    });
                }
            }
        }
        if ui.input(|i| i.pointer.any_down() || i.pointer.any_released()) {
            match &mut self.flow.gesture {
                Some(Gesture::Move {
                    start, initial, at, ..
                }) => {
                    *at = interaction::moved(*initial, *start, cursor, z);
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
                            edit::move_elements(
                                &p,
                                &edit::LayoutScope::Flow(owner),
                                BTreeMap::from([(id, at)]),
                            ),
                        );
                    }
                }
                Some(Gesture::Connect { id, output, start }) => {
                    let from = if cursor.distance(start) <= 4.0 {
                        self.flow.pending.clone().unwrap_or((id, output))
                    } else {
                        (id, output)
                    };
                    if let Some((target, target_output)) = handle {
                        if from.1 != target_output {
                            self.flow.pending = None;
                            self.transition_dialog(Some(if from.1 {
                                (from.0, target)
                            } else {
                                (target, from.0)
                            }));
                        } else if cursor.distance(start) <= 4.0 {
                            self.flow.pending = Some((target, target_output));
                        } else {
                            self.flow.pending = None;
                        }
                    } else {
                        self.flow.pending = None;
                    }
                }
                _ => {}
            }
        }
        if !self.flow.has_gesture() {
            self.flow.frozen = None;
        }
    }
}
