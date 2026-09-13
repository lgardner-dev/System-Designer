use super::*;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, vec2};
fn app() -> Designer {
    let mut a = Designer::blank();
    let p = behavior::set(a.store.project(), ROOT, Flow::starter()).expect("flow");
    a.store = Store::new(p.clone()).expect("store");
    a.saved = Some(p);
    a
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
    output
        .shapes
        .iter()
        .find_map(|s| find(&s.shape, label))

}
fn button(a: &mut Designer, ctx: &egui::Context, label: &str) {
    frame(a,ctx,vec![],true);
    for _ in 0..6 {
        let output = frame(a,ctx,vec![],true);
        if let Some(pos) = text_position(&output,label) {
            click(a,ctx,pos,Modifiers::NONE,true);return;
        }
        frame(a,ctx,vec![Event::PointerMoved(egui::pos2(750.0,500.0)),Event::MouseWheel{unit:egui::MouseWheelUnit::Point,delta:vec2(0.0,-350.0),modifiers:Modifiers::NONE}],true);
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
