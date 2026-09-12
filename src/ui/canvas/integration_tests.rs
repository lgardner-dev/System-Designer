use super::*;
use crate::exchange::{self, Scope};
fn fixture() -> Project {
    parse(include_str!("../../../tests/fixtures/project.json")).expect("fixture")
}
fn app() -> Designer {
    let p = fixture();
    let mut a = Designer::blank();
    a.current = p.root.clone();
    a.saved = Some(p.clone());
    a.store = edit::Store::new(p).expect("store");
    a
}
fn ep(node: Option<&str>, port: &str) -> Endpoint {
    Endpoint {
        node: node.map(str::to_owned),
        port: port.into(),
    }
}
fn multi() -> Project {
    let mut p = fixture();
    let e = p
        .system("root")
        .expect("root")
        .edges
        .iter()
        .find(|e| e.from.node.as_deref() == Some("A"))
        .expect("outgoing")
        .clone();
    let contract = p.port("root", &e.from).expect("port").contract.clone();
    for n in 1..4 {
        let from = format!("extra.out.{n}");
        let to = format!("extra.in.{n}");
        p.node_mut("A").expect("A").ports.push(Port {
            id: from.clone(),
            name: from.clone(),
            direction: Direction::Out,
            contract: contract.clone(),
        });
        p.node_mut("B").expect("B").ports.push(Port {
            id: to.clone(),
            name: to.clone(),
            direction: Direction::In,
            contract: contract.clone(),
        });
        p.system_mut("root").expect("root").edges.push(Edge {
            id: format!("extra.edge.{n}"),
            from: ep(Some("A"), &from),
            to: ep(Some("B"), &to),
            label: Some(format!("Distinct route {n}")),
        });
    }
    validate(&p).expect("multi fixture");
    p
}
fn packet(p: &Project, scope: Scope) -> serde_json::Value {
    serde_json::to_value(
        exchange::export(
            p,
            "root",
            scope,
            if scope == Scope::Component {
                Some("A")
            } else {
                None
            },
        )
        .expect("export"),
    )
    .expect("JSON")
}
fn render(ctx: &egui::Context, a: &mut Designer, events: Vec<egui::Event>) {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1500.0, 1000.0))),
        events,
        focused: true,
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| a.canvas_view(ui));
    });
}
#[test]
fn defaults_are_detail_focus_off_and_no_new_project_state() {
    let a = app();
    assert_eq!(a.canvas_session.view, View::Detail);
    assert_eq!(a.canvas_session.focus, Focus::Off);
    assert!(!a.dirty());
}
#[test]
fn grouping_preserves_every_edge_once_and_opposite_directions() {
    let p = multi();
    let s = p.system("root").expect("root");
    let groups = overview::groups(s);
    assert_eq!(groups.len(), 2);
    assert!(groups.iter().any(|g| g.len() == 4));
    let mut ids: Vec<_> = groups.iter().flatten().map(|e| e.id.clone()).collect();
    ids.sort();
    let mut original: Vec<_> = s.edges.iter().map(|e| e.id.clone()).collect();
    original.sort();
    assert_eq!(ids, original);
}
#[test]
fn partial_summary_reports_one_of_four_not_one_total() {
    let p = multi();
    let scene = overview::compact_scene(Scene::new(&p, "root", &auto_layout(&p, "root")));
    let focus = focus::Emphasis::new(
        &p,
        "root",
        &Selection::Port(ep(Some("A"), "A.out")),
        Focus::Selection,
    );
    let links = overview::links_for(&p, "root", &scene, true);
    let group = links.iter().find(|g| g.members.len() == 4).expect("group");
    assert_eq!(group.focused_count(&focus), 1);
    assert!(group.caption(&focus).contains("1 of 4 in focus"));
    assert_eq!(group.members.len(), 4);
}
#[test]
fn exact_edge_selection_survives_overview_without_promoting_members() {
    let p = multi();
    let id = p
        .system("root")
        .expect("root")
        .edges
        .iter()
        .find(|e| e.from.node.as_deref() == Some("A"))
        .expect("edge")
        .id
        .clone();
    let selection = Selection::Edge(id);
    let emphasis = focus::Emphasis::new(&p, "root", &selection, Focus::Selection);
    let links = overview::links_for(
        &p,
        "root",
        &overview::compact_scene(Scene::new(&p, "root", &auto_layout(&p, "root"))),
        true,
    );
    let group = links.iter().find(|g| g.members.len() == 4).expect("group");
    assert!(group.selected(&selection));
    assert_eq!(group.focused_count(&emphasis), 1);
}
#[test]
fn multi_edge_summary_cannot_be_deleted() {
    let mut a = app();
    let before = a.store.project().clone();
    a.selected = Selection::Summary("A".into(), "B".into());
    a.ask_delete();
    assert!(a.dialog.is_none());
    assert_eq!(&before, a.store.project());
    assert!(a.status.contains("exact member"));
}
#[test]
fn summary_edit_establishes_one_exact_edge_and_detail() {
    let mut a = app();
    a.canvas_session.view = View::Overview;
    a.selected = Selection::Summary("A".into(), "B".into());
    let id = a.store.project().system("root").expect("root").edges[0]
        .id
        .clone();
    a.show_exact(&id, true);
    assert_eq!(a.canvas_session.view, View::Detail);
    assert_eq!(a.selected, Selection::Edge(id.clone()));
    match a.dialog.as_ref() {
        Some(Dialog::Connection(d)) => assert_eq!(d.id.as_deref(), Some(id.as_str())),
        _ => panic!("exact dialog"),
    }
    assert_eq!(a.store.generation, 0);
}
#[test]
fn cancelling_connection_keeps_detail_and_preserves_project() {
    let mut a = app();
    let p = a.store.project().clone();
    let id = p.system("root").expect("root").edges[0].id.clone();
    a.canvas_session.view = View::Overview;
    a.show_exact(&id, true);
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 900.0))),
        events: vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    };
    let _ = ctx.run(input, |ctx| a.show_dialog(ctx));
    assert!(a.dialog.is_none());
    assert_eq!(a.canvas_session.view, View::Detail);
    assert_eq!(a.store.project(), &p);
}
#[test]
fn every_view_focus_light_combination_preserves_all_scope_packets() {
    for view in [View::Detail, View::Overview] {
        for focus in [
            Focus::Off,
            Focus::Selection,
            Focus::Incoming,
            Focus::Outgoing,
        ] {
            for light in [
                motion::Motion::Off,
                motion::Motion::Selected,
                motion::Motion::All,
            ] {
                let mut a = app();
                let before = a.store.project().clone();
                let ctx = egui::Context::default();
                ctx.data_mut(|d| {
                    d.insert_temp(egui::Id::new("system-designer.direction-lights"), light)
                });
                a.selected = Selection::Node("A".into());
                a.canvas_session.view = view;
                a.canvas_session.focus = focus;
                render(&ctx, &mut a, vec![]);
                for scope in [Scope::Component, Scope::Level, Scope::Subtree] {
                    assert_eq!(packet(&before, scope), packet(a.store.project(), scope));
                }
                assert_eq!(a.store.generation, 0);
                assert!(!a.dirty());
            }
        }
    }
}
#[test]
fn focus_is_one_hop_and_has_no_hidden_input_to_output_inference() {
    let p = fixture();
    let focus = focus::Emphasis::new(
        &p,
        "root",
        &Selection::Port(ep(Some("A"), "A.in")),
        Focus::Selection,
    );
    for e in &p.system("root").expect("root").edges {
        assert_eq!(
            focus.edges.contains(&e.id),
            e.to.port == "A.in" || e.from.port == "A.in"
        );
    }
}
#[test]
fn exact_selection_normalizes_directional_focus_visibly() {
    let mut a = app();
    a.selected = Selection::Edge("incoming".into());
    a.canvas_session.focus = Focus::Incoming;
    a.normalize_selection();
    assert_eq!(a.canvas_session.focus, Focus::Selection);
    a.selected = Selection::None;
    a.normalize_selection();
    assert_eq!(a.canvas_session.focus, Focus::Off);
}
#[test]
fn lights_respect_focus_before_summary_aggregation() {
    let p = multi();
    let selected = Selection::Node("A".into());
    let emphasis = focus::Emphasis::new(&p, "root", &selected, Focus::Incoming);
    let scene = overview::compact_scene(Scene::new(&p, "root", &auto_layout(&p, "root")));
    for link in overview::links_for(&p, "root", &scene, true) {
        assert_eq!(
            link.animate(motion::Motion::All, &selected, &emphasis, &p, "root"),
            link.edge.to.node.as_deref() == Some("A")
        );
        assert!(!link.animate(motion::Motion::Off, &selected, &emphasis, &p, "root"));
    }
}
#[test]
fn trace_crosses_identical_port_and_restores_all_navigation_state() {
    let mut a = app();
    a.selected = Selection::Port(ep(Some("A"), "A.in"));
    a.canvas_session.view = View::Overview;
    a.canvas_session.focus = Focus::Selection;
    a.canvas.pan = vec2(61.0, -20.0);
    a.canvas.zoom = 0.67;
    a.canvas.fit_requested = false;
    let old = a.canvas_location();
    let (sid, selection) =
        focus::through_boundary(a.store.project(), "root", &ep(Some("A"), "A.in")).expect("child");
    assert_eq!(selection, Selection::Boundary("A.in".into()));
    a.trace_visit(sid, selection);
    assert_eq!(a.current, "a");
    let back = focus::through_boundary(a.store.project(), "a", &ep(None, "A.in")).expect("parent");
    assert_eq!(back.0, "root");
    assert_eq!(back.1, old.selection);
    a.trace_back();
    assert_eq!(a.current, old.system);
    assert_eq!(a.selected, old.selection);
    assert_eq!(a.canvas_session.view, old.view);
    assert_eq!(a.canvas_session.focus, old.focus);
    assert_eq!(a.canvas.pan, old.viewport.pan);
    assert_eq!(a.canvas.zoom, old.viewport.zoom);
    assert!(!a.dirty());
}
#[test]
fn opaque_leaf_has_no_automatic_trace_continuation() {
    let p = fixture();
    assert!(focus::through_boundary(&p, "root", &ep(Some("B"), "B.in")).is_none());
}
#[test]
fn deleted_trace_target_is_reported_not_followed_by_name() {
    let mut a = app();
    a.selected = Selection::Node("A".into());
    a.trace_visit("root".into(), Selection::Node("B".into()));
    a.store
        .publish(
            "delete",
            edit::delete_node(a.store.project(), "A").expect("delete"),
        )
        .expect("publish");
    a.trace_back();
    assert_eq!(a.selected, Selection::None);
    assert_eq!(a.canvas_session.focus, Focus::Off);
    assert!(a.status.contains("removed"));
}
#[test]
fn view_switches_leave_origins_and_camera_fixed() {
    let p = fixture();
    let pos = auto_layout(&p, "root");
    let detail = Scene::new(&p, "root", &pos);
    let overview = overview::compact_scene(detail.clone());
    for (card, compact) in detail.cards.iter().zip(&overview.cards) {
        assert_eq!(card.rect.min, compact.rect.min);
        assert!(card.rect.width() >= compact.rect.width());
        assert!(card.rect.height() >= compact.rect.height());
    }
    let mut a = app();
    let ctx = egui::Context::default();
    render(&ctx, &mut a, vec![]);
    let before = (a.canvas.pan, a.canvas.zoom);
    a.canvas_session.view = View::Overview;
    render(&ctx, &mut a, vec![]);
    assert_eq!(before, (a.canvas.pan, a.canvas.zoom));
}
#[test]
fn arrangement_depends_on_full_graph_not_view_or_focus_and_preserves_scope() {
    let mut a = app();
    let p = a.store.project().clone();
    let expected =
        arrangement::arranged(&p, "root", arrangement::Axis::Horizontal).expect("arrange");
    a.canvas_session.view = View::Overview;
    a.selected = Selection::Port(ep(Some("A"), "A.in"));
    a.canvas_session.focus = Focus::Selection;
    arrangement::apply(&mut a, Some(arrangement::Axis::Horizontal));
    assert!(a.error.is_none());
    assert_eq!(&a.store.project().layout["root"], &expected);
    for scope in [Scope::Component, Scope::Level, Scope::Subtree] {
        assert_eq!(packet(&p, scope), packet(a.store.project(), scope));
    }
    let arranged = a.store.project().clone();
    a.store.undo();
    assert_eq!(a.store.project(), &p);
    a.store.redo();
    assert_eq!(a.store.project(), &arranged);
}
#[test]
fn frozen_side_order_does_not_jump_during_node_drag() {
    let p = fixture();
    let positions = BTreeMap::from([
        ("A".into(), Position { x: 500.0, y: 500.0 }),
        ("B".into(), Position { x: 500.0, y: 50.0 }),
    ]);
    let mut frozen = Scene::new(&p, "root", &positions);
    let original = frozen.port(&ep(Some("A"), "A.in")).expect("anchor").clone();
    frozen.move_card(
        "B",
        Position {
            x: 1100.0,
            y: 500.0,
        },
    );
    let still = frozen.port(&ep(Some("A"), "A.in")).expect("anchor");
    assert_eq!(still.point, original.point);
    assert_eq!(still.side, original.side);
}
#[test]
fn new_document_resets_view_focus_trace_but_not_light_choice() {
    let mut a = app();
    let ctx = egui::Context::default();
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new("system-designer.direction-lights"),
            motion::Motion::Off,
        )
    });
    a.canvas_session.view = View::Overview;
    a.canvas_session.focus = Focus::Incoming;
    a.selected = Selection::Node("A".into());
    a.trace_visit("root".into(), Selection::Node("B".into()));
    a.load(LoadAction::New, &ctx);
    assert_eq!(a.canvas_session.view, View::Detail);
    assert_eq!(a.canvas_session.focus, Focus::Off);
    assert!(a.canvas_session.history.is_empty());
    assert_eq!(
        ctx.data(
            |d| d.get_temp::<motion::Motion>(egui::Id::new("system-designer.direction-lights"))
        ),
        Some(motion::Motion::Off)
    );
}
#[test]
fn no_fake_boundary_or_cross_level_link_is_added_in_overview() {
    let p = fixture();
    for system in &p.systems {
        let scene =
            overview::compact_scene(Scene::new(&p, &system.id, &auto_layout(&p, &system.id)));
        assert_eq!(
            scene.ports.iter().filter(|a| a.boundary).count(),
            p.boundary(&system.id).len()
        );
        for link in overview::links_for(&p, &system.id, &scene, true) {
            for id in link.members {
                assert!(system.edges.iter().any(|e| e.id == id));
            }
        }
    }
}
#[test]
fn inspect_rows_render_for_ports_summaries_and_crossing_edges_without_hover() {
    let mut a = app();
    let ctx = egui::Context::default();
    let before = a.store.project().clone();
    for selection in [
        Selection::None,
        Selection::Node("A".into()),
        Selection::Port(ep(Some("A"), "A.in")),
        Selection::Summary("A".into(), "B".into()),
        Selection::Edge("incoming".into()),
    ] {
        a.selected = selection;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| a.inspector(ui));
        });
    }
    assert_eq!(a.store.project(), &before);
}
