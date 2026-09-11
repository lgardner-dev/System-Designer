use super::*;
use crate::edit;
use egui::{
    Align2, Color32, FontId, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use std::collections::BTreeMap;

mod geometry;
mod motion;
pub(super) mod overview;
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
    Edge(String),
}
pub(super) struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub fit_requested: bool,
    gesture: Option<Gesture>,
    pending: Option<Endpoint>,
    #[cfg(test)]
    port_positions: BTreeMap<String, Pos2>,
    #[cfg(test)]
    summary_positions: BTreeMap<String, Pos2>,
}
impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            fit_requested: true,
            gesture: None,
            pending: None,
            #[cfg(test)]
            port_positions: BTreeMap::new(),
            #[cfg(test)]
            summary_positions: BTreeMap::new(),
        }
    }
}
impl CanvasState {
    pub fn cancel(&mut self) {
        self.gesture = None;
        self.pending = None;
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
        let view = overview::controls(self, ui);
        let motion = motion::Motion::controls(ui);
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
        let scene = Scene::new(&p, &sid, &positions);
        let scene = if view.compact {
            overview::compact_scene(scene)
        } else {
            scene
        };
        let mut bounds = scene.bounds;
        let local_paths = overview::links_for(&p, &sid, &scene, view.compact);
        let member_map: BTreeMap<String, Vec<String>> = local_paths
            .iter()
            .map(|l| (l.edge.id.clone(), l.members.clone()))
            .collect();
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
        let mut label_hits = Vec::new();
        let time = ui.input(|i| i.time);
        let animate = ui.is_enabled() && ui.input(|i| i.focused);
        let mut repaint = false;
        for link in local_paths {
            let edge = link.edge;
            let path = link.path.screen(area.min, pan, z);
            let selected = self.selected == Selection::Edge(edge.id.clone())
                || (view.compact && view.group == link.members);
            if path.bounds.expand(15.0).intersects(area) {
                painter.add(egui::Shape::line(
                    path.points.clone(),
                    Stroke::new(
                        if selected { 3.0_f32 } else { 1.8_f32 },
                        if selected {
                            ACCENT
                        } else {
                            Color32::from_rgb(111, 129, 151)
                        },
                    ),
                ));
                // Destination tangent supports inputs on every side.
                if path.length > 8.0 {
                    let (head, tangent) = path.at((path.length - 7.0).max(0.0));
                    painter.arrow(
                        head - tangent * 13.0,
                        tangent * 13.0,
                        Stroke::new(1.5_f32, if selected { ACCENT } else { MUTED }),
                    );
                }
                if animate
                    && path.length > 0.01
                    && (motion.includes(edge, &self.selected)
                        || (view.compact && selected && motion == motion::Motion::Selected))
                {
                    motion::paint(&painter, &path, time, &edge.id);
                    repaint = true;
                }
                let label = link.label;
                let mid = path.at(path.length * 0.5).0;
                let galley = painter.layout_no_wrap(
                    short(&label, 32),
                    FontId::proportional((11.0 * z).max(9.0)),
                    MUTED,
                );
                let rect =
                    Rect::from_center_size(mid - vec2(0.0, 11.0), galley.size() + vec2(10.0, 4.0));
                label_hits.push((edge.id.clone(), rect));
                painter.rect_filled(rect, 3, Color32::from_rgb(14, 18, 24));
                painter.galley(rect.min + vec2(5.0, 2.0), galley, MUTED);
            }
            paths.push((edge.id.clone(), path));
        }
        if repaint {
            ui.ctx().request_repaint_after(Duration::from_millis(16));
        }
        for card in &scene.cards {
            let Some((_, node)) = p.node(&card.id) else {
                continue;
            };
            let rect = screen_rect(card.rect);
            let selected = self.selected == Selection::Node(node.id.clone());
            painter.rect_filled(rect, 7, Color32::from_rgb(30, 37, 47));
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
            if view.compact {
                let galley = clipped.layout(
                    node.name.clone(),
                    FontId::proportional((16.0 * z).max(12.0)),
                    Color32::WHITE,
                    (rect.width() - 24.0 * z).max(30.0),
                );
                let title_clip = Rect::from_min_max(
                    rect.min + vec2(12.0, 10.0) * z,
                    rect.max - vec2(12.0, 39.0) * z,
                );
                clipped.with_clip_rect(title_clip.intersect(area)).galley(
                    title_clip.min,
                    galley,
                    Color32::WHITE,
                );
                clipped.text(
                    rect.left_bottom() + vec2(12.0, -37.0) * z,
                    Align2::LEFT_TOP,
                    if node.child.is_some() {
                        "Has internals · double-click to enter"
                    } else {
                        "Leaf component"
                    },
                    FontId::proportional((10.0 * z).max(8.0)),
                    ACCENT,
                );
            } else {
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
                    short(&node.purpose.replace('\n', " "), 42),
                    FontId::proportional(11.0 * z),
                    MUTED,
                );
            }
        }
        if view.compact {
            for card in &scene.cards {
                if let Some((_, node)) = p.node(&card.id) {
                    let count = system
                        .edges
                        .iter()
                        .filter(|e| {
                            e.from.node.as_ref() == Some(&node.id)
                                || e.to.node.as_ref() == Some(&node.id)
                        })
                        .count();
                    painter.text(
                        screen(card.rect.left_bottom()) + vec2(15.0, -20.0) * z,
                        Align2::LEFT_TOP,
                        format!("{} ports · {count} connections", node.ports.len()),
                        FontId::proportional(10.0 * z),
                        MUTED,
                    );
                }
            }
        }
        let dragging = match &self.canvas.gesture {
            Some(Gesture::Wire { port, .. }) => Some(port.clone()),
            _ => self.canvas.pending.clone(),
        };
        let hovered_port = pointer.filter(|pos| area.contains(*pos)).and_then(|pos| {
            scene
                .ports
                .iter()
                .filter(|r| !view.compact || r.boundary)
                .filter(|r| screen(r.point).distance(pos) < (11.0 * z).max(9.0))
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
            if view.compact && !anchor.boundary {
                continue;
            }
            let point = screen(anchor.point);
            let compatible = dragging
                .as_ref()
                .is_some_and(|from| pair(&p, &sid, from, &anchor.endpoint).is_some());
            let hover = hovered_port.is_some_and(|q| q.endpoint == anchor.endpoint);
            let color = if hover || compatible { ACCENT } else { MUTED };
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
            port_labels(&painter, anchor, screen_rect(anchor.label_rect), z);
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
        #[cfg(test)]
        {
            self.canvas.summary_positions = label_hits
                .iter()
                .map(|(id, rect)| (id.clone(), rect.center()))
                .collect();
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
        let pressed = ui.input(|i| i.pointer.button_pressed(PointerButton::Primary));
        let middle = ui.input(|i| i.pointer.button_pressed(PointerButton::Middle));
        if (pressed || middle) && area.contains(cursor) {
            if middle {
                self.canvas.gesture = Some(Gesture::Pan {
                    start: cursor,
                    initial: self.canvas.pan,
                });
            } else if let Some(port) = hovered_port {
                if port.boundary {
                    self.selected = Selection::Boundary(port.endpoint.port.clone());
                }
                if view.compact {
                    self.canvas.gesture = None;
                } else {
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
                overview::clear(ui.ctx());
                self.selected = Selection::Node(card.id.clone());
                self.canvas.gesture = Some(Gesture::Move {
                    id: card.id.clone(),
                    start: cursor,
                    initial: card.position,
                    current: card.position,
                });
            } else if let Some((id, _)) = label_hits.iter().find(|(_, rect)| rect.contains(cursor))
            {
                if view.compact {
                    overview::select(self, ui.ctx(), &member_map[id]);
                } else {
                    self.selected = Selection::Edge(id.clone());
                }
                self.canvas.gesture = Some(Gesture::Edge(id.clone()));
            } else if let Some((id, _)) = paths
                .iter()
                .filter(|(_, p)| p.distance(cursor) < 7.0)
                .min_by(|(_, a), (_, b)| a.distance(cursor).total_cmp(&b.distance(cursor)))
            {
                if view.compact {
                    overview::select(self, ui.ctx(), &member_map[id]);
                } else {
                    self.selected = Selection::Edge(id.clone());
                }
                self.canvas.gesture = Some(Gesture::Edge(id.clone()));
            } else {
                overview::clear(ui.ctx());
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
                Some(Gesture::Edge(id)) => {
                    if ui.input(|i| i.pointer.button_double_clicked(PointerButton::Primary)) {
                        if view.compact
                            && member_map.get(&id).is_some_and(|members| members.len() > 1)
                        {
                            overview::expand(ui.ctx());
                            self.canvas.cancel();
                            return;
                        }
                        self.dialog = Some(Dialog::Connection(ConnectionDialog::new(
                            &p,
                            &sid,
                            None,
                            Some(&id),
                        )));
                    }
                }
                _ => {}
            }
        }
    }
}
