use super::*;
use crate::edit;
use egui::{
    Align2, Color32, FontId, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use std::collections::BTreeMap;

mod arrangement;
mod focus;
pub(super) mod geometry;
#[cfg(test)]
mod integration_tests;
mod overview;
pub(super) use focus::{Focus, Session, Viewport};
pub(super) use focus::{details, valid as selection_valid};
pub(super) use overview::View;
pub(super) mod motion;
#[cfg(test)]
mod tests;
use geometry::{Anchor, Path, Scene, Side};

const ACCENT: Color32 = Color32::from_rgb(91, 183, 217);
const MUTED: Color32 = Color32::from_rgb(142, 153, 172);
#[derive(Clone)]
enum Gesture {
    Move {
        id: String,
        start: Pos2,
        initial: Position,
        current: Position,
    },
    Wire {
        port: Endpoint,
        start: Pos2,
    },
    Pan {
        start: Pos2,
        initial: Vec2,
    },
    Edge(Selection),
}
pub(super) struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub fit_requested: bool,
    gesture: Option<Gesture>,
    pending: Option<Endpoint>,
    frozen_scene: Option<Scene>,
    pub generation: Option<u64>,
    #[cfg(test)]
    port_positions: BTreeMap<String, Pos2>,
}
impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            fit_requested: true,
            gesture: None,
            pending: None,
            frozen_scene: None,
            generation: None,
            #[cfg(test)]
            port_positions: BTreeMap::new(),
        }
    }
}
impl CanvasState {
    pub fn cancel(&mut self) {
        self.gesture = None;
        self.pending = None;
        self.frozen_scene = None;
    }
    pub fn has_gesture(&self) -> bool {
        self.gesture.is_some() || self.pending.is_some()
    }
}
fn short(s: &str, count: usize) -> String {
    if s.chars().count() > count {
        format!(
            "{}…",
            s.chars().take(count.saturating_sub(1)).collect::<String>()
        )
    } else {
        s.into()
    }
}
fn pair(p: &Project, sid: &str, a: &Endpoint, b: &Endpoint) -> Option<(Endpoint, Endpoint)> {
    match (p.effective_direction(sid, a), p.effective_direction(sid, b)) {
        (Some(Direction::Out), Some(Direction::In)) => Some((a.clone(), b.clone())),
        (Some(Direction::In), Some(Direction::Out)) => Some((b.clone(), a.clone())),
        _ => None,
    }
}
fn port_labels(painter: &egui::Painter, port: &Anchor, rect: Rect, zoom: f32) {
    let clipped = painter.with_clip_rect(rect.intersect(painter.clip_rect()));
    let align = match port.side {
        Side::Left => Align2::LEFT_TOP,
        Side::Right => Align2::RIGHT_TOP,
        _ => Align2::CENTER_TOP,
    };
    let x = match port.side {
        Side::Left => rect.left(),
        Side::Right => rect.right(),
        _ => rect.center().x,
    };
    let font_size = (11.0 * zoom).max(8.0);
    let capacity = (rect.width() / (font_size * 0.58)).max(2.0) as usize;
    clipped.text(
        pos2(x, rect.top()),
        align,
        short(&port.name, capacity),
        FontId::proportional(font_size),
        Color32::from_rgb(217, 224, 234),
    );
    clipped.text(
        pos2(x, rect.top() + 14.0 * zoom),
        align,
        short(&port.contract, capacity),
        FontId::monospace((9.0 * zoom).max(7.0)),
        MUTED,
    );
    if port.boundary {
        clipped.text(
            pos2(x, rect.top() + 30.0 * zoom),
            align,
            short(&port.external, capacity),
            FontId::proportional((10.0 * zoom).max(7.0)),
            ACCENT,
        );
    }
}
impl Designer {
    pub(super) fn canvas_view(&mut self, ui: &mut egui::Ui) {
        self.normalize_selection();
        if matches!(self.dialog, Some(Dialog::Connection(_))) {
            self.canvas_session.view = View::Detail;
        }
        let motion = self.canvas_controls(ui);
        let compact = self.canvas_session.view == View::Overview;
        let mut emphasis = focus::Emphasis::new(
            self.store.project(),
            &self.current,
            &self.selected,
            self.canvas_session.focus,
        );
        if matches!(self.canvas.gesture, Some(Gesture::Wire { .. }))
            || self.canvas.pending.is_some()
            || matches!(self.dialog, Some(Dialog::Connection(_)))
        {
            emphasis.active = false;
        }
        let p = self.store.snapshot();
        let sid = self.current.clone();
        let Some(system) = p.system(&sid) else {
            return;
        };
        let (area, response) = ui.allocate_exact_size(
            ui.available_size().max(vec2(50.0, 50.0)),
            Sense::click_and_drag(),
        );
        let painter = ui.painter_at(area);
        painter.rect_filled(area, 0, Color32::from_rgb(14, 18, 24));
        let mut positions = auto_layout(&p, &sid);
        if let Some(saved) = p.layout.get(&sid) {
            positions.extend(saved.iter().map(|(k, v)| (k.clone(), *v)));
        }
        if let Some(Gesture::Move { id, current, .. }) = &self.canvas.gesture {
            positions.insert(id.clone(), *current);
        }
        // Geometry is derived from actual endpoints, not inferred process order.
        // Wire previews do not move ports or mutate the project.
        let mut detail_scene = self
            .canvas
            .frozen_scene
            .clone()
            .unwrap_or_else(|| Scene::new(&p, &sid, &positions));
        if let Some(Gesture::Move { id, current, .. }) = &self.canvas.gesture {
            detail_scene.move_card(id, *current);
        }
        let scene = if compact {
            overview::compact_scene(detail_scene.clone())
        } else {
            detail_scene.clone()
        };
        let local_paths = overview::links_for(&p, &sid, &scene, compact);
        let mut bounds = scene.bounds;
        for link in &local_paths {
            bounds = bounds.union(link.path.bounds.expand(20.0));
        }
        if self.canvas.fit_requested {
            self.canvas.zoom = ((area.width() - 32.0) / bounds.width())
                .min((area.height() - 32.0) / bounds.height())
                .clamp(0.2, 1.0);
            self.canvas.pan = area.size() * 0.5 - bounds.center().to_vec2() * self.canvas.zoom;
            self.canvas.fit_requested = false;
        }
        let pointer = ui.input(|i| i.pointer.interact_pos());
        if ui.is_enabled() && response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.01 {
                if let Some(cursor) = pointer {
                    let world = (cursor - area.min - self.canvas.pan) / self.canvas.zoom;
                    self.canvas.zoom = (self.canvas.zoom * (scroll * 0.002).exp()).clamp(0.2, 2.5);
                    self.canvas.pan = cursor - area.min - world * self.canvas.zoom;
                }
            }
        }
        let z = self.canvas.zoom;
        let pan = self.canvas.pan;
        let screen = |point: Pos2| area.min + pan + point.to_vec2() * z;
        let screen_rect = |rect: Rect| Rect::from_min_max(screen(rect.min), screen(rect.max));
        let grid = 40.0 * z;
        if grid >= 10.0 {
            let origin = area.min + pan;
            let mut x = area.left() + (origin.x - area.left()).rem_euclid(grid);
            while x < area.right() {
                let mut y = area.top() + (origin.y - area.top()).rem_euclid(grid);
                while y < area.bottom() {
                    painter.circle_filled(pos2(x, y), 1.0, Color32::from_rgb(33, 39, 48));
                    y += grid;
                }
                x += grid;
            }
        }
        if let Some(frame) = scene.frame {
            let rect = screen_rect(frame);
            painter.rect_filled(rect, 8, Color32::from_rgba_unmultiplied(24, 31, 40, 210));
            painter.rect_stroke(
                rect,
                8,
                Stroke::new(1.5_f32, Color32::from_rgb(70, 93, 111)),
                StrokeKind::Inside,
            );
            if let Some((_, owner)) = p.owner(&sid) {
                painter.text(
                    rect.min + vec2(22.0, 64.0) * z,
                    Align2::LEFT_TOP,
                    format!("{} — internal system", owner.name),
                    FontId::proportional(18.0 * z),
                    Color32::WHITE,
                );
            }
        }
        let mut paths = Vec::with_capacity(local_paths.len());
        let time = ui.input(|i| i.time);
        let animate =
            ui.is_enabled() && ui.input(|i| i.focused && !i.viewport().minimized.unwrap_or(false));
        let mut repaint = false;
        for link in local_paths {
            let path = link.path.screen(area.min, pan, z);
            let selected = link.selected(&self.selected);
            let emphasized = link.focused_count(&emphasis) > 0;
            if path.bounds.expand(15.0).intersects(area) {
                let color = if selected {
                    ACCENT
                } else if emphasized {
                    Color32::from_rgb(111, 129, 151)
                } else {
                    Color32::from_rgb(46, 56, 70)
                };
                painter.add(egui::Shape::line(
                    path.points.clone(),
                    Stroke::new(
                        if selected {
                            3.0_f32
                        } else if emphasized {
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
                if animate
                    && path.length > 0.01
                    && link.animate(motion, &self.selected, &emphasis, &p, &sid)
                {
                    motion::paint(&painter, &path, time, &link.edge.id);
                    repaint = true;
                }
                if emphasized {
                    let label = link.caption(&emphasis);
                    let mid = path.at(path.length * 0.5).0;
                    let galley = painter.layout_no_wrap(
                        if compact { label } else { short(&label, 32) },
                        FontId::proportional((11.0 * z).max(9.0)),
                        MUTED,
                    );
                    let rect = Rect::from_center_size(
                        mid - vec2(0.0, 11.0),
                        galley.size() + vec2(10.0, 4.0),
                    );
                    painter.rect_filled(rect, 3, Color32::from_rgb(14, 18, 24));
                    painter.galley(rect.min + vec2(5.0, 2.0), galley, MUTED);
                }
            }
            paths.push((link.selection, path));
        }
        if repaint {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }
        for card in &scene.cards {
            let Some((_, node)) = p.node(&card.id) else {
                continue;
            };
            let rect = screen_rect(card.rect);
            let selected = self.selected == Selection::Node(node.id.clone())
                || matches!(&self.selected,Selection::Port(ep) if ep.node.as_ref()==Some(&node.id));
            painter.rect_filled(
                rect,
                7,
                if emphasis.node(&node.id) {
                    Color32::from_rgb(30, 37, 47)
                } else {
                    Color32::from_rgb(23, 29, 37)
                },
            );
            painter.rect_stroke(
                rect,
                7,
                Stroke::new(
                    if selected { 2.0_f32 } else { 1.0_f32 },
                    if selected {
                        ACCENT
                    } else {
                        Color32::from_rgb(64, 76, 93)
                    },
                ),
                StrokeKind::Inside,
            );
            let clipped = painter.with_clip_rect(rect.shrink(8.0 * z).intersect(area));
            let header = screen(pos2(card.rect.left(), card.header_y));
            clipped.text(
                header + vec2(15.0, 15.0) * z,
                Align2::LEFT_TOP,
                short(&node.name, 30),
                FontId::proportional(16.0 * z),
                Color32::WHITE,
            );
            clipped.text(
                header + vec2(15.0, 39.0) * z,
                Align2::LEFT_TOP,
                format!(
                    "{}{}",
                    node.kind.label(),
                    if node.child.is_some() {
                        "  ·  Enter >"
                    } else {
                        ""
                    }
                ),
                FontId::proportional(11.0 * z),
                ACCENT,
            );
            clipped.text(
                header + vec2(15.0, 57.0) * z,
                Align2::LEFT_TOP,
                if compact {
                    format!(
                        "{} ports · {} connections",
                        node.ports.len(),
                        system
                            .edges
                            .iter()
                            .filter(|e| e.from.node.as_ref() == Some(&node.id)
                                || e.to.node.as_ref() == Some(&node.id))
                            .count()
                    )
                } else {
                    short(&node.purpose.replace('\n', " "), 42)
                },
                FontId::proportional(11.0 * z),
                MUTED,
            );
        }
        let dragging = match &self.canvas.gesture {
            Some(Gesture::Wire { port, .. }) => Some(port.clone()),
            _ => self.canvas.pending.clone(),
        };
        let hovered_port = pointer.filter(|pos| area.contains(*pos)).and_then(|pos| {
            scene
                .ports
                .iter()
                .filter(|r| {
                    (!compact || r.boundary) && screen(r.point).distance(pos) < (11.0 * z).max(9.0)
                })
                .min_by(|a, b| {
                    screen(a.point)
                        .distance(pos)
                        .total_cmp(&screen(b.point).distance(pos))
                })
        });
        #[cfg(test)]
        {
            self.canvas.port_positions = scene
                .ports
                .iter()
                .map(|a| (a.endpoint.port.clone(), screen(a.point)))
                .collect();
        }
        for anchor in &scene.ports {
            if compact && !anchor.boundary {
                continue;
            }
            let point = screen(anchor.point);
            let compatible = dragging
                .as_ref()
                .is_some_and(|from| pair(&p, &sid, from, &anchor.endpoint).is_some());
            let hover = hovered_port.is_some_and(|q| q.endpoint == anchor.endpoint);
            let color = if hover || compatible {
                ACCENT
            } else if emphasis.port(&anchor.endpoint.port) {
                MUTED
            } else {
                Color32::from_rgb(63, 74, 88)
            };
            let radius = (5.0 * z).max(4.0);
            if anchor.direction == Direction::Out {
                painter.circle_filled(point, radius, color);
            } else {
                painter.circle_filled(point, radius, Color32::from_rgb(14, 18, 24));
                painter.circle_stroke(point, radius, Stroke::new(1.5_f32, color));
            }
            if hover || compatible {
                painter.circle_stroke(point, (9.0 * z).max(8.0), Stroke::new(1.0_f32, color));
            }
            if hover || compatible || emphasis.port(&anchor.endpoint.port) {
                port_labels(&painter, anchor, screen_rect(anchor.label_rect), z);
            }
            if hover {
                response.clone().on_hover_text(format!(
                    "{} · {}\n{}{}",
                    anchor.direction.label(),
                    anchor.name,
                    anchor.contract,
                    if anchor.boundary {
                        format!("\n{}", anchor.external)
                    } else {
                        String::new()
                    }
                ));
            }
        }
        if let (Some(from), Some(cursor)) = (dragging.as_ref(), pointer) {
            if let Some(start) = scene.port(from) {
                let end_normal = hovered_port.map(|p| p.normal).unwrap_or(-start.normal);
                let end = hovered_port.map(|p| screen(p.point)).unwrap_or(cursor);
                let path = if start.direction == Direction::Out {
                    Path::between(screen(start.point), start.normal, end, end_normal)
                } else {
                    Path::between(end, end_normal, screen(start.point), start.normal)
                };
                painter.add(egui::Shape::line(path.points, Stroke::new(2.5_f32, ACCENT)));
            }
        }
        if system.nodes.is_empty() {
            painter.text(
                area.center(),
                Align2::CENTER_CENTER,
                if scene.frame.is_some() {
                    "This component has no internal components yet."
                } else {
                    "Start with a purpose, then add only the components you need."
                },
                FontId::proportional(16.0),
                MUTED,
            );
        }
        if !ui.is_enabled() {
            return;
        }
        let Some(cursor) = pointer else {
            if ui.input(|i| !i.pointer.any_down()) {
                self.canvas.gesture = None;
            }
            return;
        };
        if ui.input(|i| !i.pointer.any_down() && !i.pointer.any_released()) {
            self.canvas.gesture = None;
        }
        if ui.input(|i| i.pointer.button_pressed(PointerButton::Secondary)) {
            if let Some(port) = hovered_port {
                self.trace_visit(
                    sid.clone(),
                    if port.boundary {
                        Selection::Boundary(port.endpoint.port.clone())
                    } else {
                        Selection::Port(port.endpoint.clone())
                    },
                );
            }
        }
        let pressed = ui.input(|i| i.pointer.button_pressed(PointerButton::Primary));
        let middle = ui.input(|i| i.pointer.button_pressed(PointerButton::Middle));
        if (pressed || middle) && area.contains(cursor) {
            if middle {
                self.canvas.gesture = Some(Gesture::Pan {
                    start: cursor,
                    initial: self.canvas.pan,
                });
            } else if let Some(port) = hovered_port {
                self.selected = if port.boundary {
                    Selection::Boundary(port.endpoint.port.clone())
                } else {
                    Selection::Port(port.endpoint.clone())
                };
                if !compact {
                    self.canvas.frozen_scene = Some(detail_scene.clone());
                    self.canvas.gesture = Some(Gesture::Wire {
                        port: port.endpoint.clone(),
                        start: cursor,
                    });
                }
            } else if let Some(card) = scene
                .cards
                .iter()
                .rev()
                .find(|c| screen_rect(c.rect).contains(cursor))
            {
                self.canvas.pending = None;
                self.canvas.frozen_scene = Some(detail_scene.clone());
                self.selected = Selection::Node(card.id.clone());
                self.canvas.gesture = Some(Gesture::Move {
                    id: card.id.clone(),
                    start: cursor,
                    initial: card.position,
                    current: card.position,
                });
            } else if let Some((id, _)) = paths
                .iter()
                .filter(|(_, p)| p.distance(cursor) < 7.0)
                .min_by(|(_, a), (_, b)| a.distance(cursor).total_cmp(&b.distance(cursor)))
            {
                self.canvas.cancel();
                self.selected = id.clone();
                self.canvas.gesture = Some(Gesture::Edge(id.clone()));
            } else {
                self.selected = Selection::None;
                self.canvas.pending = None;
                self.canvas.gesture = Some(Gesture::Pan {
                    start: cursor,
                    initial: self.canvas.pan,
                });
            }
        }
        if ui.input(|i| i.pointer.any_down()) {
            match &mut self.canvas.gesture {
                Some(Gesture::Move {
                    start,
                    initial,
                    current,
                    ..
                }) => {
                    let delta = (cursor - *start) / z;
                    *current = Position {
                        x: (initial.x + delta.x as f64).max(0.0),
                        y: (initial.y + delta.y as f64).max(0.0),
                    };
                    ui.ctx().request_repaint();
                }
                Some(Gesture::Pan { start, initial }) => {
                    self.canvas.pan = *initial + (cursor - *start);
                    ui.ctx().request_repaint();
                }
                Some(Gesture::Wire { .. }) => ui.ctx().request_repaint(),
                _ => {}
            }
        }
        if ui.input(|i| i.pointer.any_released()) {
            match self.canvas.gesture.take() {
                Some(Gesture::Move {
                    id,
                    start,
                    initial,
                    current,
                }) => {
                    if start.distance(cursor) < 4.0
                        && ui.input(|i| i.pointer.button_double_clicked(PointerButton::Primary))
                    {
                        if let Some(child) = p.node(&id).and_then(|(_, n)| n.child.clone()) {
                            self.navigate(child);
                        } else if let Some((_, node)) = p.node(&id) {
                            self.dialog = Some(Dialog::Node(node.clone()));
                        }
                    } else if initial != current {
                        let candidate = edit::candidate(&p, |q| {
                            q.layout
                                .entry(sid.clone())
                                .or_insert_with(BTreeMap::new)
                                .insert(id, current);
                            Ok(())
                        });
                        self.publish("Move component", candidate);
                    }
                }
                Some(Gesture::Wire { port, start }) => {
                    if let Some(target) = hovered_port {
                        let from = if start.distance(cursor) < 5.0 {
                            self.canvas.pending.clone().unwrap_or_else(|| port.clone())
                        } else {
                            port.clone()
                        };
                        if let Some((from, to)) = pair(&p, &sid, &from, &target.endpoint) {
                            self.canvas.pending = None;
                            self.dialog = Some(Dialog::Connection(ConnectionDialog::new(
                                &p,
                                &sid,
                                Some((from, to)),
                                None,
                            )));
                        } else if start.distance(cursor) < 5.0 {
                            self.canvas.pending = Some(target.endpoint.clone());
                        } else {
                            self.canvas.pending = None;
                        }
                    } else {
                        self.canvas.pending = None;
                    }
                }
                Some(Gesture::Edge(Selection::Edge(id))) => {
                    if ui.input(|i| i.pointer.button_double_clicked(PointerButton::Primary)) {
                        self.show_exact(&id, true);
                    }
                }
                _ => {}
            }
        }
        if !self.canvas.has_gesture() {
            self.canvas.frozen_scene = None;
        }
        self.canvas_session.normalize(&self.selected);
    }
}

impl Designer {
    fn canvas_controls(&mut self, ui: &mut egui::Ui) -> motion::Motion {
        let mut motion = motion::Motion::All;
        let before = self.canvas_session.view;
        let mut arrange = None;
        let mut back = false;
        ui.horizontal_wrapped(|ui| {
            ui.label("View");
            egui::ComboBox::from_id_salt("canvas-view")
                .selected_text(if before == View::Detail {
                    "Detail"
                } else {
                    "Overview"
                })
                .width(86.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.canvas_session.view, View::Detail, "Detail");
                    ui.selectable_value(&mut self.canvas_session.view, View::Overview, "Overview");
                });
            ui.label("Focus");
            egui::ComboBox::from_id_salt("canvas-focus")
                .selected_text(self.canvas_session.focus.label())
                .width(90.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.canvas_session.focus, Focus::Off, "Off");
                    ui.add_enabled_ui(self.selected != Selection::None, |ui| {
                        ui.selectable_value(
                            &mut self.canvas_session.focus,
                            Focus::Selection,
                            "Selection",
                        );
                    });
                    ui.add_enabled_ui(matches!(self.selected, Selection::Node(_)), |ui| {
                        ui.selectable_value(
                            &mut self.canvas_session.focus,
                            Focus::Incoming,
                            "Incoming",
                        );
                        ui.selectable_value(
                            &mut self.canvas_session.focus,
                            Focus::Outgoing,
                            "Outgoing",
                        );
                    });
                });
            ui.menu_button("Arrange", |ui| {
                if ui.button("By connections · Left to right").clicked() {
                    arrange = Some(Some(arrangement::Axis::Horizontal));
                    ui.close();
                }
                if ui.button("By connections · Top to bottom").clicked() {
                    arrange = Some(Some(arrangement::Axis::Vertical));
                    ui.close();
                }
                ui.separator();
                if ui.button("Grid").clicked() {
                    arrange = Some(None);
                    ui.close();
                }
                ui.small("Current level only; one undoable layout edit. Not execution order.");
            });
            motion = motion::Motion::controls(ui);
            if ui.button("Fit").clicked() {
                self.canvas.fit_requested = true;
            }
            if ui.small_button("−").clicked() {
                self.canvas.zoom = (self.canvas.zoom / 1.15).max(0.2);
            }
            if ui.small_button("+").clicked() {
                self.canvas.zoom = (self.canvas.zoom * 1.15).min(2.5);
            }
            ui.label(format!("{:.0}%", self.canvas.zoom * 100.0));
            if ui
                .add_enabled(
                    !self.canvas_session.history.is_empty(),
                    egui::Button::new("Back trace"),
                )
                .clicked()
            {
                back = true;
            }
        });
        if before != self.canvas_session.view {
            self.canvas.cancel();
        }
        if back {
            self.trace_back();
        }
        if let Some(axis) = arrange {
            arrangement::apply(self, axis);
        }
        self.canvas_session.normalize(&self.selected);
        let p = self.store.project();
        let counts = p
            .system(&self.current)
            .map(|s| (s.nodes.len(), s.edges.len()))
            .unwrap_or_default();
        let focus =
            focus::Emphasis::new(p, &self.current, &self.selected, self.canvas_session.focus);
        let name = p
            .owner(&self.current)
            .map(|(_, n)| n.name.as_str())
            .unwrap_or(&p.name);
        let mut title = format!(
            "{name} · {} components · {} connections",
            counts.0, counts.1
        );
        if self.canvas_session.view == View::Overview {
            let strokes = p
                .system(&self.current)
                .map(|s| overview::groups(s).len())
                .unwrap_or(0);
            title.push_str(&format!(" · {strokes} summary strokes; all edges retained"));
        }
        if counts.0 > 8 {
            title.push_str(" · Consider meaningful decomposition");
        }
        ui.add(egui::Label::new(egui::RichText::new(&title).strong()).truncate())
            .on_hover_text(title);
        // Fixed status row keeps mode changes from shifting the canvas origin.
        let status = if focus.active {
            format!(
                "Focus: {} · {} — {} emphasized / {} muted",
                focus::description(p, &self.current, &self.selected),
                self.canvas_session.focus.label(),
                focus.edges.len(),
                counts.1.saturating_sub(focus.edges.len())
            )
        } else {
            "Focus off · Hollow receives / filled produces · Right-click a port to inspect it"
                .into()
        };
        ui.add(egui::Label::new(status).truncate());
        ui.small(if motion != motion::Motion::Off {
            "Direction preview — not live execution. Muted focus context never animates."
        } else {
            "Lights off. Arrowheads show direction; layouts are not execution order."
        });
        motion
    }
}
