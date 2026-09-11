use super::*;
use crate::exchange::{self, Scope};
use geometry::Side;
use motion::Motion;

fn fixture() -> Project {
    parse(include_str!("../../../tests/fixtures/project.json")).expect("fixture")
}
fn endpoint(node: Option<&str>, port: &str) -> Endpoint {
    Endpoint {
        node: node.map(str::to_owned),
        port: port.into(),
    }
}
fn positions(b: Position) -> BTreeMap<String, Position> {
    BTreeMap::from([
        ("A".into(), Position { x: 500.0, y: 500.0 }),
        ("B".into(), b),
    ])
}
fn cases() -> [(Position, Side); 4] {
    [
        (Position { x: 500.0, y: 50.0 }, Side::Top),
        (
            Position {
                x: 1000.0,
                y: 500.0,
            },
            Side::Right,
        ),
        (
            Position {
                x: 500.0,
                y: 1000.0,
            },
            Side::Bottom,
        ),
        (Position { x: 20.0, y: 500.0 }, Side::Left),
    ]
}
#[test]
fn both_inputs_and_outputs_can_occupy_all_four_sides() {
    let p = fixture();
    for (b, side) in cases() {
        let scene = Scene::new(&p, "root", &positions(b));
        for (port, direction) in [("A.in", Direction::In), ("A.out", Direction::Out)] {
            let anchor = scene.port(&endpoint(Some("A"), port)).expect("anchor");
            assert_eq!(anchor.side, side);
            assert_eq!(anchor.direction, direction);
            assert_eq!(anchor.normal, side.normal());
        }
    }
}
#[test]
fn side_assignment_does_not_change_the_project_or_exchange_base() {
    let p = fixture();
    let before = serde_json::to_string(&p).expect("serialize");
    let packet = exchange::export(&p, "root", Scope::Level, None).expect("export");
    for (b, _) in cases() {
        let _scene = Scene::new(&p, "root", &positions(b));
    }
    assert_eq!(before, serde_json::to_string(&p).expect("serialize"));
    assert_eq!(
        serde_json::to_value(packet).expect("packet"),
        serde_json::to_value(exchange::export(&p, "root", Scope::Level, None).expect("export"))
            .expect("packet")
    );
}
#[test]
fn a_shared_fanout_port_keeps_one_anchor() {
    let mut p = fixture();
    let mut extra = p.system("root").expect("root").edges[1].clone();
    extra.id = "fanout.extra".into();
    p.system_mut("root").expect("root").edges.push(extra);
    validate(&p).expect("valid");
    let scene = Scene::new(&p, "root", &auto_layout(&p, "root"));
    assert_eq!(
        scene
            .ports
            .iter()
            .filter(|a| a.endpoint.port == "A.out")
            .count(),
        1
    );
}
#[test]
fn projected_child_ports_keep_owner_identity_and_effective_direction() {
    let p = fixture();
    let scene = Scene::new(&p, "a", &auto_layout(&p, "a"));
    for port in p.boundary("a") {
        let anchor = scene.port(&endpoint(None, &port.id)).expect("boundary");
        assert_eq!(anchor.direction, port.direction.opposite());
        assert_eq!(anchor.normal, -anchor.side.normal());
        assert_eq!(
            anchor.contract,
            port.contract.as_ref().expect("type").to_string()
        );
        assert!(anchor.external.contains('B'));
        assert!(
            scene
                .frame
                .expect("frame")
                .expand(0.01)
                .contains(anchor.point)
        );
    }
    assert_eq!(scene.ports.iter().filter(|a| a.boundary).count(), 2);
}
#[test]
fn root_does_not_gain_fake_external_ports() {
    let p = fixture();
    let scene = Scene::new(&p, "root", &auto_layout(&p, "root"));
    assert!(scene.frame.is_none());
    assert!(!scene.ports.iter().any(|a| a.boundary));
}
#[test]
fn unconnected_draft_ports_remain_visible_and_unassigned() {
    let mut p = fixture();
    p.node_mut("B").expect("B").1.ports.push(Port {
        id: "draft.port".into(),
        name: "Draft".into(),
        direction: Direction::In,
        contract: None,
    });
    let scene = Scene::new(&p, "root", &auto_layout(&p, "root"));
    let anchor = scene
        .port(&endpoint(Some("B"), "draft.port"))
        .expect("draft");
    assert_eq!(anchor.contract, "Unassigned");
    assert_eq!(anchor.direction, Direction::In);
}
#[test]
fn side_assignment_is_deterministic_and_not_an_edge_order_effect() {
    let mut p = fixture();
    let pos = positions(cases()[0].0);
    let before = Scene::new(&p, "root", &pos);
    p.system_mut("root").expect("root").edges.reverse();
    let after = Scene::new(&p, "root", &pos);
    for anchor in before.ports {
        let other = after.port(&anchor.endpoint).expect("other");
        assert_eq!(anchor.side, other.side);
        assert_eq!(anchor.point, other.point);
    }
}
#[test]
fn labels_have_distinct_nonoverlapping_slots_on_each_side() {
    let p = fixture();
    for (b, _) in cases() {
        let scene = Scene::new(&p, "root", &positions(b));
        let anchors: Vec<_> = scene
            .ports
            .iter()
            .filter(|a| a.endpoint.node.as_deref() == Some("A"))
            .collect();
        assert!(!anchors[0].label_rect.intersects(anchors[1].label_rect));
        for a in anchors {
            let card = scene.cards.iter().find(|c| c.id == "A").expect("card");
            assert!(card.rect.contains_rect(a.label_rect));
        }
    }
}
#[test]
fn paths_start_and_finish_at_real_ports_with_correct_tangents() {
    let p = fixture();
    for (b, _) in cases() {
        let scene = Scene::new(&p, "root", &positions(b));
        for edge in &p.system("root").expect("root").edges {
            let a = scene.port(&edge.from).expect("source");
            let b = scene.port(&edge.to).expect("target");
            let path = scene.route(&edge.from, &edge.to, 0).expect("path");
            assert!(path.at(0.0).0.distance(a.point) < 0.001);
            assert!(path.at(path.length).0.distance(b.point) < 0.001);
            assert!(path.at(0.0).1.dot(a.normal) > 0.8);
            assert!(path.at(path.length).1.dot(-b.normal) > 0.8);
            assert!(path.distance(path.at(path.length * 0.5).0) < 0.01);
        }
    }
}
#[test]
fn self_loop_routes_outside_its_own_card() {
    let mut p = fixture();
    p.system_mut("root").expect("root").edges.push(Edge {
        id: "loop".into(),
        from: endpoint(Some("A"), "A.out"),
        to: endpoint(Some("A"), "A.in"),
        label: None,
    });
    let scene = Scene::new(&p, "root", &auto_layout(&p, "root"));
    let rect = scene
        .cards
        .iter()
        .find(|c| c.id == "A")
        .expect("card")
        .rect
        .shrink(0.01);
    let path = scene
        .route(
            &endpoint(Some("A"), "A.out"),
            &endpoint(Some("A"), "A.in"),
            0,
        )
        .expect("loop");
    for i in 1..100 {
        assert!(!rect.contains(path.at(path.length * i as f32 / 100.0).0));
    }
}
#[test]
fn zoom_and_pan_preserve_path_endpoints_and_hit_testing() {
    let path = Path::between(
        pos2(20.0, 20.0),
        vec2(0.0, 1.0),
        pos2(150.0, 400.0),
        vec2(-1.0, 0.0),
    );
    let origin = pos2(11.0, 19.0);
    let pan = vec2(30.0, -10.0);
    for zoom in [0.2, 0.67, 1.0, 2.5] {
        let screen = path.screen(origin, pan, zoom);
        assert!((screen.length - path.length * zoom).abs() < 0.01);
        for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let point = origin + pan + path.at(path.length * fraction).0.to_vec2() * zoom;
            assert!(screen.distance(point) < 0.02);
        }
    }
}
#[test]
fn degenerate_paths_do_not_panic_or_animate() {
    for path in [
        Path::new(vec![]),
        Path::new(vec![Pos2::ZERO]),
        Path::new(vec![Pos2::ZERO, Pos2::ZERO]),
    ] {
        let _ = path.at(0.0);
        assert!(motion::position(1.0, path.length, "edge").is_none());
    }
    assert!(motion::position(f64::NAN, 10.0, "edge").is_none());
    assert!(motion::position(1.0, f32::INFINITY, "edge").is_none());
}
#[test]
fn lights_advance_source_to_destination_at_constant_screen_speed() {
    let mut checked = 0;
    for step in 0..1000 {
        let t = step as f64 * 0.01;
        if let (Some(a), Some(b)) = (
            motion::position(t, 300.0, "edge"),
            motion::position(t + 0.01, 300.0, "edge"),
        ) {
            assert!((b - a - 1.0).abs() < 0.001);
            checked += 1;
        }
    }
    assert!(checked > 500);
}
#[test]
fn light_has_arrival_gap_and_deterministic_per_edge_phase() {
    let samples: Vec<_> = (0..500)
        .map(|i| motion::position(i as f64 * 0.01, 300.0, "edge"))
        .collect();
    assert!(samples.iter().any(Option::is_none));
    assert!(samples.iter().any(Option::is_some));
    assert_eq!(
        motion::position(0.5, 300.0, "edge"),
        motion::position(0.5, 300.0, "edge")
    );
    assert!(
        (0..100).any(|i| motion::position(i as f64 * 0.02, 300.0, "edge.a")
            != motion::position(i as f64 * 0.02, 300.0, "edge.b"))
    );
}
#[test]
fn selected_motion_only_uses_actual_incident_edges() {
    let p = fixture();
    let edge = &p.system("root").expect("root").edges[0];
    assert!(!Motion::Off.includes(edge, &Selection::Edge(edge.id.clone())));
    assert!(Motion::All.includes(edge, &Selection::None));
    assert!(Motion::Selected.includes(edge, &Selection::Node("A".into())));
    assert!(!Motion::Selected.includes(edge, &Selection::Node("X".into())));
    assert!(!Motion::Selected.includes(edge, &Selection::Boundary("A.in".into())));
    let edge = &p.system("a").expect("a").edges[0];
    assert!(Motion::Selected.includes(edge, &Selection::Boundary("A.in".into())));
}
#[test]
fn text_truncation_is_unicode_safe() {
    assert_eq!(short("αβγδε", 3), "αβ…");
}
fn app(b: Position) -> Designer {
    let mut p = fixture();
    p.layout.insert("root".into(), positions(b));
    let mut a = Designer::blank();
    a.current = "root".into();
    a.store = crate::edit::Store::new(p).expect("store");
    a
}
fn frame(ctx: &egui::Context, a: &mut Designer, events: Vec<egui::Event>, time: f64) {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 1000.0))),
        events,
        time: Some(time),
        focused: true,
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| a.canvas_view(ui));
    });
}
fn event(pos: Pos2, pressed: bool) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    }
}
fn drag(ctx: &egui::Context, a: &mut Designer, from: Pos2, to: Pos2) {
    frame(
        ctx,
        a,
        vec![egui::Event::PointerMoved(from), event(from, true)],
        0.1,
    );
    frame(ctx, a, vec![egui::Event::PointerMoved(to)], 0.2);
    frame(ctx, a, vec![event(to, false)], 0.3);
}
#[test]
fn real_pointer_drag_from_every_side_opens_explicit_contract_dialog() {
    for (b, _) in cases() {
        let ctx = egui::Context::default();
        let mut a = app(b);
        frame(&ctx, &mut a, vec![], 0.0);
        let from = a.canvas.port_positions["B.out"];
        let to = a.canvas.port_positions["A.in"];
        drag(&ctx, &mut a, from, to);
        match a.dialog.as_ref() {
            Some(Dialog::Connection(d)) => {
                assert!(d.selected.is_none());
                assert_eq!(d.from.as_ref().expect("from").port, "B.out");
                assert_eq!(d.to.as_ref().expect("to").port, "A.in");
            }
            _ => panic!("missing contract dialog"),
        }
        assert_eq!(
            a.store.project().system("root").expect("root").edges.len(),
            2
        );
        assert_eq!(a.store.generation, 0);
    }
}
#[test]
fn reverse_pointer_drag_from_every_side_preserves_flow_direction() {
    for (b, _) in cases() {
        let ctx = egui::Context::default();
        let mut a = app(b);
        frame(&ctx, &mut a, vec![], 0.0);
        let from = a.canvas.port_positions["A.in"];
        let to = a.canvas.port_positions["B.out"];
        drag(&ctx, &mut a, from, to);
        match a.dialog.as_ref() {
            Some(Dialog::Connection(d)) => {
                assert!(d.selected.is_none());
                assert_eq!(d.from.as_ref().expect("from").port, "B.out");
            }
            _ => panic!("missing reverse-drag dialog"),
        }
    }
}
#[test]
fn click_connect_still_requires_contract_choice() {
    let ctx = egui::Context::default();
    let mut a = app(cases()[0].0);
    frame(&ctx, &mut a, vec![], 0.0);
    for (id, time) in [("B.out", 0.1), ("A.in", 0.5)] {
        let point = a.canvas.port_positions[id];
        frame(
            &ctx,
            &mut a,
            vec![egui::Event::PointerMoved(point), event(point, true)],
            time,
        );
        frame(&ctx, &mut a, vec![event(point, false)], time + 0.1);
    }
    assert!(matches!(a.dialog, Some(Dialog::Connection(_))));
    assert_eq!(a.store.generation, 0);
}
#[test]
fn child_boundary_drag_preserves_projected_source_identity() {
    let ctx = egui::Context::default();
    let mut a = app(cases()[0].0);
    a.navigate("a".into());
    frame(&ctx, &mut a, vec![], 0.0);
    let from = a.canvas.port_positions["A.in"];
    let to = a.canvas.port_positions["X.in"];
    drag(&ctx, &mut a, from, to);
    match a.dialog.as_ref() {
        Some(Dialog::Connection(d)) => {
            assert_eq!(d.from.as_ref().expect("source").node, None);
            assert_eq!(d.from.as_ref().expect("source").port, "A.in");
            assert!(d.selected.is_none());
        }
        _ => panic!("missing boundary dialog"),
    }
}
#[test]
fn invalid_drop_cancels_without_mutation() {
    let ctx = egui::Context::default();
    let mut a = app(cases()[0].0);
    let before = a.store.project().clone();
    frame(&ctx, &mut a, vec![], 0.0);
    let from = a.canvas.port_positions["B.out"];
    drag(&ctx, &mut a, from, pos2(1390.0, 990.0));
    assert!(a.dialog.is_none());
    assert!(a.canvas.pending.is_none());
    assert_eq!(&before, a.store.project());
}
#[test]
fn editing_existing_wire_keeps_its_current_contract() {
    let a = app(cases()[0].0);
    let d = ConnectionDialog::new(a.store.project(), "root", None, Some("incoming"));
    assert_eq!(d.id.as_deref(), Some("incoming"));
    assert_eq!(d.selected.as_ref().expect("type").version, 1);
}
#[test]
fn animation_frames_leave_publication_and_history_unchanged() {
    let ctx = egui::Context::default();
    let mut a = app(cases()[0].0);
    let before = a.store.project().clone();
    for i in 0..10 {
        frame(&ctx, &mut a, vec![], i as f64 * 0.1);
    }
    assert_eq!(a.store.project(), &before);
    assert_eq!(a.store.generation, 0);
    assert!(a.store.undo_label().is_none());
}
#[test]
fn motion_off_survives_navigation() {
    let ctx = egui::Context::default();
    let id = egui::Id::new("system-designer.direction-lights");
    ctx.data_mut(|d| d.insert_temp(id, Motion::Off));
    let mut a = app(cases()[0].0);
    frame(&ctx, &mut a, vec![], 0.0);
    a.navigate("a".into());
    frame(&ctx, &mut a, vec![], 1.0);
    assert_eq!(ctx.data(|d| d.get_temp::<Motion>(id)), Some(Motion::Off));
}
