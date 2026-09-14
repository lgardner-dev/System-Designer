//! Full workspace raw-input regressions (egui, no OS/window renderer).
use super::*;
use crate::behavior::{self, Flow, ROOT};
use egui::{Event, Modifiers, Pos2, Rect, vec2};
fn app(layer: Layer) -> Designer {
    let mut a = Designer::blank();
    let p = parse(include_str!("../../tests/fixtures/project.json")).expect("fixture");
    let p = behavior::set(&p, ROOT, Flow::starter()).expect("flow");
    a.current = p.root.clone();
    a.store = Store::new(p.clone()).expect("store");
    a.saved = Some(p);
    a.layer = layer;
    a
}
fn frame(
    a: &mut Designer,
    ctx: &egui::Context,
    size: egui::Vec2,
    events: Vec<Event>,
    focused: bool,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            focused,
            ..Default::default()
        },
        |ctx| a.draw(ctx),
    )
}
fn locate(out: &egui::FullOutput, label: &str) -> Pos2 {
    fn find(s: &egui::Shape, label: &str) -> Option<Pos2> {
        match s {
            egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + t.galley.size() * 0.5),
            egui::Shape::Vec(v) => v.iter().find_map(|s| find(s, label)),
            _ => None,
        }
    }
    out.shapes
        .iter()
        .find_map(|s| find(&s.shape, label))
        .unwrap_or_else(|| panic!("missing visible {label}"))
}
fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn click(a: &mut Designer, ctx: &egui::Context, label: &str) {
    let size = vec2(1600.0, 960.0);
    for _ in 0..3 {
        frame(a, ctx, size, vec![], true);
    }
    let at = locate(&frame(a, ctx, size, vec![], true), label);
    for pressed in [true, false] {
        frame(
            a,
            ctx,
            size,
            vec![Event::PointerMoved(at), pointer(at, pressed)],
            true,
        );
    }
}
#[test]
fn global_settings_preserves_scope_layer_camera_selection_and_publishes_last_character_once() {
    for layer in [Layer::Flow, Layer::Interfaces] {
        for nested in [false, true] {
            let mut a = app(layer);
            if nested {
                let id = a.store.project().systems[0].nodes[0].id.clone();
                a.go_scope(id, layer);
            }
            a.canvas.pan = vec2(-123.0, 89.0);
            a.canvas.zoom = 0.6;
            a.canvas.fit_requested = false;
            let before = a.store.project().clone();
            let view = (
                a.current.clone(),
                a.owner.clone(),
                a.layer,
                a.canvas.pan,
                a.canvas.zoom,
                a.selected.clone(),
                a.flow.selection.clone(),
            );
            let ctx = egui::Context::default();
            click(&mut a, &ctx, "Project settings…");
            click(&mut a, &ctx, "Cancel");
            assert_eq!(a.store.project(), &before);
            assert_eq!(a.store.generation, 0);
            assert_eq!(
                (
                    a.current.clone(),
                    a.owner.clone(),
                    a.layer,
                    a.canvas.pan,
                    a.canvas.zoom,
                    a.selected.clone(),
                    a.flow.selection.clone()
                ),
                view
            );
            click(&mut a, &ctx, "Project settings…");
            if let Some(Dialog::Project { purpose, .. }) = &mut a.dialog {
                *purpose = "Intent".into();
            }
            for _ in 0..3 {
                frame(&mut a, &ctx, vec2(1600.0, 960.0), vec![], true);
            }
            ctx.memory_mut(|m| m.request_focus(egui::Id::new("project_purpose")));
            let out = frame(&mut a, &ctx, vec2(1600.0, 960.0), vec![], true);
            let save = locate(&out, "Save details");
            frame(
                &mut a,
                &ctx,
                vec2(1600.0, 960.0),
                vec![
                    Event::Text("!".into()),
                    Event::PointerMoved(save),
                    pointer(save, true),
                ],
                true,
            );
            frame(
                &mut a,
                &ctx,
                vec2(1600.0, 960.0),
                vec![pointer(save, false)],
                true,
            );
            assert!(a.dialog.is_none(), "{:?}", a.error);
            assert!(
                a.store.project().purpose.contains('!'),
                "same-frame text lost"
            );
            assert_eq!(a.store.generation, 1);
            assert!(a.dirty());
            a.store.undo();
            assert_eq!(a.store.project(), &before);
            a.store.redo();
            assert!(a.store.project().purpose.contains('!'));
        }
    }
}
#[test]
fn modal_actions_remain_fixed_at_long_body_top_middle_bottom_and_ui_scales() {
    for size in [
        vec2(1280.0, 720.0),
        vec2(1600.0, 960.0),
        vec2(1920.0, 1080.0),
    ] {
        for scale in [1.0, 1.5] {
            let ctx = egui::Context::default();
            ctx.set_zoom_factor(scale);
            let mut a = app(Layer::Flow);
            a.project_settings();
            if let Some(Dialog::Project { purpose, .. }) = &mut a.dialog {
                *purpose = "Long purpose paragraph with constraints and intent.\n".repeat(180);
            }
            for _ in 0..4 {
                frame(&mut a, &ctx, size, vec![], true);
            }
            let out = frame(&mut a, &ctx, size, vec![], true);
            let save = locate(&out, "Save details");
            let cancel = locate(&out, "Cancel");
            assert!(ctx.content_rect().contains(save) && ctx.content_rect().contains(cancel));
            let logical = ctx.content_rect().size();
            assert!(save.x > logical.x * 0.5 && save.y < logical.y * 0.35);
            for _ in 0..5 {
                let out = frame(
                    &mut a,
                    &ctx,
                    size,
                    vec![
                        Event::PointerMoved(logical.to_pos2() * 0.7),
                        Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: vec2(0.0, -500.0),
                            modifiers: Modifiers::NONE,
                        },
                    ],
                    true,
                );
                assert!(locate(&out, "Save details").distance(save) < 1.0);
                assert!(locate(&out, "Cancel").distance(cancel) < 1.0);
            }
            assert_eq!(a.store.generation, 0);
        }
    }
}
#[test]
fn both_layers_capture_signed_moves_and_cancel_on_focus_loss() {
    for layer in [Layer::Flow, Layer::Interfaces] {
        let ctx = egui::Context::default();
        let mut a = app(layer);
        let size = vec2(1600.0, 960.0);
        for _ in 0..3 {
            frame(&mut a, &ctx, size, vec![], true);
        }
        let (id, rect, initial) = if layer == Layer::Flow {
            (
                "action".to_owned(),
                a.flow.rects["action"],
                behavior::positions(&a.store.project().behavior[ROOT])["action"],
            )
        } else {
            let id = a.store.project().systems[0].nodes[0].id.clone();
            let initial = a
                .store
                .project()
                .layout
                .get(&a.current)
                .and_then(|m| m.get(&id))
                .copied()
                .unwrap_or_else(|| auto_layout(a.store.project(), &a.current)[&id]);
            (id.clone(), a.canvas.rects[&id], initial)
        };
        let start = rect.center();
        let target =
            start - vec2((initial.x + 150.0) as f32, (initial.y + 180.0) as f32) * a.canvas.zoom;
        let before = a.store.project().clone();
        frame(
            &mut a,
            &ctx,
            size,
            vec![Event::PointerMoved(start), pointer(start, true)],
            true,
        );
        frame(&mut a, &ctx, size, vec![Event::PointerMoved(target)], true);
        assert_eq!(a.store.project(), &before);
        frame(&mut a, &ctx, size, vec![pointer(target, false)], true);
        assert_eq!(a.store.generation, 1, "layer {layer:?}: {:?}", a.error);
        let positions = if layer == Layer::Flow {
            &a.store.project().flow_layout[ROOT]
        } else {
            &a.store.project().layout[&a.current]
        };
        assert!((positions[&id].x + 150.0).abs() < 0.01 && (positions[&id].y + 180.0).abs() < 0.01);
        assert_eq!(a.store.project().version, 3);
        a.store.undo();
        assert_eq!(a.store.project(), &before);
        frame(&mut a, &ctx, size, vec![], true);
        frame(
            &mut a,
            &ctx,
            size,
            vec![Event::PointerMoved(start), pointer(start, true)],
            true,
        );
        frame(&mut a, &ctx, size, vec![Event::PointerMoved(target)], true);
        frame(&mut a, &ctx, size, vec![], false);
        frame(&mut a, &ctx, size, vec![pointer(target, false)], true);
        assert_eq!(a.store.project(), &before);
        assert!(!a.canvas.has_gesture() && !a.flow.has_gesture());
    }
}
#[test]
fn lights_popup_does_not_clear_selection_or_capture_background() {
    for layer in [Layer::Flow, Layer::Interfaces] {
        let mut a = app(layer);
        let ctx = egui::Context::default();
        a.flow.selection.steps.insert("action".into());
        a.selected = Selection::Node(a.store.project().systems[0].nodes[0].id.clone());
        let before = (
            a.selected.clone(),
            a.flow.selection.clone(),
            a.store.project().clone(),
        );
        click(&mut a, &ctx, "All visible");
        click(&mut a, &ctx, "Off");
        assert_eq!(
            (
                a.selected.clone(),
                a.flow.selection.clone(),
                a.store.project().clone()
            ),
            before
        );
        assert!(!a.canvas.has_gesture() && !a.flow.has_gesture());
        assert_eq!(a.store.generation, 0);
    }
}

