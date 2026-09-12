//! Native views over the same component identities. Geometry never publishes edits.
use super::geometry::{Path, Scene, Side};
use super::model::{Atlas, Behavior, Kind};
use super::{App, Selection, Tab};
use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use std::collections::BTreeMap;
use system_designer::model::{Direction, Position};
pub const ACCENT: Color32 = Color32::from_rgb(109, 203, 239);
pub const MUTED: Color32 = Color32::from_rgb(154, 172, 193);
pub const WARN: Color32 = Color32::from_rgb(224, 182, 105);
const BG: Color32 = Color32::from_rgb(13, 18, 26);
#[derive(Clone, Copy)]
pub struct Camera {
    pub pan: Vec2,
    pub zoom: f32,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Process,
    Decision,
    Subprocess,
    Terminator,
    Io,
    Card,
}
#[derive(Clone)]
pub struct BoxView {
    pub id: String,
    pub name: String,
    pub subtitle: String,
    pub rect: Rect,
    pub shape: Shape,
    pub target: Option<String>,
}
#[derive(Clone)]
pub struct Wire {
    pub id: String,
    pub path: Path,
    pub label: String,
    pub from: String,
    pub to: String,
}
#[derive(Clone)]
struct PortView {
    id: String,
    point: Pos2,
    normal: Vec2,
    source: bool,
    name: String,
    contract: String,
    label: Rect,
    side: Side,
}
struct Drawing {
    boxes: Vec<BoxView>,
    wires: Vec<Wire>,
    ports: Vec<PortView>,
    bounds: Rect,
    frame: Option<Rect>,
    frame_title: String,
    leaf: bool,
}
fn shape(k: Kind) -> Shape {
    match k {
        Kind::Entry | Kind::Outcome => Shape::Terminator,
        Kind::Decision => Shape::Decision,
        Kind::Call => Shape::Subprocess,
        Kind::Io => Shape::Io,
        _ => Shape::Process,
    }
}
fn size(s: Shape) -> Vec2 {
    match s {
        Shape::Decision => vec2(250.0, 124.0),
        Shape::Terminator => vec2(155.0, 56.0),
        _ => vec2(250.0, 80.0),
    }
}
/// Kahn layering of this atlas's local procedures. A back-edge gets a bounded
/// fallback rank; no edge or its semantic direction is changed.
pub fn flow_boxes(flow: &Behavior) -> Vec<BoxView> {
    let ids: BTreeMap<_, _> = flow
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.id.as_str(), i))
        .collect();
    let mut incoming = vec![0usize; ids.len()];
    let mut next = vec![vec![]; ids.len()];
    for t in &flow.transitions {
        let (a, b) = (ids[t.from.as_str()], ids[t.to.as_str()]);
        next[a].push(b);
        incoming[b] += 1;
    }
    let mut rank = vec![0usize; ids.len()];
    let mut pending: std::collections::VecDeque<_> = incoming
        .iter()
        .enumerate()
        .filter(|(_, n)| **n == 0)
        .map(|(i, _)| i)
        .collect();
    let mut visited = vec![false; ids.len()];
    while let Some(i) = pending.pop_front() {
        visited[i] = true;
        for &j in &next[i] {
            rank[j] = rank[j].max(rank[i] + 1);
            incoming[j] -= 1;
            if incoming[j] == 0 {
                pending.push_back(j);
            }
        }
    }
    for (i, v) in visited.iter().enumerate() {
        if !v {
            rank[i] = i;
        }
    }
    let mut rows: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, &r) in rank.iter().enumerate() {
        rows.entry(r).or_default().push(i);
    }
    let max_width = rows.values().map(|row| row.len()).max().unwrap_or(1) as f32 * 310.0;
    let mut result = vec![];
    let mut y = 35.0;
    for row in rows.values() {
        let h = row
            .iter()
            .map(|&i| size(shape(flow.steps[i].kind)).y)
            .fold(0.0_f32, f32::max);
        for (j, &i) in row.iter().enumerate() {
            let s = &flow.steps[i];
            let sh = shape(s.kind);
            let sz = size(sh);
            let bypass_exit = row.len() == 1
                && s.kind == Kind::Outcome
                && flow
                    .transitions
                    .iter()
                    .any(|edge| edge.to == s.id && rank[ids[edge.from.as_str()]] + 1 < rank[i]);
            let x = if bypass_exit {
                // Keep a return with early exits out of the central action column.
                0.0
            } else {
                (max_width - row.len() as f32 * 310.0) * 0.5
                    + j as f32 * 310.0
                    + (310.0 - sz.x) * 0.5
            };
            result.push(BoxView {
                id: s.id.clone(),
                name: s.label.clone(),
                subtitle: if s.kind == Kind::Call {
                    "Enter component".into()
                } else {
                    String::new()
                },
                rect: Rect::from_min_size(pos2(x, y + (h - sz.y) * 0.5), sz),
                shape: sh,
                target: s.target.clone(),
            });
        }
        y += h + 52.0;
    }
    result
}
fn anchor(b: &BoxView, toward: Pos2) -> (Pos2, Vec2) {
    let r = b.rect;
    let d = toward - r.center();
    if b.shape == Shape::Decision {
        if d.length_sq() < 0.001 {
            return (r.center_bottom(), vec2(0.0, 1.0));
        }
        let half = r.size() * 0.5;
        let scale = 1.0 / (d.x.abs() / half.x + d.y.abs() / half.y);
        let normal = if d.x.abs() < 0.001 {
            vec2(0.0, d.y.signum())
        } else if d.y.abs() < 0.001 {
            vec2(d.x.signum(), 0.0)
        } else {
            vec2(d.x.signum() / half.x, d.y.signum() / half.y).normalized()
        };
        return (r.center() + d * scale, normal);
    }
    if d.x.abs() / r.width() > d.y.abs() / r.height() {
        if b.shape == Shape::Io {
            let sign = d.x.signum();
            let inset = r.width() * 14.0 / 250.0;
            return (
                r.center() + vec2(sign * (r.width() * 0.5 - inset * 0.5), 0.0),
                vec2(sign, sign * inset / r.height()).normalized(),
            );
        }
        if d.x > 0.0 {
            (r.right_center(), vec2(1.0, 0.0))
        } else {
            (r.left_center(), vec2(-1.0, 0.0))
        }
    } else if d.y > 0.0 {
        (r.center_bottom(), vec2(0.0, 1.0))
    } else {
        (r.center_top(), vec2(0.0, -1.0))
    }
}
pub fn contains(b: &BoxView, p: Pos2) -> bool {
    let r = b.rect;
    if b.shape == Shape::Decision {
        return ((p.x - r.center().x) / (r.width() * 0.5)).abs()
            + ((p.y - r.center().y) / (r.height() * 0.5)).abs()
            <= 1.0;
    }
    if b.shape == Shape::Terminator {
        let radius = r.height() * 0.5;
        let core = Rect::from_min_max(r.min + vec2(radius, 0.0), r.max - vec2(radius, 0.0));
        let x = p.x.clamp(core.left(), core.right());
        return p.distance(pos2(x, r.center().y)) <= radius;
    }
    if b.shape == Shape::Io {
        if !r.contains(p) {
            return false;
        }
        let t = (p.y - r.top()) / r.height();
        let inset = r.width() * 14.0 / 250.0;
        return p.x >= r.left() + inset * (1.0 - t) && p.x <= r.right() - inset * t;
    }
    r.contains(p)
}

