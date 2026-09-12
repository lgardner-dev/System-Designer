//! Authored fixture layout, not a new automatic layout engine.
use super::model::{Flow, Kind, Step};
use super::{App, Selection, Tab};
use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use std::collections::BTreeMap;
use system_designer::model::{Direction, Endpoint};
pub const ACCENT: Color32 = Color32::from_rgb(109, 203, 239);
pub const WARN: Color32 = Color32::from_rgb(225, 183, 110);
const MUTED: Color32 = Color32::from_rgb(157, 174, 193);
const BG: Color32 = Color32::from_rgb(13, 18, 26);
#[derive(Clone, Copy)]
pub struct Camera {
    pub pan: Vec2,
    pub zoom: f32,
}
#[derive(Clone)]
struct Route {
    id: String,
    transition: String,
    points: Vec<Pos2>,
    label: String,
}
#[derive(Clone)]
struct Anchor {
    point: Pos2,
    normal: Vec2,
    source: bool,
    name: String,
    contract: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FlowShape {
    Process,
    Decision,
    Subprocess,
    Terminator,
    InterfaceCard,
}
fn flow_shape(step: &Step, tab: Tab) -> FlowShape {
    if tab == Tab::Interfaces {
        return FlowShape::InterfaceCard;
    }
    match step.kind {
        Kind::Entry | Kind::Outcome => FlowShape::Terminator,
        Kind::Action => FlowShape::Process,
        Kind::Choice => FlowShape::Decision,
        Kind::Call => FlowShape::Subprocess,
    }
}
fn rect(s: &Step) -> Rect {
    let size = match s.kind {
        Kind::Choice => vec2(240.0, 140.0),
        // Boundary entry markers stay compact so authored process positions do not overlap.
        Kind::Entry => vec2(110.0, 80.0),
        Kind::Outcome => vec2(170.0, 80.0),
        _ => vec2(240.0, 108.0),
    };
    Rect::from_min_size(pos2(s.position[0], s.position[1]), size)
}
fn perimeter(r: Rect, toward: Pos2) -> (Pos2, Vec2) {
    let d = toward - r.center();
    if d.x.abs() > d.y.abs() {
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
fn diamond_anchor(r: Rect, toward: Pos2) -> (Pos2, Vec2) {
    let d = toward - r.center();
    if d.length_sq() < 0.001 {
        return (r.center_bottom(), vec2(0.0, 1.0));
    }
    let half = r.size() * 0.5;
    let scale = 1.0 / (d.x.abs() / half.x + d.y.abs() / half.y);
    let point = r.center() + d * scale;
    let normal = vec2(
        if d.x >= 0.0 {
            1.0 / half.x
        } else {
            -1.0 / half.x
        },
        if d.y >= 0.0 {
            1.0 / half.y
        } else {
            -1.0 / half.y
        },
    )
    .normalized();
    (point, normal)
}
fn control_anchor(step: &Step, toward: Pos2) -> (Pos2, Vec2) {
    let r = rect(step);
    if flow_shape(step, Tab::Control) == FlowShape::Decision {
        diamond_anchor(r, toward)
    } else {
        perimeter(r, toward)
    }
}
/// Same cubic Bezier connector used by the production System Design Canvas.
fn connector(a: Pos2, a_normal: Vec2, b: Pos2, b_normal: Vec2) -> Vec<Pos2> {
    let distance = (a.distance(b) * 0.4).clamp(20.0, 180.0);
    let controls = [a, a + a_normal * distance, b + b_normal * distance, b];
    let samples = (a.distance(b) / 12.0).ceil().clamp(24.0, 128.0) as usize;
    (0..=samples)
        .map(|i| {
            let t = i as f32 / samples as f32;
            let u = 1.0 - t;
            let v = controls[0].to_vec2() * (u * u * u)
                + controls[1].to_vec2() * (3.0 * u * u * t)
                + controls[2].to_vec2() * (3.0 * u * t * t)
                + controls[3].to_vec2() * (t * t * t);
            pos2(v.x, v.y)
        })
        .collect()
}
fn control_routes(flow: &Flow) -> Vec<Route> {
    flow.transitions
        .iter()
        .filter_map(|e| {
            let from = flow.step(&e.from)?;
            let to = flow.step(&e.to)?;
            let (a, an) = control_anchor(from, rect(to).center());
            let (b, bn) = control_anchor(to, rect(from).center());
            Some(Route {
                id: e.id.clone(),
                transition: e.id.clone(),
                points: connector(a, an, b, bn),
                label: e.condition.clone(),
            })
        })
        .collect()
}
fn anchor_key(e: &Endpoint) -> String {
    format!("{}::{}", e.node.as_deref().unwrap_or("boundary"), e.port)
}
fn interface_routes(app: &App, flow: &Flow) -> (Vec<Route>, Vec<Anchor>) {
    let Some(sys) = app.document.project.system(&flow.system) else {
        return (vec![], vec![]);
    };
    let resolve = |ep: &Endpoint, source: bool| -> Option<Rect> {
        let step = if let Some(id) = &ep.node {
            flow.steps.iter().find(|s| s.component.as_ref() == Some(id))
        } else {
            flow.steps.iter().find(|s| {
                s.component.is_none() && s.kind == if source { Kind::Entry } else { Kind::Outcome }
            })
        };
        step.map(rect)
    };
    let mut requests: BTreeMap<String, (Endpoint, bool, Rect, Vec<Pos2>)> = BTreeMap::new();
    for edge in &sys.edges {
        if let (Some(a), Some(b)) = (resolve(&edge.from, true), resolve(&edge.to, false)) {
            requests
                .entry(anchor_key(&edge.from))
                .or_insert_with(|| (edge.from.clone(), true, a, vec![]))
                .3
                .push(b.center());
            requests
                .entry(anchor_key(&edge.to))
                .or_insert_with(|| (edge.to.clone(), false, b, vec![]))
                .3
                .push(a.center());
        }
    }
    let mut grouped: BTreeMap<(String, u8), Vec<String>> = BTreeMap::new();
    let mut normals = BTreeMap::new();
    for (key, (ep, source, r, peers)) in &requests {
        let sum = peers.iter().fold(Vec2::ZERO, |s, p| s + p.to_vec2()) / peers.len() as f32;
        let (_, normal) = perimeter(*r, pos2(sum.x, sum.y));
        let side = if normal.x > 0.5 {
            1
        } else if normal.x < -0.5 {
            3
        } else if normal.y > 0.5 {
            2
        } else {
            0
        };
        let owner = ep.node.clone().unwrap_or_else(|| {
            if *source {
                "input.boundary".into()
            } else {
                "output.boundary".into()
            }
        });
        grouped.entry((owner, side)).or_default().push(key.clone());
        normals.insert(key.clone(), normal);
    }
    let mut anchors = BTreeMap::new();
    for ((_, side), keys) in grouped {
        for (i, key) in keys.iter().enumerate() {
            let (ep, source, r, _) = &requests[key];
            let fraction = (i + 1) as f32 / (keys.len() + 1) as f32;
            let point = match side {
                0 => pos2(r.left() + r.width() * fraction, r.top()),
                1 => pos2(r.right(), r.top() + r.height() * fraction),
                2 => pos2(r.left() + r.width() * fraction, r.bottom()),
                _ => pos2(r.left(), r.top() + r.height() * fraction),
            };
            let port = app.document.project.port(&flow.system, ep);
            anchors.insert(
                key.clone(),
                Anchor {
                    point,
                    normal: normals[key],
                    source: *source,
                    name: port.map(|p| p.name.clone()).unwrap_or(ep.port.clone()),
                    contract: port
                        .and_then(|p| p.contract.as_ref())
                        .map(ToString::to_string)
                        .unwrap_or("Unbound".into()),
                },
            );
        }
    }
    let mut routes = vec![];
    for e in &sys.edges {
        if let (Some(a), Some(b)) = (
            anchors.get(&anchor_key(&e.from)),
            anchors.get(&anchor_key(&e.to)),
        ) {
            let transition = flow
                .transitions
                .iter()
                .find(|t| t.exchanges.contains(&e.id))
                .map(|t| t.id.clone())
                .unwrap_or_default();
            let points = connector(a.point, a.normal, b.point, b.normal);
            routes.push(Route {
                id: e.id.clone(),
                transition,
                points,
                label: a.contract.clone(),
            });
        }
    }
    (routes, anchors.into_values().collect())
}
pub fn length(points: &[Pos2]) -> f32 {
    points.windows(2).map(|s| s[0].distance(s[1])).sum()
}
pub fn at(points: &[Pos2], distance: f32) -> (Pos2, Vec2) {
    let mut remaining = distance.max(0.0);
    for pair in points.windows(2) {
        let delta = pair[1] - pair[0];
        let len = delta.length();
        if len > 0.001 {
            if remaining <= len {
                return (pair[0] + delta * (remaining / len), delta / len);
            }
            remaining -= len;
        }
    }
    (points.last().copied().unwrap_or(Pos2::ZERO), Vec2::ZERO)
}
pub fn distance(points: &[Pos2], p: Pos2) -> f32 {
    points
        .windows(2)
        .map(|s| {
            let d = s[1] - s[0];
            let t = if d.length_sq() > 0.0 {
                ((p - s[0]).dot(d) / d.length_sq()).clamp(0.0, 1.0)
            } else {
                0.0
            };
            p.distance(s[0] + d * t)
        })
        .fold(f32::INFINITY, f32::min)
}
fn active(app: &App, flow: &Flow, r: &Route) -> bool {
    match &app.at.selection {
        Selection::None => false,
        Selection::Transition(id) => *id == r.transition,
        Selection::Exchange(id) => *id == r.id,
        Selection::Step(id) => flow
            .transition(&r.transition)
            .is_some_and(|t| t.from == *id || t.to == *id),
        Selection::Contract(id) => app
            .document
            .source_edge(
                flow.transition(&r.transition)
                    .map(|t| t.source_flow.as_str())
                    .unwrap_or(""),
            )
            .is_some_and(|e| e.contracts.contains(id)),
    }
}
pub fn canvas(app: &mut App, ui: &mut egui::Ui, flow: &Flow) {
    let (area, response) = ui.allocate_exact_size(
        ui.available_size().max(vec2(100.0, 100.0)),
        Sense::click_and_drag(),
    );
    let painter = ui.painter_at(area);
    painter.rect_filled(area, 8, BG);
    let (routes, anchors) = if app.at.tab == Tab::Control {
        (control_routes(flow), vec![])
    } else {
        interface_routes(app, flow)
    };
    let mut bounds = Rect::NOTHING;
    for s in &flow.steps {
        bounds = bounds.union(rect(s));
    }
    for route in &routes {
        for p in &route.points {
            bounds = bounds.union(Rect::from_min_max(*p, *p));
        }
    }
    bounds = bounds.expand(50.0);
    let key = (app.at.scope, app.at.original, app.at.tab);
    let cam = app.cameras.entry(key).or_insert_with(|| {
        let zoom = ((area.width() - 30.0) / bounds.width())
            .min((area.height() - 45.0) / bounds.height())
            .clamp(0.2, 1.2);
        Camera {
            pan: area.size() * 0.5 - bounds.center().to_vec2() * zoom,
            zoom,
        }
    });
    if response.dragged() {
        cam.pan += response.drag_delta();
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if let Some(p) = ui.input(|i| i.pointer.hover_pos()) {
            let world = (p - area.min - cam.pan) / cam.zoom;
            cam.zoom = (cam.zoom * (scroll * 0.002).exp()).clamp(0.2, 2.5);
            cam.pan = p - area.min - world * cam.zoom;
        }
    }
    let cam = *cam;
    let z = cam.zoom;
    let transform = |p: Pos2| area.min + cam.pan + p.to_vec2() * z;
    let rr = |r: Rect| Rect::from_min_max(transform(r.min), transform(r.max));
    if app.at.scope == 1 {
        let frame = rr(Rect::from_min_max(pos2(10.0, 135.0), pos2(1300.0, 420.0)));
        painter.rect_stroke(
            frame,
            12,
            Stroke::new(1.0, Color32::from_rgb(69, 92, 113)),
            StrokeKind::Inside,
        );
        painter.text(
            frame.min + vec2(20.0, 16.0),
            Align2::LEFT_TOP,
            "S2.experiment · public boundary is derived from the owner",
            FontId::proportional(13.0),
            MUTED,
        );
    }
    let pointer = ui.input(|i| i.pointer.hover_pos());
    let mut hovered_edge: Option<(String, f32)> = None;
    for route in &routes {
        let points: Vec<_> = route.points.iter().map(|p| transform(*p)).collect();
        let emphasized = active(app, flow, route);
        let quiet = app.at.selection != Selection::None && !emphasized;
        let color = if emphasized {
            ACCENT
        } else if quiet {
            Color32::from_rgb(53, 66, 83)
        } else {
            Color32::from_rgb(132, 156, 180)
        };
        painter.add(egui::Shape::line(
            points.clone(),
            Stroke::new(if emphasized { 2.8 } else { 1.7 }, color),
        ));
        let len = length(&points);
        let (tip, tangent) = at(&points, (len - 2.0).max(0.0));
        painter.arrow(
            tip - tangent * 13.0,
            tangent * 13.0,
            Stroke::new(1.8, color),
        );
        if app.lights && emphasized && ui.input(|i| i.focused) && len > 1.0 {
            let t = ui.input(|i| i.time) as f32;
            let d = (t * 105.0).rem_euclid(len + 90.0);
            if d < len {
                let p = at(&points, d).0;
                for (radius, alpha) in [(10.0, 12), (7.0, 23), (4.0, 65)] {
                    painter.circle_filled(
                        p,
                        radius,
                        Color32::from_rgba_unmultiplied(125, 213, 255, alpha),
                    );
                }
                for i in 1..=7 {
                    if d > i as f32 * 3.0 {
                        painter.circle_filled(
                            at(&points, d - i as f32 * 3.0).0,
                            1.4,
                            Color32::from_rgba_unmultiplied(160, 225, 255, 150 - i * 16),
                        );
                    }
                }
                painter.circle_filled(p, 2.0, Color32::WHITE);
            }
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
        if !route.label.is_empty() && (!quiet) && (app.at.tab == Tab::Control || emphasized) {
            let point = at(&points, len * 0.5).0 - vec2(0.0, 14.0);
            let galley = painter.layout(
                route.label.clone(),
                FontId::proportional((13.0 * z).max(10.0)),
                color,
                230.0 * z,
            );
            let label = Rect::from_center_size(point, galley.size() + vec2(12.0, 6.0));
            painter.rect_filled(label, 4, BG);
            painter.galley(label.min + vec2(6.0, 3.0), galley, color);
        }
        if let Some(pos) = pointer.filter(|p| area.contains(*p)) {
            let d = distance(&points, pos);
            if d < 8.0 && hovered_edge.as_ref().is_none_or(|(_, best)| d < *best) {
                hovered_edge = Some((route.id.clone(), d));
            }
        }
    }
    let mut hovered_node = None;
    for step in &flow.steps {
        let r = rr(rect(step));
        let selected = app.at.selection == Selection::Step(step.id.clone());
        let role = if step.id == flow.entry {
            "Entry - bounded work"
        } else if step.id == "step.S2.decide" {
            "Accountable authority"
        } else {
            match step.kind {
                Kind::Choice => "Semantic assessment",
                Kind::Call => "Proposed subprocess",
                Kind::Outcome => "Declared outcome",
                Kind::Entry => "Owner input",
                Kind::Action => "Bounded work",
            }
        };
        let border = if selected {
            ACCENT
        } else if step.kind == Kind::Choice {
            Color32::from_rgb(137, 126, 175)
        } else {
            Color32::from_rgb(77, 100, 121)
        };
        let fill = Color32::from_rgb(26, 35, 47);
        let stroke = Stroke::new(if selected { 2.4 } else { 1.3 }, border);
        match flow_shape(step, app.at.tab) {
            FlowShape::Decision => {
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
            FlowShape::Terminator => {
                let radius = r.height() * 0.5;
                painter.rect_filled(r, radius, fill);
                painter.rect_stroke(r, radius, stroke, StrokeKind::Inside);
            }
            FlowShape::Subprocess => {
                painter.rect_filled(r, 4, fill);
                painter.rect_stroke(r, 4, stroke, StrokeKind::Inside);
                let inset = 13.0 * z;
                for x in [r.left() + inset, r.right() - inset] {
                    painter.line_segment(
                        [pos2(x, r.top()), pos2(x, r.bottom())],
                        Stroke::new(if selected { 2.0 } else { 1.2 }, border),
                    );
                }
            }
            FlowShape::Process => {
                painter.rect_filled(r, 4, fill);
                painter.rect_stroke(r, 4, stroke, StrokeKind::Inside);
            }
            FlowShape::InterfaceCard => {
                painter.rect_filled(r, 10, fill);
                painter.rect_stroke(r, 10, stroke, StrokeKind::Inside);
            }
        }
        let width = if flow_shape(step, app.at.tab) == FlowShape::Decision {
            r.width() * 0.63
        } else if flow_shape(step, app.at.tab) == FlowShape::Subprocess {
            r.width() - 54.0 * z
        } else {
            r.width() - 24.0 * z
        };
        let title = painter.layout(
            step.name.clone(),
            FontId::proportional((17.0 * z).max(10.0)),
            Color32::from_rgb(231, 241, 250),
            width,
        );
        let title_at = pos2(
            r.center().x - title.size().x * 0.5,
            r.center().y - title.size().y * 0.5 - 8.0 * z,
        );
        painter.galley(title_at, title, Color32::WHITE);
        if step.component.is_some() {
            let small = if step.kind == Kind::Call {
                "Hypotheses · sealed plan · observations ›"
            } else {
                role
            };
            painter.text(
                pos2(r.center().x, r.center().y + 28.0 * z),
                Align2::CENTER_CENTER,
                small,
                FontId::proportional((10.5 * z).max(7.5)),
                if step.kind == Kind::Call {
                    ACCENT
                } else {
                    MUTED
                },
            );
        }
        if pointer.is_some_and(|p| {
            let inside = if flow_shape(step, app.at.tab) == FlowShape::Decision {
                ((p.x - r.center().x) / (r.width() * 0.5)).abs()
                    + ((p.y - r.center().y) / (r.height() * 0.5)).abs()
                    <= 1.0
            } else {
                r.contains(p)
            };
            inside && area.contains(p)
        }) {
            hovered_node = Some(step.id.clone());
        }
    }
    for anchor in anchors {
        let point = transform(anchor.point);
        let radius = 4.6;
        if anchor.source {
            painter.circle_filled(point, radius, ACCENT);
        } else {
            painter.circle_filled(point, radius, BG);
            painter.circle_stroke(point, radius, Stroke::new(1.4, ACCENT));
        }
        if pointer.is_some_and(|p| p.distance(point) < 10.0) {
            response.clone().on_hover_text(format!(
                "{} · {}\n{}",
                if anchor.source {
                    Direction::Out.label()
                } else {
                    Direction::In.label()
                },
                anchor.name,
                anchor.contract
            ));
        }
    }
    if response.clicked() {
        app.at.selection = if let Some(id) = hovered_node.clone() {
            Selection::Step(id)
        } else if let Some((id, _)) = hovered_edge {
            if app.at.tab == Tab::Control {
                Selection::Transition(id)
            } else {
                Selection::Exchange(id)
            }
        } else {
            Selection::None
        };
    }
    if response.double_clicked() {
        if let Some(id) = hovered_node {
            if flow.step(&id).is_some_and(|s| s.kind == Kind::Call) {
                app.enter(1);
            }
        }
    }
}

#[cfg(test)]
mod geometry_tests {
    use super::*;
    use crate::model::Document;

    #[test]
    fn standard_control_shapes_are_used_only_in_control_flow() {
        let doc = Document::load().unwrap();
        let parent = &doc.study.flows[0];
        assert_eq!(
            flow_shape(parent.step("step.S2.frame").unwrap(), Tab::Control),
            FlowShape::Process
        );
        assert_eq!(
            flow_shape(parent.step("step.S2.enough").unwrap(), Tab::Control),
            FlowShape::Decision
        );
        assert_eq!(
            flow_shape(parent.step("step.S2.experiment").unwrap(), Tab::Control),
            FlowShape::Subprocess
        );
        assert_eq!(
            flow_shape(parent.step("step.S2.exit").unwrap(), Tab::Control),
            FlowShape::Terminator
        );
        assert_eq!(
            flow_shape(parent.step("step.S2.enough").unwrap(), Tab::Interfaces),
            FlowShape::InterfaceCard
        );
    }

    #[test]
    fn decision_anchors_touch_the_drawn_diamond() {
        let r = Rect::from_min_size(pos2(10.0, 20.0), vec2(240.0, 140.0));
        for toward in [pos2(-50.0, 250.0), pos2(300.0, 300.0), pos2(130.0, -200.0)] {
            let (p, _) = diamond_anchor(r, toward);
            let normalized = ((p.x - r.center().x) / (r.width() * 0.5)).abs()
                + ((p.y - r.center().y) / (r.height() * 0.5)).abs();
            assert!((normalized - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn connector_matches_the_production_cubic_behavior() {
        let a = pos2(10.0, 20.0);
        let b = pos2(330.0, 250.0);
        let points = connector(a, vec2(1.0, 0.0), b, vec2(0.0, -1.0));
        assert_eq!(points.first().copied(), Some(a));
        assert_eq!(points.last().copied(), Some(b));
        assert!(points.len() >= 25);
        assert!(points.windows(2).any(|pair| {
            let d = pair[1] - pair[0];
            d.x.abs() > 0.01 && d.y.abs() > 0.01
        }));
    }

    #[test]
    fn every_choice_route_touches_its_choice_in_both_control_views() {
        let doc = Document::load().unwrap();
        for flow in [doc.original(), doc.study.flows[0].clone()] {
            for route in control_routes(&flow) {
                let transition = flow.transition(&route.transition).unwrap();
                for (id, point) in [
                    (&transition.from, route.points.first().unwrap()),
                    (&transition.to, route.points.last().unwrap()),
                ] {
                    let step = flow.step(id).unwrap();
                    if step.kind == Kind::Choice {
                        let r = rect(step);
                        let n = ((point.x - r.center().x) / (r.width() * 0.5)).abs()
                            + ((point.y - r.center().y) / (r.height() * 0.5)).abs();
                        assert!((n - 1.0).abs() < 0.001, "{}: {}", route.id, n);
                    }
                }
            }
        }
    }
}
