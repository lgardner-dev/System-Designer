use super::*;
use sha2::{Digest, Sha256};
use system_designer::{
    exchange::{self, Scope},
    model::*,
};
fn atlas() -> Atlas {
    Atlas::load().expect("valid authored atlas")
}
#[test]
fn authored_design_validates() {
    atlas().check().unwrap();
}
#[test]
fn generated_interface_projection_matches_exactly() {
    let a = atlas();
    let p = parse(system_designer::APPLICATION_DESIGN).unwrap();
    assert_eq!(a.project, p);
}
#[test]
fn all_components_including_root_and_leaves_have_behavior() {
    let a = atlas();
    assert_eq!(a.scopes.len(), 52);
    assert_eq!(
        a.project
            .systems
            .iter()
            .map(|s| s.nodes.len())
            .sum::<usize>(),
        51
    );
    for s in &a.project.systems {
        for n in &s.nodes {
            assert!(a.scope(&n.id).is_some());
        }
    }
    assert!(a.scope("@root").is_some());
}
#[test]
fn readability_budget_is_met_by_this_design() {
    let a = atlas();
    assert!(a.project.systems.iter().all(|s| s.nodes.len() <= 8));
    assert!(a.scopes.iter().all(|s| s.work_steps() <= 8));
}
#[test]
fn scope_metrics_are_positive() {
    for s in atlas().scopes {
        assert!(s.metric() > 0);
    }
}
#[test]
fn leaf_primitive_rules_are_explicit() {
    let a = atlas();
    for b in &a.scopes {
        if a.system(&b.owner).is_none() {
            assert!(b.primitive_stop.as_deref().is_some_and(|v| !v.is_empty()));
            assert!(b.steps.iter().all(|s| s.target.is_none()));
        }
    }
}
#[test]
fn five_level_validation_path_is_present() {
    let a = atlas();
    assert_eq!(
        a.lineage("model.validation.edges.contracts"),
        vec![
            "@root",
            "model",
            "model.validation",
            "model.validation.edges",
            "model.validation.edges.contracts"
        ]
    );
}
#[test]
fn exact_contract_primitive_includes_assignment_id_and_version() {
    let a = atlas();
    let s = a
        .scope("model.validation.edges.contracts")
        .unwrap()
        .step("check")
        .unwrap();
    assert!(s.rule.contains("is_some"));
    assert!(s.rule.contains("ID and version"));
}
#[test]
fn no_dangling_or_cross_scope_call_is_accepted() {
    let mut a = atlas();
    a.scopes[0]
        .steps
        .iter_mut()
        .find(|s| s.kind == super::model::Kind::Call)
        .unwrap()
        .target = Some("model.validation.edges.contracts".into());
    assert!(a.check().is_err());
}
#[test]
fn duplicate_owner_is_rejected() {
    let mut a = atlas();
    a.scopes.push(a.scopes[0].clone());
    assert!(a.check().is_err());
}
#[test]
fn missing_leaf_behavior_is_rejected() {
    let mut a = atlas();
    a.scopes.pop();
    assert!(a.check().is_err());
}
#[test]
fn unlabelled_choice_is_rejected() {
    let mut a = atlas();
    let b = &mut a.scopes[0];
    let id = b
        .steps
        .iter()
        .find(|s| s.kind == super::model::Kind::Decision)
        .unwrap()
        .id
        .clone();
    b.transitions
        .iter_mut()
        .find(|t| t.from == id)
        .unwrap()
        .label
        .clear();
    assert!(a.check().is_err());
}
#[test]
fn cross_level_interface_binding_is_rejected() {
    let mut a = atlas();
    a.scopes[0].transitions[0]
        .exchanges
        .push("missing.edge".into());
    assert!(a.check().is_err());
}
#[test]
fn absent_stopping_rule_is_rejected() {
    let mut a = atlas();
    let i = a
        .scopes
        .iter()
        .position(|b| a.system(&b.owner).is_none())
        .unwrap();
    a.scopes[i].primitive_stop = None;
    assert!(a.check().is_err());
}
#[test]
fn unknown_atlas_fields_are_rejected() {
    let mut v: serde_json::Value = serde_json::from_str(model::ATLAS).unwrap();
    v["silent_extension"] = true.into();
    assert!(Atlas::parse(&v.to_string()).is_err());
}
#[test]
fn production_parser_rejects_atlas_envelope() {
    assert!(parse(model::ATLAS).is_err());
}
#[test]
fn source_hashes_match_inspected_files() {
    let a = atlas();
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (path, hash) in &a.source_hashes {
        // Git source uses LF; Windows checkout may materialize CRLF.
        // Normalize only that representation difference, not other content.
        let text = std::fs::read_to_string(base.join(path)).unwrap();
        let bytes = text.replace("\r\n", "\n");
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes.as_bytes())),
            *hash,
            "{path}"
        );
    }
}
#[test]
fn source_paths_and_symbols_are_locatable() {
    let a = atlas();
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for b in &a.scopes {
        for s in &b.sources {
            let text = std::fs::read_to_string(base.join(&s.path)).unwrap();
            let symbol = s.symbol.split("::").last().unwrap();
            assert!(text.contains(symbol), "{}: {}", s.path, s.symbol);
        }
    }
}
#[test]
fn full_interface_scope_exchanges_round_trip() {
    let a = atlas();
    for s in &a.project.systems {
        for scope in [Scope::Level, Scope::Subtree] {
            let packet = exchange::export(&a.project, &s.id, scope, None).unwrap();
            let q = exchange::replace(&a.project, &packet, &s.id).unwrap();
            validate(&q).unwrap();
        }
        for n in &s.nodes {
            let packet =
                exchange::export(&a.project, &s.id, Scope::Component, Some(&n.id)).unwrap();
            let q = exchange::replace(&a.project, &packet, &s.id).unwrap();
            validate(&q).unwrap();
        }
    }
}
#[test]
fn navigation_preserves_model_and_exact_return_location() {
    let mut app = App::new(atlas());
    let before = serde_json::to_string(&app.atlas).unwrap();
    app.selection = Selection::Component("model".into());
    app.tab = Tab::Interfaces;
    app.navigate("model");
    app.navigate("model.validation");
    app.back();
    assert_eq!(app.owner, "model");
    app.back();
    assert_eq!(app.owner, "@root");
    assert_eq!(app.tab, Tab::Interfaces);
    assert_eq!(app.selection, Selection::Component("model".into()));
    assert_eq!(serde_json::to_string(&app.atlas).unwrap(), before);
}
#[test]
fn primitive_scope_navigation_does_not_require_a_child_system() {
    let mut app = App::new(atlas());
    app.navigate("model.validation.edges.contracts");
    assert!(app.atlas.system(&app.owner).is_none());
    assert!(app.atlas.scope(&app.owner).is_some());
}
#[test]
fn invalid_navigation_does_not_change_scope() {
    let mut app = App::new(atlas());
    app.navigate("missing");
    assert_eq!(app.owner, "@root");
}
#[test]
fn prompt_is_embedded_and_calls_out_protocol_boundary() {
    for term in [
        "primitive",
        "version-1",
        "immediate children",
        "Control Flow",
        "source_hashes",
    ] {
        assert!(model::PROMPT.contains(term), "missing {term}");
    }
}
#[test]
fn blank_production_project_still_starts_blank() {
    let p = Project::blank();
    assert!(p.systems[0].nodes.is_empty());
    assert!(p.contracts.is_empty());
}
#[test]
fn pending_cfg_work_is_not_labeled_implemented() {
    let a = atlas();
    assert!(!a.scope("model.types").unwrap().planned.is_empty());
    assert!(!a.scope("collaboration.replace").unwrap().planned.is_empty());
}
#[test]
fn graph_controls_can_draw_every_scope_without_project_mutation() {
    let ctx = egui::Context::default();
    let mut app = App::new(atlas());
    let before = serde_json::to_string(&app.atlas).unwrap();
    let scopes: Vec<_> = app.atlas.scopes.iter().map(|s| s.owner.clone()).collect();
    for owner in scopes {
        app.owner = owner;
        for tab in [Tab::Control, Tab::Interfaces] {
            app.tab = tab;
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1920.0, 1160.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| app.show(ctx));
        }
    }
    assert_eq!(before, serde_json::to_string(&app.atlas).unwrap());
}