fn route_cost(path: &Path, boxes: &[BoxView], from: &str, to: &str, frame: Option<Rect>) -> f32 {
    let hits: usize = boxes
        .iter()
        .filter(|node| node.id != from && node.id != to)
        .map(|node| {
            path.points
                .iter()
                .filter(|point| node.rect.expand(4.0).contains(**point))
                .count()
        })
        .sum();
    let outside = frame.map_or(0, |rect| {
        path.points
            .iter()
            .filter(|point| !rect.expand(1.0).contains(**point))
            .count()
    });
    hits as f32 * 1_000_000.0 + outside as f32 * 1_000_000_000.0 + path.length
}

/// Choose among the same cubic connectors, never orthogonal waypoint routes.
/// This bounded drawing heuristic does not alter flow order or interface identity.
fn control_path(from: &BoxView, to: &BoxView, boxes: &[BoxView]) -> Path {
    let (a, an) = anchor(from, to.rect.center());
    let (b, bn) = anchor(to, from.rect.center());
    let mut best = Path::between(a, an, b, bn);
    let mut cost = route_cost(&best, boxes, &from.id, &to.id, None);
    if cost < 1_000_000.0 {
        return best;
    }
    let options = |node: &BoxView, peer: Pos2| {
        let mut anchors = vec![anchor(node, peer)];
        for side in [
            vec2(0.0, -1.0),
            vec2(1.0, 0.0),
            vec2(0.0, 1.0),
            vec2(-1.0, 0.0),
        ] {
            anchors.push(anchor(node, node.rect.center() + side * 10_000.0));
        }
        anchors
    };
    for (a, an) in options(from, to.rect.center()) {
        for (b, bn) in options(to, from.rect.center()) {
            for tension in [1.0, 2.0, 4.0, 8.0] {
                let candidate = Path::between(a, an * tension, b, bn * tension);
                let score = route_cost(&candidate, boxes, &from.id, &to.id, None);
                if score < cost {
                    cost = score;
                    best = candidate;
                }
            }
        }
    }
    best
}