#[test]
fn nudge_uses_the_key_event_chord_after_same_frame_modifier_release() {
    for layer in [Layer::Flow, Layer::Interfaces] {
        let mut a = app(layer);
        let ctx = egui::Context::default();
        let size = vec2(1600.0, 960.0);
        for _ in 0..3 {
            frame(&mut a, &ctx, size, vec![], true);
        }
        let (id, initial) = if layer == Layer::Flow {
            a.flow.selection.steps.insert("action".into());
            (
                "action".to_owned(),
                behavior::positions(&a.store.project().behavior[ROOT])["action"],
            )
        } else {
            let id = a.store.project().systems[0].nodes[0].id.clone();
            a.selected = Selection::Node(id.clone());
            let initial = auto_layout(a.store.project(), &a.current)[&id];
            (id, initial)
        };
        for (modifiers, expected, generation) in [
            (Modifiers::SHIFT, 10.0, 1),
            (Modifiers::NONE, 11.0, 2),
            (Modifiers::CTRL, 11.0, 2),
        ] {
            let events = [true, false]
                .into_iter()
                .map(|pressed| Event::Key {
                    key: egui::Key::ArrowRight,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: if pressed { modifiers } else { Modifiers::NONE },
                })
                .collect();
            frame(&mut a, &ctx, size, events, true);
            let position = if layer == Layer::Flow {
                a.store.project().flow_layout[ROOT][&id]
            } else {
                a.store.project().layout[&a.current][&id]
            };
            assert_eq!(
                position,
                Position {
                    x: initial.x + expected,
                    y: initial.y
                }
            );
            assert_eq!(a.store.generation, generation);
        }
    }
}
