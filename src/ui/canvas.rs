use super::*;
use crate::edit;
use egui::{
    Align2, Color32, FontId, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use std::collections::BTreeMap;
const WIDTH: f32 = 300.0;
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
}
impl Default for CanvasState {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            fit_requested: true,
            gesture: None,
            pending: None,
        }
    }
}
impl CanvasState {
    pub fn cancel(&mut self) {
        self.gesture = None;
        self.pending = None;
    }
}
#[derive(Clone)]
struct PortHit {
    endpoint: Endpoint,
    point: Pos2,
    direction: Direction,
    name: String,
    contract: String,
    boundary: bool,
    external: String,
}
struct NodeHit {
    id: String,
    rect: Rect,
    position: Position,
}
fn point(rect: Rect, pan: Vec2, zoom: f32, x: f32, y: f32) -> Pos2 {
    rect.min + pan + vec2(x, y) * zoom
}
fn controls(a: Pos2, b: Pos2) -> [Pos2; 4] {
    let d = ((b.x - a.x).abs() * 0.45).clamp(45.0, 220.0);
    [a, a + vec2(d, 0.0), b - vec2(d, 0.0), b]
}
fn sample(c: [Pos2; 4], t: f32) -> Pos2 {
    let u = 1.0 - t;
    pos2(
        u * u * u * c[0].x
            + 3.0 * u * u * t * c[1].x
            + 3.0 * u * t * t * c[2].x
            + t * t * t * c[3].x,
        u * u * u * c[0].y
            + 3.0 * u * u * t * c[1].y
            + 3.0 * u * t * t * c[2].y
            + t * t * t * c[3].y,
    )
}
fn distance_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let t = if ab.length_sq() > 0.0 {
        ((p - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    p.distance(a + ab * t)
}
fn curve_distance(p: Pos2, c: [Pos2; 4]) -> f32 {
    let mut d = f32::INFINITY;
    let mut a = c[0];
    for i in 1..=32 {
        let b = sample(c, i as f32 / 32.0);
        d = d.min(distance_segment(p, a, b));
        a = b;
    }
    d
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
impl Designer {
    pub(super) fn canvas_view(&mut self, ui: &mut egui::Ui) {
        let p = self.store.snapshot();
        let sid = self.current.clone();
        let Some(s) = p.system(&sid) else {
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
        let mut extent = Rect::from_min_max(pos2(40.0, 40.0), pos2(900.0, 500.0));
        for n in &s.nodes {
            if let Some(pos) = positions.get(&n.id) {
                extent = extent.union(Rect::from_min_size(
                    pos2(pos.x as f32, pos.y as f32),
                    vec2(WIDTH, node_height(n) as f32),
                ));
            }
        }
        let boundary = p.owner(&sid);
        let boundary_rows = p
            .boundary(&sid)
            .iter()
            .filter(|r| r.direction == Direction::In)
            .count()
            .max(
                p.boundary(&sid)
                    .iter()
                    .filter(|r| r.direction == Direction::Out)
                    .count(),
            );
        let frame = Rect::from_min_max(
            pos2(40.0, 40.0),
            pos2(
                extent.max.x + 240.0,
                (extent.max.y + 100.0).max(180.0 + boundary_rows as f32 * 70.0),
            ),
        );
        let mut bounds = if boundary.is_some() {
            frame.expand2(vec2(80.0, 30.0))
        } else {
            extent.expand(50.0)
        };
        if s.nodes.is_empty() && boundary.is_none() {
            bounds = Rect::from_min_max(Pos2::ZERO, pos2(900.0, 550.0));
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
        let world = |x: f32, y: f32| point(area, pan, z, x, y);
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
        let mut ports = vec![];
        let mut nodes = vec![];
        if let Some((_, owner)) = boundary {
            let screen = Rect::from_min_max(
                world(frame.min.x, frame.min.y),
                world(frame.max.x, frame.max.y),
            );
            painter.rect_filled(screen, 8, Color32::from_rgba_unmultiplied(24, 31, 40, 210));
            painter.rect_stroke(
                screen,
                8,
                Stroke::new(1.5, Color32::from_rgb(70, 93, 111)),
                StrokeKind::Inside,
            );
            painter.text(
                screen.min + vec2(22.0, 22.0) * z,
                Align2::LEFT_TOP,
                format!("{} — internal system", owner.name),
                FontId::proportional(18.0 * z),
                Color32::WHITE,
            );
            let external = p.external_connections(&sid);
            for dir in [Direction::In, Direction::Out] {
                let list: Vec<_> = owner.ports.iter().filter(|r| r.direction == dir).collect();
                for (i, r) in list.iter().enumerate() {
                    let x = if dir == Direction::In {
                        frame.min.x
                    } else {
                        frame.max.x
                    };
                    let y = frame.min.y + 100.0 + i as f32 * 70.0;
                    let names = external
                        .iter()
                        .find(|v| v["port"].as_str() == Some(&r.id))
                        .and_then(|v| v["links"].as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v["name"].as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default();
                    ports.push(PortHit {
                        endpoint: Endpoint {
                            node: None,
                            port: r.id.clone(),
                        },
                        point: world(x, y),
                        direction: dir.opposite(),
                        name: r.name.clone(),
                        contract: r
                            .contract
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "Unassigned".into()),
                        boundary: true,
                        external: if names.is_empty() {
                            "No external connection".into()
                        } else {
                            format!(
                                "{} {names}",
                                if dir == Direction::In { "from" } else { "to" }
                            )
                        },
                    });
                }
            }
        }
        for n in &s.nodes {
            let pos = positions.get(&n.id).copied().unwrap_or_default();
            let rect = Rect::from_min_size(
                world(pos.x as f32, pos.y as f32),
                vec2(WIDTH, node_height(n) as f32) * z,
            );
            nodes.push(NodeHit {
                id: n.id.clone(),
                rect,
                position: pos,
            });
            for dir in [Direction::In, Direction::Out] {
                for (i, r) in n.ports.iter().filter(|r| r.direction == dir).enumerate() {
                    ports.push(PortHit {
                        endpoint: Endpoint {
                            node: Some(n.id.clone()),
                            port: r.id.clone(),
                        },
                        point: pos2(
                            if dir == Direction::In {
                                rect.left()
                            } else {
                                rect.right()
                            },
                            rect.top() + (88.0 + i as f32 * 38.0) * z,
                        ),
                        direction: dir,
                        name: r.name.clone(),
                        contract: r
                            .contract
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "Unassigned".into()),
                        boundary: false,
                        external: String::new(),
                    });
                }
            }
        }
        let find_port = |ep: &Endpoint| ports.iter().find(|r| r.endpoint == *ep).map(|r| r.point);
        let mut curves = vec![];
        for e in &s.edges {
            if let (Some(a), Some(b)) = (find_port(&e.from), find_port(&e.to)) {
                let c = controls(a, b);
                let selected = self.selected == Selection::Edge(e.id.clone());
                painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                    c,
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(
                        if selected { 3.0 } else { 1.8 },
                        if selected {
                            ACCENT
                        } else {
                            Color32::from_rgb(111, 129, 151)
                        },
                    ),
                ));
                painter.arrow(
                    b - vec2(11.0, 0.0),
                    vec2(11.0, 0.0),
                    Stroke::new(1.5, if selected { ACCENT } else { MUTED }),
                );
                let label = e
                    .label
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| {
                        p.port(&sid, &e.from)
                            .and_then(|r| r.contract.as_ref())
                            .map(ToString::to_string)
                            .unwrap_or_default()
                    });
                let mid = sample(c, 0.5);
                let galley = painter.layout_no_wrap(
                    short(&label, 32),
                    FontId::proportional((11.0 * z).max(9.0)),
                    MUTED,
                );
                let rect =
                    Rect::from_center_size(mid - vec2(0.0, 9.0), galley.size() + vec2(10.0, 4.0));
                painter.rect_filled(rect, 3, Color32::from_rgb(14, 18, 24));
                painter.galley(rect.min + vec2(5.0, 2.0), galley, MUTED);
                curves.push((e.id.clone(), c));
            }
        }
        for h in &nodes {
            let Some((_, n)) = p.node(&h.id) else {
                continue;
            };
            let selected = self.selected == Selection::Node(n.id.clone());
            painter.rect_filled(h.rect, 7, Color32::from_rgb(30, 37, 47));
            painter.rect_stroke(
                h.rect,
                7,
                Stroke::new(
                    if selected { 2.0 } else { 1.0 },
                    if selected {
                        ACCENT
                    } else {
                        Color32::from_rgb(64, 76, 93)
                    },
                ),
                StrokeKind::Inside,
            );
            let clipped = painter.with_clip_rect(h.rect.shrink(8.0 * z));
            clipped.text(
                h.rect.min + vec2(15.0, 15.0) * z,
                Align2::LEFT_TOP,
                short(&n.name, 30),
                FontId::proportional(16.0 * z),
                Color32::WHITE,
            );
            clipped.text(
                h.rect.min + vec2(15.0, 39.0) * z,
                Align2::LEFT_TOP,
                format!(
                    "{}{}",
                    n.kind.label(),
                    if n.child.is_some() {
                        "  ·  Enter >"
                    } else {
                        ""
                    }
                ),
                FontId::proportional(11.0 * z),
                ACCENT,
            );
            clipped.text(
                h.rect.min + vec2(15.0, 57.0) * z,
                Align2::LEFT_TOP,
                short(&n.purpose.replace('\n', " "), 42),
                FontId::proportional(11.0 * z),
                MUTED,
            );
        }
        let dragging = match &self.canvas.gesture {
            Some(Gesture::Wire { port, .. }) => Some(port.clone()),
            _ => self.canvas.pending.clone(),
        };
        let hovered_port = pointer
            .filter(|pos| area.contains(*pos))
            .and_then(|pos| {
                ports
                    .iter()
                    .filter(|r| r.point.distance(pos) < (11.0 * z).max(9.0))
                    .min_by(|a, b| a.point.distance(pos).total_cmp(&b.point.distance(pos)))
            })
            .cloned();
        for r in &ports {
            let compatible = dragging
                .as_ref()
                .is_some_and(|from| pair(&p, &sid, from, &r.endpoint).is_some());
            let hover = hovered_port
                .as_ref()
                .is_some_and(|q| q.endpoint == r.endpoint);
            let color = if hover || compatible { ACCENT } else { MUTED };
            painter.circle_filled(r.point, (5.0 * z).max(4.0), color);
            if hover || compatible {
                painter.circle_stroke(r.point, (9.0 * z).max(8.0), Stroke::new(1.0, color));
            }
            let inward = if r.boundary {
                r.direction == Direction::Out
            } else {
                r.direction == Direction::In
            };
            let align = if inward {
                Align2::LEFT_CENTER
            } else {
                Align2::RIGHT_CENTER
            };
            let offset = vec2(if inward { 14.0 * z } else { -14.0 * z }, 0.0);
            painter.text(
                r.point + offset,
                align,
                short(&r.name, if r.boundary { 26 } else { 16 }),
                FontId::proportional(11.0 * z),
                Color32::from_rgb(217, 224, 234),
            );
            painter.text(
                r.point + offset + vec2(0.0, 14.0 * z),
                align,
                short(&r.contract, if r.boundary { 28 } else { 20 }),
                FontId::monospace(9.0 * z),
                MUTED,
            );
            if r.boundary {
                painter.text(
                    r.point + offset + vec2(0.0, 30.0 * z),
                    align,
                    short(&r.external, 28),
                    FontId::proportional(10.0 * z),
                    ACCENT,
                );
            }
        }
        if let (Some(from), Some(cursor)) = (dragging.as_ref(), pointer) {
            if let Some(start) = find_port(from) {
                let (a, b) = if p.effective_direction(&sid, from) == Some(Direction::Out) {
                    (start, cursor)
                } else {
                    (cursor, start)
                };
                painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
                    controls(a, b),
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(2.5, ACCENT),
                ));
            }
        }
        if s.nodes.is_empty() {
            let msg = if boundary.is_some() {
                "This component has no internal components yet."
            } else {
                "Start with a purpose, then add only the components you need."
            };
            painter.text(
                area.center(),
                Align2::CENTER_CENTER,
                msg,
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
        // A platform may drop the release event when the pointer leaves the window.
        // Do not leave an uncommitted gesture stuck active when no button is down.
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
            } else if let Some(r) = &hovered_port {
                if r.boundary {
                    self.selected = Selection::Boundary(r.endpoint.port.clone());
                }
                self.canvas.gesture = Some(Gesture::Wire {
                    port: r.endpoint.clone(),
                    start: cursor,
                });
            } else if let Some(n) = nodes.iter().rev().find(|n| n.rect.contains(cursor)) {
                self.selected = Selection::Node(n.id.clone());
                self.canvas.gesture = Some(Gesture::Move {
                    id: n.id.clone(),
                    start: cursor,
                    initial: n.position,
                    current: n.position,
                });
            } else if let Some((id, _)) = curves
                .iter()
                .filter(|(_, c)| curve_distance(cursor, *c) < 7.0)
                .min_by(|(_, a), (_, b)| {
                    curve_distance(cursor, *a).total_cmp(&curve_distance(cursor, *b))
                })
            {
                self.selected = Selection::Edge(id.clone());
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
                        } else if let Some((_, n)) = p.node(&id) {
                            self.dialog = Some(Dialog::Node(n.clone()));
                        }
                    } else if initial != current {
                        let q = edit::candidate(&p, |q| {
                            q.layout
                                .entry(sid.clone())
                                .or_insert_with(BTreeMap::new)
                                .insert(id, current);
                            Ok(())
                        });
                        self.publish("Move component", q);
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
                            self.canvas.pending = Some(target.endpoint);
                        } else {
                            self.canvas.pending = None;
                        }
                    } else {
                        self.canvas.pending = None;
                    }
                }
                Some(Gesture::Edge(id)) => {
                    if ui.input(|i| i.pointer.button_double_clicked(PointerButton::Primary)) {
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bezier_hit_testing() {
        let c = controls(pos2(0.0, 0.0), pos2(300.0, 100.0));
        assert!(curve_distance(sample(c, 0.5), c) < 0.01);
        assert!(curve_distance(pos2(100.0, 500.0), c) > 100.0);
    }
    #[test]
    fn text_truncation_is_unicode_safe() {
        assert_eq!(short("αβγδε", 3), "αβ…");
    }
}
#[cfg(test)]
mod interaction_tests {
    use super::*;
    fn app() -> Designer {
        let mut a = Designer::blank();
        let p = parse(include_str!("../../tests/fixtures/project.json")).expect("fixture");
        a.current = p.root.clone();
        a.store = crate::edit::Store::new(p).expect("store");
        a
    }
    fn frame(ctx: &egui::Context, a: &mut Designer, events: Vec<egui::Event>) -> Rect {
        let mut rect = Rect::NOTHING;
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1100.0, 800.0))),
            events,
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                rect = ui.available_rect_before_wrap();
                a.canvas_view(ui);
            });
        });
        rect
    }
    fn event(pos: Pos2, pressed: bool) -> egui::Event {
        egui::Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        }
    }
    #[test]
    fn real_pointer_drag_opens_explicit_contract_dialog() {
        let ctx = egui::Context::default();
        let mut a = app();
        let rect = frame(&ctx, &mut a, vec![]);
        let from = point(rect, a.canvas.pan, a.canvas.zoom, 820.0, 218.0);
        let to = point(rect, a.canvas.pan, a.canvas.zoom, 100.0, 218.0);
        frame(
            &ctx,
            &mut a,
            vec![egui::Event::PointerMoved(from), event(from, true)],
        );
        frame(&ctx, &mut a, vec![egui::Event::PointerMoved(to)]);
        frame(&ctx, &mut a, vec![event(to, false)]);
        match a.dialog.as_ref() {
            Some(Dialog::Connection(d)) => {
                assert!(d.selected.is_none());
                assert_eq!(d.from.as_ref().expect("from").port, "B.out");
                assert_eq!(d.to.as_ref().expect("to").port, "A.in");
            }
            _ => panic!("drag did not open contract dialog"),
        }
        assert_eq!(
            a.store.project().system("root").expect("root").edges.len(),
            2
        );
    }
    #[test]
    fn reverse_drag_is_supported() {
        let ctx = egui::Context::default();
        let mut a = app();
        let rect = frame(&ctx, &mut a, vec![]);
        let from = point(rect, a.canvas.pan, a.canvas.zoom, 100.0, 218.0);
        let to = point(rect, a.canvas.pan, a.canvas.zoom, 820.0, 218.0);
        frame(
            &ctx,
            &mut a,
            vec![egui::Event::PointerMoved(from), event(from, true)],
        );
        frame(&ctx, &mut a, vec![egui::Event::PointerMoved(to)]);
        frame(&ctx, &mut a, vec![event(to, false)]);
        match a.dialog.as_ref() {
            Some(Dialog::Connection(d)) => {
                assert!(d.selected.is_none());
                assert_eq!(d.from.as_ref().expect("from").port, "B.out");
            }
            _ => panic!("no reverse-drag dialog"),
        }
    }
    #[test]
    fn release_on_empty_canvas_cancels_without_mutation() {
        let ctx = egui::Context::default();
        let mut a = app();
        let before = a.store.project().clone();
        let rect = frame(&ctx, &mut a, vec![]);
        let from = point(rect, a.canvas.pan, a.canvas.zoom, 820.0, 218.0);
        let to = rect.right_bottom() - vec2(25.0, 25.0);
        frame(
            &ctx,
            &mut a,
            vec![egui::Event::PointerMoved(from), event(from, true)],
        );
        frame(&ctx, &mut a, vec![egui::Event::PointerMoved(to)]);
        frame(&ctx, &mut a, vec![event(to, false)]);
        assert!(a.dialog.is_none());
        assert_eq!(a.store.project(), &before);
    }
    #[test]
    fn editing_existing_wire_preserves_current_selection() {
        let a = app();
        let d = ConnectionDialog::new(a.store.project(), "root", None, Some("incoming"));
        assert_eq!(d.id.as_deref(), Some("incoming"));
        assert_eq!(d.selected.as_ref().expect("type").version, 1);
    }
}