fn interface_path(
    scene: &Scene,
    edge: &system_designer::model::Edge,
    lane: usize,
    boxes: &[BoxView],
) -> Option<Path> {
    let mut best = scene.route(&edge.from, &edge.to, lane)?;
    let from = edge.from.node.as_deref().unwrap_or("@boundary");
    let to = edge.to.node.as_deref().unwrap_or("@boundary");
    let mut cost = route_cost(&best, boxes, from, to, scene.frame);
    if cost < 1_000_000.0 || from == to {
        return Some(best);
    }
    let a = scene.port(&edge.from)?;
    let b = scene.port(&edge.to)?;
    let turn = |n: Vec2| vec2(-n.y, n.x);
    for tension in [1.0, 2.0, 4.0] {
        for left in [0.0, 1.0, -1.0, 2.0, -2.0] {
            for right in [0.0, 1.0, -1.0, 2.0, -2.0] {
                // Lean outward handles while preserving their port positions
                // and a positive projection onto each outward normal.
                let an = (a.normal + turn(a.normal) * left) * tension;
                let bn = (b.normal + turn(b.normal) * right) * tension;
                let candidate = Path::between(a.point, an, b.point, bn);
                let score = route_cost(&candidate, boxes, from, to, scene.frame);
                if score < cost {
                    cost = score;
                    best = candidate;
                }
            }
        }
    }
    Some(best)
}

fn boundary_arrow(port: &PortView, zoom: f32, point: Pos2) -> (Pos2, Vec2) {
    let direction = if port.source {
        port.normal
    } else {
        -port.normal
    };
    let start = if port.source {
        point - direction * 24.0 * zoom
    } else {
        point + direction * 2.0 * zoom
    };
    (start, direction * 22.0 * zoom)
}

