use super::*;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, vec2};
fn app() -> Designer {
    let mut a = Designer::blank();
    let p = behavior::set(a.store.project(), ROOT, Flow::starter()).expect("flow");
    a.store = Store::new(p.clone()).expect("store");
    a.saved = Some(p);
    a
}
#[test]
fn f2_review_distinct_return_labels_have_separate_canvas_routes() {
    let mut a = app();
    a.store = Store::new(
        parse(include_str!(
            "../../../tests/fixtures/control-flow-review/coincident-returns.project.json"
        ))
        .expect("review fixture"),
    )
    .expect("store");
    let ctx = egui::Context::default();
    let output = frame(&mut a, &ctx, vec![], false);
    let accepted = text_position(&output, "Accepted").expect("accepted label");
    let rejected = text_position(&output, "Rejected").expect("rejected label");
    assert!(
        accepted.distance(rejected) > 24.0,
        "overlapping label positions: {accepted:?}, {rejected:?}"
    );
}
#[test]
fn f2_raw_canvas_selects_each_return_with_lights_off_and_static_arrows() {
    let mut a = app();
    a.store = Store::new(
        parse(include_str!(
            "../../../tests/fixtures/control-flow-review/coincident-returns.project.json"
        ))
        .expect("review fixture"),
    )
    .expect("store");
    let ctx = egui::Context::default();
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new("system-designer.direction-lights"),
            canvas::motion::Motion::Off,
        )
    });
    let output = frame(&mut a, &ctx, vec![], false);
    for id in ["accepted", "rejected"] {
        let path = a.flow.paths[id].clone();
        let tip = path.at(path.length).0;
        assert!(output.shapes.iter().any(|s| matches!(&s.shape, egui::Shape::Path(p) if p.closed && p.points.len() == 3 && p.points[0].distance(tip) < 0.1)), "missing static arrow for {id}");
        let at = path.at(path.length * 0.5).0;
        click(&mut a, &ctx, at, Modifiers::NONE, false);
        assert_eq!(a.flow.selection.transition.as_deref(), Some(id));
    }
    assert_eq!(a.store.generation, 0);
}
#[test]
fn f2_raw_canvas_selects_three_siblings_reciprocals_and_two_self_loops() {
    for case in ["three", "reciprocal", "loops"] {
        let mut a = app();
        let mut p = a.store.project().clone();
        let f = p.behavior.get_mut(ROOT).expect("flow");
        f.steps
            .push(Step::new("other", StepKind::Action, "Other work"));
        let count = if case == "three" { 3 } else { 2 };
        for i in 0..count {
            let (from, to) = if case == "loops" {
                ("action", "action")
            } else if case == "reciprocal" && i == 1 {
                ("other", "action")
            } else {
                ("action", "other")
            };
            f.transitions.push(Transition {
                id: format!("edge.{i}"),
                from: from.into(),
                to: to.into(),
                condition: format!("Alternative {i}"),
                outcome: None,
            });
        }
        p.flow_layout.insert(
            ROOT.into(),
            std::collections::BTreeMap::from([
                ("entry".into(), Position { x: 100.0, y: 30.0 }),
                ("action".into(), Position { x: 100.0, y: 260.0 }),
                ("other".into(), Position { x: 540.0, y: 260.0 }),
                ("done".into(), Position { x: 540.0, y: 490.0 }),
            ]),
        );
        a.store = Store::new(p).expect("valid draft");
        let ctx = egui::Context::default();
        frame(&mut a, &ctx, vec![], false);
        for i in 0..count {
            let id = format!("edge.{i}");
            let path = &a.flow.paths[&id];
            let at = path.at(path.length * 0.5).0;
            click(&mut a, &ctx, at, Modifiers::NONE, false);
            assert_eq!(
                a.flow.selection.transition.as_deref(),
                Some(id.as_str()),
                "case {case}"
            );
        }
        assert_eq!(a.store.generation, 0);
    }
}

