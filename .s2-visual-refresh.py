from pathlib import Path

view = Path("examples/s2-flow-candidate/view.rs")
s = view.read_text()

start = s.index("fn rect(s: &Step) -> Rect {")
end = s.index("fn anchor_key(e: &Endpoint) -> String {")
replacement = r'''#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        Kind::Entry | Kind::Outcome => vec2(210.0, 84.0),
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
        if d.x >= 0.0 { 1.0 / half.x } else { -1.0 / half.x },
        if d.y >= 0.0 { 1.0 / half.y } else { -1.0 / half.y },
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
'''
s = s[:start] + replacement + s[end:]

old = '''            let points = if transition == "S2.e10" {
                // Two distinct channels remain distinct around the return lane.
                let offset = if a.contract.starts_with("C04") {
                    12.0
                } else {
                    0.0
                };
                smooth(&[
                    a.point,
                    a.point + a.normal * 24.0,
                    pos2(16.0 + offset, a.point.y + 24.0),
                    pos2(16.0 + offset, b.point.y - 24.0),
                    b.point + b.normal * 24.0,
                    b.point,
                ])
            } else {
                direct(a.point, a.normal, b.point, b.normal)
            };'''
assert old in s
s = s.replace(old, '            let points = connector(a.point, a.normal, b.point, b.normal);')

old = '''        if diamond(step, app.at.tab) {
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
        } else {
            let round = if step.kind == Kind::Outcome || step.kind == Kind::Entry {
                28
            } else {
                10
            };
            painter.rect_filled(r, round, fill);
            painter.rect_stroke(r, round, stroke, StrokeKind::Inside);
            if step.kind == Kind::Call {
                painter.rect_stroke(
                    r.shrink(5.0 * z),
                    7,
                    Stroke::new(0.7, border),
                    StrokeKind::Inside,
                );
            }
        }
        let width = if diamond(step, app.at.tab) {
            r.width() * 0.63
        } else {
            r.width() - 24.0 * z
        };'''
assert old in s
replacement = '''        match flow_shape(step, app.at.tab) {
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
        };'''
s = s.replace(old, replacement)
s = s.replace(
    '            let inside = if diamond(step, app.at.tab) {',
    '            let inside = if flow_shape(step, app.at.tab) == FlowShape::Decision {',
)

marker = '#[cfg(test)]\nmod geometry_tests {'
idx = s.index(marker)
tests = r'''#[cfg(test)]
mod geometry_tests {
    use super::*;
    use crate::model::Document;

    #[test]
    fn standard_control_shapes_are_used_only_in_control_flow() {
        let doc = Document::load().unwrap();
        let parent = &doc.study.flows[0];
        assert_eq!(flow_shape(parent.step("step.S2.frame").unwrap(), Tab::Control), FlowShape::Process);
        assert_eq!(flow_shape(parent.step("step.S2.enough").unwrap(), Tab::Control), FlowShape::Decision);
        assert_eq!(flow_shape(parent.step("step.S2.experiment").unwrap(), Tab::Control), FlowShape::Subprocess);
        assert_eq!(flow_shape(parent.step("step.S2.exit").unwrap(), Tab::Control), FlowShape::Terminator);
        assert_eq!(flow_shape(parent.step("step.S2.enough").unwrap(), Tab::Interfaces), FlowShape::InterfaceCard);
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
'''
s = s[:idx] + tests
view.write_text(s, encoding="utf-8", newline="\n")

main = Path("examples/s2-flow-candidate/main.rs")
s = main.read_text()
old = 'ui.small("Rounded card · action\\nDiamond · recorded choice (Control Flow only)\\nDouble border · subprocess\\nPill · entry or outcome");'
new = 'ui.small("Control Flow uses standard flowchart notation:\\nRectangle · process/action\\nDiamond · decision/assessment\\nPredefined process · subprocess\\nTerminator · entry/outcome\\nInterfaces remain ordinary system cards");'
assert old in s
main.write_text(s.replace(old, new), encoding="utf-8", newline="\n")