fn control(a: &Atlas, b: &Behavior) -> Drawing {
    let boxes = flow_boxes(b);
    let by: BTreeMap<_, _> = boxes.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut wires = vec![];
    for t in &b.transitions {
        let (from, to) = (by[t.from.as_str()], by[t.to.as_str()]);
        let path = control_path(from, to, &boxes);
        wires.push(Wire {
            id: t.id.clone(),
            path,
            label: t.label.clone(),
            from: t.from.clone(),
            to: t.to.clone(),
        });
    }
    let bounds = boxes.iter().fold(Rect::NOTHING, |r, b| r.union(b.rect));
    let bounds = wires
        .iter()
        .fold(bounds, |r, w| r.union(w.path.bounds))
        .expand(32.0);
    Drawing {
        boxes,
        wires,
        ports: vec![],
        bounds,
        frame: None,
        frame_title: a.name(&b.owner).into(),
        leaf: false,
    }
}
fn interface(a: &Atlas, b: &Behavior) -> Drawing {
    let Some(system) = a.system(&b.owner) else {
        return primitive_interface(a, b);
    };
    // Use the exact production card/port measurement. Local layout remains a view
    // proposal: no changes are made to the saved project or its scope hashes.
    let p = &a.project;
    let columns = if system.nodes.len() <= 3 {
        system.nodes.len().max(1)
    } else {
        3
    };
    let mut width = 300.0;
    let mut height = 180.0;
    let mut positions = BTreeMap::new();
    for _ in 0..4 {
        positions = system
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                (
                    n.id.clone(),
                    Position {
                        x: (280.0 + (i % columns) as f32 * (width + 170.0)) as f64,
                        y: (150.0 + (i / columns) as f32 * (height + 120.0)) as f64,
                    },
                )
            })
            .collect();
        let scene = Scene::new(p, &system.id, &positions);
        width = scene
            .cards
            .iter()
            .map(|c| c.rect.width())
            .fold(width, f32::max);
        height = scene
            .cards
            .iter()
            .map(|c| c.rect.height())
            .fold(height, f32::max);
    }
    let scene = Scene::new(p, &system.id, &positions);
    let boxes: Vec<BoxView> = scene
        .cards
        .iter()
        .filter_map(|c| {
            p.node(&c.id).map(|(_, n)| BoxView {
                id: c.id.clone(),
                name: n.name.clone(),
                subtitle: if n.child.is_some() {
                    "Decomposed · enter".into()
                } else {
                    "Leaf · inspect behavior".into()
                },
                rect: c.rect,
                shape: Shape::Card,
                target: Some(n.id.clone()),
            })
        })
        .collect();
    let ports = scene
        .ports
        .iter()
        .map(|x| PortView {
            id: x.endpoint.port.clone(),
            point: x.point,
            normal: x.normal,
            source: x.direction == Direction::Out,
            name: x.name.clone(),
            contract: x.contract.clone(),
            label: x.label_rect,
            side: x.side,
        })
        .collect();
    let wires: Vec<Wire> = system
        .edges
        .iter()
        .enumerate()
        .filter_map(|(i, e)| {
            interface_path(&scene, e, i, &boxes).map(|path| Wire {
                id: e.id.clone(),
                path,
                label: p
                    .port(&system.id, &e.from)
                    .and_then(|p| p.contract.as_ref())
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                from: e.from.node.clone().unwrap_or_else(|| "@boundary".into()),
                to: e.to.node.clone().unwrap_or_else(|| "@boundary".into()),
            })
        })
        .collect();
    let bounds = wires.iter().fold(scene.bounds, |r, wire| {
        r.union(wire.path.bounds.expand(20.0))
    });
    Drawing {
        boxes,
        wires,
        ports,
        bounds,
        frame: scene.frame,
        frame_title: format!("{} — exact public boundary", a.name(&b.owner)),
        leaf: false,
    }
}
fn primitive_interface(a: &Atlas, b: &Behavior) -> Drawing {
    let n = &a.project.node(&b.owner).expect("validated owner").1;
    let count = n
        .ports
        .iter()
        .filter(|p| p.direction == Direction::In)
        .count()
        .max(
            n.ports
                .iter()
                .filter(|p| p.direction == Direction::Out)
                .count(),
        )
        .max(1);
    let h = (count as f32 * 88.0 + 130.0).max(340.0);
    let frame = Rect::from_min_size(pos2(40.0, 50.0), vec2(880.0, h));
    let mut ports = vec![];
    let mut ix = 0;
    let mut ox = 0;
    for p in &n.ports {
        let is_in = p.direction == Direction::In;
        let i = if is_in {
            let v = ix;
            ix += 1;
            v
        } else {
            let v = ox;
            ox += 1;
            v
        };
        let pt = pos2(
            if is_in { frame.left() } else { frame.right() },
            frame.top() + 100.0 + i as f32 * 88.0,
        );
        let label = Rect::from_min_size(
            pt + vec2(if is_in { 14.0 } else { -300.0 }, -22.0),
            vec2(285.0, 62.0),
        );
        ports.push(PortView {
            id: p.id.clone(),
            point: pt,
            normal: vec2(if is_in { 1.0 } else { -1.0 }, 0.0),
            source: is_in,
            name: p.name.clone(),
            contract: p
                .contract
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "Unassigned".into()),
            label,
            side: if is_in { Side::Left } else { Side::Right },
        });
    }
    let body = BoxView {
        id: "@body".into(),
        name: n.name.clone(),
        subtitle: "One responsibility · no hidden children".into(),
        rect: Rect::from_center_size(frame.center(), vec2(260.0, 95.0)),
        shape: Shape::Card,
        target: None,
    };
    Drawing {
        boxes: vec![body],
        wires: vec![],
        ports,
        bounds: frame.expand(45.0),
        frame: Some(frame),
        frame_title: format!("{} — primitive boundary view", n.name),
        leaf: true,
    }
}
fn text(p: &egui::Painter, r: Rect, s: &str, font: f32, color: Color32) {
    let g = p.layout(s.to_owned(), FontId::proportional(font), color, r.width());
    let q = pos2(
        r.center().x - g.size().x * 0.5,
        r.center().y - g.size().y * 0.5,
    );
    p.with_clip_rect(r.intersect(p.clip_rect()))
        .galley(q, g, color);
}
fn paint_box(p: &egui::Painter, b: &BoxView, selected: bool, z: f32) {
    let r = b.rect;
    let fill = Color32::from_rgb(29, 38, 50);
    let color = if selected {
        ACCENT
    } else if b.shape == Shape::Decision {
        Color32::from_rgb(148, 139, 183)
    } else {
        Color32::from_rgb(82, 105, 127)
    };
    let stroke = Stroke::new(if selected { 2.4 } else { 1.3 }, color);
    match b.shape {
        Shape::Decision => {
            p.add(egui::Shape::convex_polygon(
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
        Shape::Io => {
            let inset = 14.0 * z;
            p.add(egui::Shape::convex_polygon(
                vec![
                    r.left_top() + vec2(inset, 0.0),
                    r.right_top(),
                    r.right_bottom() - vec2(inset, 0.0),
                    r.left_bottom(),
                ],
                fill,
                stroke,
            ));
        }
        _ => {
            let round = if b.shape == Shape::Terminator {
                r.height() * 0.5
            } else if b.shape == Shape::Card {
                7.0
            } else {
                2.0
            };
            p.rect_filled(r, round, fill);
            p.rect_stroke(r, round, stroke, StrokeKind::Inside);
            if b.shape == Shape::Subprocess {
                for x in [r.left() + 13.0 * z, r.right() - 13.0 * z] {
                    p.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], stroke);
                }
            }
        }
    }
    let w = if b.shape == Shape::Decision {
        r.width() * 0.66
    } else {
        r.width() - 32.0 * z
    };
    let tr = Rect::from_center_size(
        r.center() - vec2(0.0, if b.subtitle.is_empty() { 0.0 } else { 9.0 * z }),
        vec2(w, r.height() * 0.64),
    );
    text(
        p,
        tr,
        &b.name,
        (16.0 * z).max(9.5),
        Color32::from_rgb(230, 239, 249),
    );
    if !b.subtitle.is_empty() {
        text(
            p,
            Rect::from_center_size(
                r.center() + vec2(0.0, 25.0 * z),
                vec2(r.width() - 20.0 * z, 20.0 * z),
            ),
            &b.subtitle,
            (10.0 * z).max(7.0),
            MUTED,
        );
    }
}
fn emphasized(app: &App, b: &Behavior, w: &Wire) -> bool {
    match &app.selection {
        Selection::None => false,
        Selection::Step(id) => {
            if app.tab == Tab::Control {
                w.from == *id || w.to == *id
            } else {
                b.step(id)
                    .and_then(|s| s.target.as_ref())
                    .is_some_and(|x| w.from == *x || w.to == *x)
            }
        }
        Selection::Component(id) => {
            if app.tab == Tab::Interfaces {
                w.from == *id || w.to == *id
            } else {
                b.steps
                    .iter()
                    .filter(|s| s.target.as_ref() == Some(id))
                    .any(|s| w.from == s.id || w.to == s.id)
            }
        }
        Selection::Transition(id) => {
            if app.tab == Tab::Control {
                w.id == *id
            } else {
                b.transitions
                    .iter()
                    .find(|t| t.id == *id)
                    .is_some_and(|t| t.exchanges.contains(&w.id))
            }
        }
        Selection::Edge(id) => {
            if app.tab == Tab::Interfaces {
                w.id == *id
            } else {
                b.transitions
                    .iter()
                    .find(|t| t.id == w.id)
                    .is_some_and(|t| t.exchanges.contains(id))
            }
        }
        Selection::Port(_) => false,
    }
}
pub fn canvas(app: &mut App, ui: &mut egui::Ui, b: &Behavior) {
    let d = if app.tab == Tab::Control {
        control(&app.atlas, b)
    } else {
        interface(&app.atlas, b)
    };
    let (area, response) = ui.allocate_exact_size(
        ui.available_size().max(vec2(150.0, 100.0)),
        Sense::click_and_drag(),
    );
    let p = ui.painter_at(area);
    p.rect_filled(area, 8, BG);
    let key = (app.owner.clone(), app.tab);
    let camera = app.cameras.entry(key).or_insert_with(|| {
        let z = ((area.width() - 25.0) / d.bounds.width())
            .min((area.height() - 35.0) / d.bounds.height())
            .clamp(0.18, 1.35);
        Camera {
            pan: area.size() * 0.5 - d.bounds.center().to_vec2() * z,
            zoom: z,
        }
    });
    if response.dragged() {
        camera.pan += response.drag_delta();
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.01 {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                let world = (pos - area.min - camera.pan) / camera.zoom;
                camera.zoom = (camera.zoom * (scroll * 0.002).exp()).clamp(0.18, 2.8);
                camera.pan = pos - area.min - world * camera.zoom;
            }
        }
    }
    let cam = *camera;
    let z = cam.zoom;
    let screen = |q: Pos2| area.min + cam.pan + q.to_vec2() * z;
    let rr = |r: Rect| Rect::from_min_max(screen(r.min), screen(r.max));
    if let Some(f) = d.frame {
        let r = rr(f);
        p.rect_stroke(
            r,
            8,
            Stroke::new(1.3, Color32::from_rgb(70, 91, 111)),
            StrokeKind::Inside,
        );
        p.text(
            r.min + vec2(16.0, 18.0) * z,
            Align2::LEFT_TOP,
            &d.frame_title,
            FontId::proportional((15.0 * z).max(9.0)),
            MUTED,
        );
    }
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let mut hit_edge = None;
    let mut nearest = 8.0;
    for w in &d.wires {
        let path = w.path.screen(area.min, cam.pan, z);
        let hi = emphasized(app, b, w);
        let quiet = app.selection != Selection::None && !hi;
        let color = if hi {
            ACCENT
        } else if quiet {
            Color32::from_rgb(48, 65, 85)
        } else {
            Color32::from_rgb(117, 141, 163)
        };
        p.add(egui::Shape::line(
            path.points.clone(),
            Stroke::new(if hi { 2.6 } else { 1.6 }, color),
        ));
        let (tip, normal) = path.at((path.length - 5.0).max(0.0));
        p.arrow(tip - normal * 12.0, normal * 12.0, Stroke::new(1.6, color));
        if app.lights > 0
            && (hi || app.lights == 2 && app.selection == Selection::None)
            && ui.input(|i| i.focused)
            && path.length > 0.01
        {
            let time = ui.input(|i| i.time);
            let at = ((time * 100.0) % (path.length as f64 + 80.0)) as f32;
            if at < path.length {
                let q = path.at(at).0;
                for (radius, alpha) in [(10.0, 10), (7.0, 22), (4.0, 62)] {
                    p.circle_filled(
                        q,
                        radius,
                        Color32::from_rgba_unmultiplied(130, 220, 255, alpha),
                    );
                }
                p.circle_filled(q, 1.8, Color32::WHITE);
                for j in 1..7 {
                    if at > j as f32 * 3.0 {
                        p.circle_filled(
                            path.at(at - j as f32 * 3.0).0,
                            1.3,
                            Color32::from_rgba_unmultiplied(150, 220, 255, 150 - j * 20),
                        );
                    }
                }
            }
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
        if !w.label.is_empty() && !quiet {
            let middle = path.at(path.length * 0.5).0 - vec2(0.0, 12.0);
            let g = p.layout(
                w.label.clone(),
                FontId::proportional((11.5 * z).max(9.0)),
                color,
                185.0 * z,
            );
            let r = Rect::from_center_size(middle, g.size() + vec2(10.0, 5.0));
            p.rect_filled(r, 3, BG);
            p.galley(r.min + vec2(5.0, 2.5), g, color);
        }
        if let Some(pos) = pointer.filter(|v| area.contains(*v)) {
            let dist = path.distance(pos);
            if dist < nearest {
                nearest = dist;
                hit_edge = Some(w.id.clone());
            }
        }
    }
    let mut hit_node = None;
    let mut enter = None;
    for node in &d.boxes {
        let mut n = node.clone();
        n.rect = rr(n.rect);
        let selected = match &app.selection {
            Selection::Component(id) => *id == n.id,
            Selection::Step(id) => {
                if app.tab == Tab::Control {
                    *id == n.id
                } else {
                    b.step(id).and_then(|s| s.target.as_deref()) == Some(n.id.as_str())
                }
            }
            _ => false,
        };
        if n.shape == Shape::Card && !d.leaf {
            let r = n.rect;
            p.rect_filled(r, 7, Color32::from_rgb(29, 38, 50));
            p.rect_stroke(
                r,
                7,
                Stroke::new(
                    if selected { 2.4 } else { 1.2 },
                    if selected {
                        ACCENT
                    } else {
                        Color32::from_rgb(70, 91, 111)
                    },
                ),
                StrokeKind::Inside,
            );
            // Production side-specific labels may occupy the top band. Keep header at
            // the same measured position by looking it up in the rendered geometry.
            let top_label = d
                .ports
                .iter()
                .filter(|v| {
                    v.side == Side::Top
                        && v.point.x >= node.rect.left()
                        && v.point.x <= node.rect.right()
                        && (v.point.y - node.rect.top()).abs() < 0.1
                })
                .count()
                > 0;
            let y = r.top() + if top_label { 50.0 * z } else { 10.0 * z };
            text(
                &p,
                Rect::from_min_size(
                    pos2(r.left() + 12.0 * z, y),
                    vec2(r.width() - 24.0 * z, 32.0 * z),
                ),
                &n.name,
                (16.0 * z).max(9.5),
                Color32::WHITE,
            );
            text(
                &p,
                Rect::from_min_size(
                    pos2(r.left() + 12.0 * z, y + 32.0 * z),
                    vec2(r.width() - 24.0 * z, 18.0 * z),
                ),
                &n.subtitle,
                (10.0 * z).max(7.0),
                MUTED,
            );
        } else {
            paint_box(&p, &n, selected, z);
        }
        if pointer.is_some_and(|v| area.contains(v) && contains(&n, v)) {
            hit_node = Some(n.id.clone());
            enter = n.target.clone();
        }
    }
    let mut hit_port = None;
    for q in &d.ports {
        let pt = screen(q.point);
        let rad = 4.2;
        if q.source {
            p.circle_filled(pt, rad, ACCENT);
        } else {
            p.circle_filled(pt, rad, BG);
            p.circle_stroke(pt, rad, Stroke::new(1.3, ACCENT));
        }
        let r = rr(q.label);
        let pp = p.with_clip_rect(r.intersect(area));
        let name = short(
            &q.name,
            (r.width() / ((10.0 * z).max(8.0) * 0.52)).max(4.0) as usize,
        );
        let align = match q.side {
            Side::Left => Align2::LEFT_TOP,
            Side::Right => Align2::RIGHT_TOP,
            _ => Align2::CENTER_TOP,
        };
        let x = match q.side {
            Side::Left => r.left(),
            Side::Right => r.right(),
            _ => r.center().x,
        };
        pp.text(
            pos2(x, r.top()),
            align,
            name,
            FontId::proportional((10.5 * z).max(8.0)),
            Color32::from_rgb(214, 225, 238),
        );
        pp.text(
            pos2(x, r.top() + 14.0 * z),
            align,
            &q.contract,
            FontId::monospace((9.0 * z).max(7.0)),
            MUTED,
        );
        if d.leaf {
            let (start, direction) = boundary_arrow(q, z, pt);
            p.arrow(start, direction, Stroke::new(1.5_f32, ACCENT));
        }
        if pointer.is_some_and(|v| area.contains(v) && pt.distance(v) < 10.0) {
            hit_port = Some(q.id.clone());
            response
                .clone()
                .on_hover_text(format!("{}\n{}\n{}", q.name, q.contract, q.id));
        }
    }
    if d.leaf {
        p.text(
            pos2(area.center().x, area.bottom() - 16.0),
            Align2::CENTER_BOTTOM,
            "Exact public ports; no synthetic child nodes or internal interface edges.",
            FontId::proportional(12.0),
            MUTED,
        );
    }
    if response.clicked() {
        app.selection = if let Some(id) = hit_port {
            Selection::Port(id)
        } else if let Some(id) = hit_node {
            if app.tab == Tab::Control {
                Selection::Step(id)
            } else {
                Selection::Component(id)
            }
        } else if let Some(id) = hit_edge {
            if app.tab == Tab::Control {
                Selection::Transition(id)
            } else {
                Selection::Edge(id)
            }
        } else {
            Selection::None
        };
    }
    if response.double_clicked() {
        if let Some(owner) = enter {
            app.navigate(&owner);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_control_bypasses_do_not_run_through_other_steps() {
        let a = Atlas::load().unwrap();
        for b in &a.scopes {
            let drawing = control(&a, b);
            for wire in &drawing.wires {
                assert!(
                    route_cost(&wire.path, &drawing.boxes, &wire.from, &wire.to, None)
                        < 1_000_000.0,
                    "{}: {} crosses an unrelated step",
                    b.owner,
                    wire.id
                );
            }
        }
    }
    #[test]
    fn primitive_boundary_arrows_receive_inward_and_produce_outward() {
        let a = Atlas::load().unwrap();
        for b in &a.scopes {
            if a.system(&b.owner).is_some() {
                continue;
            }
            for port in primitive_interface(&a, b).ports {
                let (_, vector) = boundary_arrow(&port, 1.0, port.point);
                let dot = vector.dot(port.normal);
                assert!(if port.source { dot > 0.0 } else { dot < 0.0 });
            }
        }
    }
    #[test]
    fn input_output_shapes_attach_at_the_sloped_outline() {
        let node = BoxView {
            id: "io".into(),
            name: "Read".into(),
            subtitle: String::new(),
            rect: Rect::from_min_size(Pos2::ZERO, vec2(250.0, 80.0)),
            shape: Shape::Io,
            target: None,
        };
        for side in [-1.0, 1.0] {
            let (point, normal) = anchor(&node, node.rect.center() + vec2(side * 1000.0, 0.0));
            assert!(contains(&node, point - normal * 0.1));
            assert!(!contains(&node, point + normal * 0.1));
        }
    }
    #[test]
    fn interface_curve_clearance_keeps_every_exact_port_endpoint() {
        let a = Atlas::load().unwrap();
        for b in &a.scopes {
            let Some(system) = a.system(&b.owner) else {
                continue;
            };
            let drawing = interface(&a, b);
            for edge in &system.edges {
                let wire = drawing.wires.iter().find(|w| w.id == edge.id).unwrap();
                let from = drawing
                    .ports
                    .iter()
                    .find(|p| p.id == edge.from.port)
                    .unwrap();
                let to = drawing.ports.iter().find(|p| p.id == edge.to.port).unwrap();
                assert_eq!(wire.path.points.first().copied(), Some(from.point));
                assert_eq!(wire.path.points.last().copied(), Some(to.point));
            }
        }
    }
    #[test]
    fn all_control_endpoints_exist() {
        let a = Atlas::load().unwrap();
        for b in &a.scopes {
            let d = control(&a, b);
            assert_eq!(d.wires.len(), b.transitions.len());
            for w in &d.wires {
                assert!(w.path.length > 0.0);
                assert!(w.path.points.len() >= 25);
            }
        }
    }
    #[test]
    fn decisions_attach_to_visible_outline() {
        let a = Atlas::load().unwrap();
        for b in &a.scopes {
            let boxes = flow_boxes(b);
            for n in boxes.iter().filter(|n| n.shape == Shape::Decision) {
                for d in [vec2(400.0, 120.0), vec2(-400.0, -300.0), vec2(0.0, 300.0)] {
                    let (p, _) = anchor(n, n.rect.center() + d);
                    assert!(contains(n, p) || contains(n, p + (n.rect.center() - p) * 0.0001));
                }
            }
        }
    }
    #[test]
    fn terminator_corners_are_not_clickable() {
        let b = BoxView {
            id: "x".into(),
            name: "X".into(),
            subtitle: String::new(),
            rect: Rect::from_min_size(Pos2::ZERO, vec2(150.0, 60.0)),
            shape: Shape::Terminator,
            target: None,
        };
        assert!(!contains(&b, b.rect.min));
        assert!(contains(&b, b.rect.center()));
    }
    #[test]
    fn every_interface_view_has_exact_wire_count() {
        let a = Atlas::load().unwrap();
        for b in &a.scopes {
            let d = interface(&a, b);
            assert_eq!(
                d.wires.len(),
                a.system(&b.owner).map_or(0, |s| s.edges.len())
            );
        }
    }
    #[test]
    fn leaf_views_add_no_project_components() {
        let a = Atlas::load().unwrap();
        let before = serde_json::to_string(&a).unwrap();
        for b in &a.scopes {
            let _ = interface(&a, b);
            let _ = control(&a, b);
        }
        assert_eq!(before, serde_json::to_string(&a).unwrap());
    }
    #[test]
    fn primitive_ports_match_owner_exactly() {
        let a = Atlas::load().unwrap();
        for b in &a.scopes {
            if a.system(&b.owner).is_none() {
                let d = interface(&a, b);
                let n = a.project.node(&b.owner).unwrap().1;
                assert_eq!(d.ports.len(), n.ports.len());
                for p in &d.ports {
                    assert!(n.ports.iter().any(|v| v.id == p.id));
                }
            }
        }
    }
    #[test]
    fn every_interface_card_is_rectangular() {
        let a = Atlas::load().unwrap();
        for b in &a.scopes {
            assert!(
                interface(&a, b)
                    .boxes
                    .iter()
                    .all(|n| n.shape == Shape::Card)
            );
        }
    }
    #[test]
    fn curved_connector_keeps_endpoints() {
        let a = pos2(10.0, 20.0);
        let b = pos2(240.0, 200.0);
        let p = Path::between(a, vec2(1.0, 0.0), b, vec2(0.0, -1.0));
        assert_eq!(p.points.first().copied(), Some(a));
        assert_eq!(p.points.last().copied(), Some(b));
        assert!(
            p.points
                .windows(2)
                .any(|x| (x[1].x - x[0].x).abs() > 0.1 && (x[1].y - x[0].y).abs() > 0.1)
        );
    }
}