#[test]
fn f2_geometry_parallel_reciprocal_and_loop_routes_are_stable_and_exact() {
    use egui::pos2;
    for (count, reverse, loops) in [
        (2, false, false),
        (3, false, false),
        (2, true, false),
        (2, false, true),
    ] {
        for kind in [
            StepKind::Action,
            StepKind::Decision,
            StepKind::Entry,
            StepKind::Outcome,
            StepKind::Call,
            StepKind::Merge,
        ] {
            let mut f = Flow {
                steps: vec![
                    Step::new("a", kind, "Same label"),
                    Step::new("b", kind, "Same label"),
                ],
                ..Default::default()
            };
            for i in 0..count {
                let (from, to) = if loops {
                    ("a", "a")
                } else if reverse && i == 1 {
                    ("b", "a")
                } else {
                    ("a", "b")
                };
                f.transitions.push(Transition {
                    id: format!("edge.{i}"),
                    from: from.into(),
                    to: to.into(),
                    condition: "Same guard".into(),
                    outcome: None,
                });
            }
            let (_, height) = behavior::footprint(kind);
            let rects = std::collections::BTreeMap::from([
                (
                    "a".into(),
                    Rect::from_min_size(pos2(100.0, 300.0), vec2(260.0, height as f32)),
                ),
                (
                    "b".into(),
                    Rect::from_min_size(pos2(600.0, 340.0), vec2(260.0, height as f32)),
                ),
            ]);
            let routes = drawing::routes(&f, &rects);
            for (i, (t, path)) in routes.iter().enumerate() {
                assert!(path.bounds.is_finite() && path.length.is_finite() && path.length > 1.0);
                assert!(path.points.iter().all(|p| p.is_finite()));
                let (from, direction) = path.at(0.0);
                let (to, arriving) = path.at(path.length);
                let (expected_from, outward) = drawing::perimeter(rects[&t.from], kind, from);
                let (expected_to, target_normal) = drawing::perimeter(rects[&t.to], kind, to);
                assert!(expected_from.distance(from) < 0.01 && expected_to.distance(to) < 0.01);
                assert!(direction.dot(outward) > 0.85 && arriving.dot(-target_normal) > 0.85);
                for (_, other) in &routes[i + 1..] {
                    let middle = path.at(path.length * 0.5).0;
                    assert!(
                        other.distance(middle) > 15.0,
                        "indistinguishable lane for {:?}, reverse={reverse}, loops={loops}",
                        kind
                    );
                }
            }
            let original: std::collections::BTreeMap<_, _> = routes
                .iter()
                .map(|(t, p)| (t.id.clone(), p.points.clone()))
                .collect();
            f.transitions.reverse();
            f.steps.reverse();
            for t in &mut f.transitions {
                t.condition = "Renamed guard".into();
            }
            for s in &mut f.steps {
                s.name = "Renamed step".into();
            }
            let reordered: std::collections::BTreeMap<_, _> = drawing::routes(&f, &rects)
                .into_iter()
                .map(|(t, p)| (t.id.clone(), p.points))
                .collect();
            assert_eq!(original, reordered);
        }
    }
}
#[test]
fn f1_extracted_parent_and_child_scenes_keep_cards_separate() {
    let p = parse(include_str!(
        "../../../tests/fixtures/control-flow-review/before-extraction.project.json"
    ))
    .expect("review fixture");
    let plan = behavior::preview(
        &p,
        ROOT,
        &BTreeSet::from(["a".into(), "b".into(), "c".into()]),
        "Import record",
        "Prepare one record",
    )
    .expect("preview");
    let mut a = app();
    a.store = Store::new(behavior::apply(&p, &plan).expect("apply")).expect("store");
    let ctx = egui::Context::default();
    for owner in [ROOT, plan.component.as_str()] {
        a.go_scope(owner.into(), Layer::Flow);
        frame(&mut a, &ctx, vec![], false);
        let rects: Vec<_> = a.flow.rects.iter().collect();
        for (i, (id, rect)) in rects.iter().enumerate() {
            for (other, bounds) in &rects[i + 1..] {
                assert!(!rect.intersects(**bounds), "{owner}: {id} overlaps {other}");
            }
        }
    }
}
fn frame(
    a: &mut Designer,
    ctx: &egui::Context,
    events: Vec<Event>,
    modal: bool,
) -> egui::FullOutput {
    let modifiers = events
        .iter()
        .find_map(|e| {
            if let Event::PointerButton { modifiers, .. } = e {
                Some(*modifiers)
            } else {
                None
            }
        })
        .unwrap_or_default();
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1500.0, 1000.0))),
            focused: true,
            events,
            modifiers,
            ..Default::default()
        },
        |ctx| {
            a.normalize_selection();
            a.shortcuts(ctx);
            if modal {
                a.show_dialog(ctx);
            } else {
                egui::CentralPanel::default().show(ctx, |ui| a.flow_view(ui));
            }
        },
    )
}
fn pointer(pos: Pos2, pressed: bool, modifiers: Modifiers) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers,
    }
}
fn click(a: &mut Designer, ctx: &egui::Context, pos: Pos2, modifiers: Modifiers, modal: bool) {
    frame(
        a,
        ctx,
        vec![Event::PointerMoved(pos), pointer(pos, true, modifiers)],
        modal,
    );
    frame(a, ctx, vec![pointer(pos, false, modifiers)], modal);
}
fn text_position(output: &egui::FullOutput, label: &str) -> Option<Pos2> {
    fn find(shape: &egui::Shape, label: &str) -> Option<Pos2> {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + t.galley.size() * 0.5),
            egui::Shape::Vec(v) => v.iter().find_map(|s| find(s, label)),
            _ => None,
        }
    }
    output.shapes.iter().find_map(|s| find(&s.shape, label))
}
fn button(a: &mut Designer, ctx: &egui::Context, label: &str) {
    frame(a, ctx, vec![], true);
    for _ in 0..6 {
        let output = frame(a, ctx, vec![], true);
        if let Some(pos) = text_position(&output, label) {
            click(a, ctx, pos, Modifiers::NONE, true);
            return;
        }
        frame(
            a,
            ctx,
            vec![
                Event::PointerMoved(egui::pos2(750.0, 500.0)),
                Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: vec2(0.0, -350.0),
                    modifiers: Modifiers::NONE,
                },
            ],
            true,
        );
    }
    panic!("No visible text {label}");
}
#[test]
fn root_leaf_and_layer_navigation_never_invent_structural_children() {
    let mut a = app();
    let (p, id) = edit::add_node(a.store.project(), &a.current).expect("node");
    a.publish("Add", Ok(p));
    let before = a.store.project().clone();
    let generation = a.store.generation;
    a.canvas.pan = vec2(111.0, 222.0);
    a.canvas.zoom = 0.6;
    a.canvas.fit_requested = false;
    a.flow.selection.step("action".into(), false);
    a.go_scope(id.clone(), Layer::Flow);
    assert!(a.interface_system().is_none());
    assert!(a.canvas.fit_requested);
    assert_eq!(a.owner, id);
    a.go_scope(id.clone(), Layer::Interfaces);
    assert!(a.interface_system().is_none());
    a.back_scope();
    assert_eq!(a.layer, Layer::Flow);
    a.back_scope();
    assert_eq!(a.owner, ROOT);
    assert_eq!(a.canvas.pan, vec2(111.0, 222.0));
    assert_eq!(a.canvas.zoom, 0.6);
    assert!(a.flow.selection.steps.contains("action"));
    assert_eq!(a.store.project(), &before);
    assert_eq!(a.store.generation, generation);
    a.go_scope(id.clone(), Layer::Flow);
    a.publish(
        "Leaf flow",
        behavior::set(a.store.project(), &id, Flow::starter()),
    );
    assert!(a.store.project().node(&id).expect("leaf").1.child.is_none());
    assert!(a.store.project().behavior.contains_key(&id));
}
#[test]
fn raw_connection_drags_in_both_directions_open_unpublished_drafts() {
    for reverse in [false, true] {
        let ctx = egui::Context::default();
        let mut a = app();
        frame(&mut a, &ctx, vec![], false);
        let from = a.flow.rects["action"].right_center();
        let to = a.flow.rects["done"].left_center();
        let (start, end) = if reverse { (to, from) } else { (from, to) };
        let p = a.store.project().clone();
        frame(
            &mut a,
            &ctx,
            vec![
                Event::PointerMoved(start),
                pointer(start, true, Modifiers::NONE),
            ],
            false,
        );
        frame(&mut a, &ctx, vec![Event::PointerMoved(end)], false);
        frame(
            &mut a,
            &ctx,
            vec![pointer(end, false, Modifiers::NONE)],
            false,
        );
        assert!(
            matches!(&a.dialog,Some(Dialog::Flow(FlowDialog::Transition(t))) if t.from == "action" && t.to == "done")
        );
        assert_eq!(a.store.project(), &p);
        assert_eq!(a.store.generation, 0);
        button(&mut a, &ctx, "Cancel — discard draft");
        assert!(a.dialog.is_none());
        assert_eq!(a.store.project(), &p);
    }
}
#[test]
fn raw_selection_preview_cancel_apply_cross_view_and_undo() {
    let ctx = egui::Context::default();
    let mut a = app();
    frame(&mut a, &ctx, vec![], false);
    let pos = a.flow.rects["action"].center();
    click(&mut a, &ctx, pos, Modifiers::NONE, false);
    assert!(a.flow.selection.steps.contains("action"));
    let original = a.store.project().clone();
    a.extraction_dialog(false);
    if let Some(Dialog::Flow(FlowDialog::Extract(d))) = &mut a.dialog {
        d.name = "Validate import".into();
        d.purpose = "Validate input records".into();
    }
    button(&mut a, &ctx, "Preview boundary");
    assert!(matches!(&a.dialog,Some(Dialog::Flow(FlowDialog::Extract(d))) if d.plan.is_some()));
    button(&mut a, &ctx, "Cancel — discard draft");
    assert_eq!(a.store.project(), &original);
    assert_eq!(a.store.generation, 0);
    a.extraction_dialog(false);
    if let Some(Dialog::Flow(FlowDialog::Extract(d))) = &mut a.dialog {
        d.name = "Validate import".into();
        d.purpose = "Validate input records".into();
    }
    button(&mut a, &ctx, "Preview boundary");
    button(&mut a, &ctx, "Create component");
    assert!(a.dialog.is_none());
    assert_eq!(a.store.generation, 1);
    let component = a.flow.success.clone().expect("success").1;
    let call = a.flow.selection.steps.iter().next().expect("call").clone();
    a.show_component_interfaces(&component);
    assert_eq!(a.selected, Selection::Node(component.clone()));
    a.go_scope(component.clone(), Layer::Flow);
    assert!(a.interface_system().is_none());
    a.back_scope();
    assert_eq!(a.layer, Layer::Interfaces);
    a.back_scope();
    assert!(a.flow.selection.steps.contains(&call));
    a.store.undo();
    a.normalize_selection();
    assert_eq!(a.store.project(), &original);
    assert!(a.flow.selection.steps.is_empty());
    a.store.redo();
    a.normalize_selection();
    assert!(a.store.project().node(&component).is_some());
}
#[test]
fn raw_move_and_escape_do_not_publish_cancelled_gestures() {
    let ctx = egui::Context::default();
    let mut a = app();
    frame(&mut a, &ctx, vec![], false);
    let original = a.store.project().clone();
    let start = a.flow.rects["action"].center();
    let end = start + vec2(60.0, 50.0);
    frame(
        &mut a,
        &ctx,
        vec![
            Event::PointerMoved(start),
            pointer(start, true, Modifiers::NONE),
        ],
        false,
    );
    frame(&mut a, &ctx, vec![Event::PointerMoved(end)], false);
    frame(
        &mut a,
        &ctx,
        vec![Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        false,
    );
    frame(
        &mut a,
        &ctx,
        vec![pointer(end, false, Modifiers::NONE)],
        false,
    );
    assert_eq!(a.store.project(), &original);
    assert_eq!(a.store.generation, 0);
}
#[test]
fn changing_meaning_in_form_and_deleting_is_undoable() {
    let ctx = egui::Context::default();
    let mut a = app();
    let mut step = a.store.project().behavior[ROOT].steps[1].clone();
    step.name = "Validate".into();
    a.dialog = Some(Dialog::Flow(FlowDialog::Step(step)));
    button(&mut a, &ctx, "Save step");
    assert!(a.dialog.is_none());
    assert_eq!(a.store.generation, 1);
    a.ask_delete_flow();
    button(&mut a, &ctx, "Delete listed items");
    assert!(a.store.project().behavior[ROOT].step("action").is_none());
    a.store.undo();
    assert_eq!(
        a.store.project().behavior[ROOT]
            .step("action")
            .expect("restored")
            .name,
        "Validate"
    );
}
#[test]
fn diamond_and_terminator_intersections_are_on_visible_perimeters() {
    let r = Rect::from_min_size(Pos2::ZERO, vec2(200.0, 100.0));
    for direction in [vec2(1.0, 1.0), vec2(-2.0, 1.0), vec2(1.0, -3.0)] {
        let (p, _) = drawing::perimeter(r, StepKind::Decision, r.center() + direction);
        let d = p - r.center();
        assert!((d.x.abs() / 100.0 + d.y.abs() / 50.0 - 1.0).abs() < 0.0001);
        let (p, _) = drawing::perimeter(r, StepKind::Outcome, r.center() + direction);
        let d = p - r.center();
        assert!(((d.x / 100.0).powi(2) + (d.y / 50.0).powi(2) - 1.0).abs() < 0.0001);
    }
}
#[test]
fn stale_plan_and_visual_camera_motion_preserve_store_history() {
    let mut a = app();
    let original = a.store.project().clone();
    let plan = behavior::preview(
        &original,
        ROOT,
        &BTreeSet::from(["action".into()]),
        "Worker",
        "One duty",
    )
    .expect("plan");
    a.canvas.pan = vec2(33.0, 44.0);
    a.flow.selection.step("action".into(), false);
    assert!(behavior::apply(a.store.project(), &plan).is_ok());
    a.publish(
        "Move",
        edit::candidate(&original, |q| {
            q.flow_layout
                .entry(ROOT.into())
                .or_default()
                .insert("action".into(), Position { x: 40.0, y: 40.0 });
            Ok(())
        }),
    );
    let before = a.store.project().clone();
    let generation = a.store.generation;
    a.accept_extraction(&plan);
    assert!(a.error.is_some());
    assert_eq!(a.store.project(), &before);
    assert_eq!(a.store.generation, generation);
}

#[test]
fn raw_information_review_cancel_and_apply_reconcile_both_layers() {
    let ctx = egui::Context::default();
    let mut a = app();
    let link = DataLink {
        id: "input".into(),
        name: "Input record".into(),
        from: DataEnd {
            step: None,
            port: None,
        },
        to: DataEnd {
            step: Some("action".into()),
            port: None,
        },
        contract: None,
        exchange: None,
    };
    let (p, _) =
        behavior::information_candidate(a.store.project(), ROOT, link, None).expect("information");
    let plan = behavior::preview(
        &p,
        ROOT,
        &BTreeSet::from(["action".into()]),
        "Worker",
        "One duty",
    )
    .expect("plan");
    a.store = Store::new(behavior::apply(&p, &plan).expect("extraction")).expect("store");
    let original = a.store.project().clone();
    a.open_port_refinement(&plan.requirements[0].id);
    if let Some(Dialog::Flow(FlowDialog::Port(d))) = &mut a.dialog {
        d.define_new = true;
        d.definition = Contract::draft("Input".into());
    }
    button(&mut a, &ctx, "Review complete edit");
    button(&mut a, &ctx, "Cancel — discard draft");
    assert_eq!(a.store.project(), &original);
    assert_eq!(a.store.generation, 0);
    a.open_port_refinement(&plan.requirements[0].id);
    if let Some(Dialog::Flow(FlowDialog::Port(d))) = &mut a.dialog {
        d.define_new = true;
        d.definition = Contract::draft("Input".into());
    }
    button(&mut a, &ctx, "Review complete edit");
    button(&mut a, &ctx, "Apply contract refinement");
    assert!(a.dialog.is_none(), "{:?}", a.error);
    assert_eq!(a.store.generation, 1);
    assert!(a.store.project().behavior[ROOT].data[0].contract.is_some());
    assert_eq!(
        a.store.project().behavior[ROOT].data[0].contract,
        a.store.project().behavior[&plan.component].data[0].contract
    );
}
#[test]
fn raw_multiselect_and_explicit_start_promotion_are_undoable() {
    let ctx = egui::Context::default();
    let mut a = Designer::blank();
    let original = a.store.project().clone();
    a.dialog = Some(Dialog::Flow(FlowDialog::Start));
    button(&mut a, &ctx, "Start flow");
    assert_eq!(a.store.project().version, 2);
    frame(&mut a, &ctx, vec![], false);
    let action = a.flow.rects["action"].center();
    click(&mut a, &ctx, action, Modifiers::NONE, false);
    let done = a.flow.rects["done"].center();
    click(&mut a, &ctx, done, Modifiers::SHIFT, false);
    assert_eq!(a.flow.selection.steps.len(), 2);
    a.store.undo();
    a.normalize_selection();
    assert_eq!(a.store.project(), &original);
    assert!(a.flow.selection.steps.is_empty());
}
#[test]
fn document_load_resets_scope_and_layer_state_and_preserves_v1() {
    let ctx = egui::Context::default();
    let mut a = app();
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("legacy.json");
    let p = Project::blank();
    storage::save_project(&path, &p, None).expect("save");
    a.flow.selection.step("action".into(), false);
    a.go_scope(ROOT.into(), Layer::Interfaces);
    a.load(LoadAction::Open(path), &ctx);
    assert_eq!(a.layer, Layer::Interfaces);
    assert!(!a.can_go_back());
    assert_eq!(a.store.project().version, 1);
    assert!(!a.dirty());
    a.go_scope(ROOT.into(), Layer::Flow);
    assert!(!a.dirty());
    assert_eq!(a.store.project().version, 1);
}

#[test]
fn full_workspace_modal_resizes_after_short_start_dialog() {
    let ctx = egui::Context::default();
    let mut a = app();
    let draw = |a: &mut Designer| {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1500.0, 1000.0))),
                focused: true,
                ..Default::default()
            },
            |ctx| a.draw(ctx),
        )
    };
    a.dialog = Some(Dialog::Flow(FlowDialog::Start));
    draw(&mut a);
    draw(&mut a);
    a.dialog = Some(Dialog::Flow(FlowDialog::Step(
        a.store.project().behavior[ROOT].steps[1].clone(),
    )));
    draw(&mut a);
    let out = draw(&mut a);
    assert!(text_position(&out, "Save step").is_some());
    assert!(text_position(&out, "Cancel — discard draft").is_some());
}

#[test]
fn leaf_rendering_does_not_report_an_empty_selection_as_removed() {
    let ctx = egui::Context::default();
    let mut a = app();
    let (p, id) = edit::add_node(a.store.project(), &a.current).expect("leaf");
    a.store = Store::new(p).expect("store");
    a.go_scope(id, Layer::Flow);
    a.status = "Entered component".into();
    frame(&mut a, &ctx, vec![], false);
    assert_eq!(a.status, "Entered component");
}
