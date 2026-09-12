use super::*;
use model::{INTERFACES, STUDY, complexity};
use serde_json::Value;

fn doc() -> Document {
    Document::load().expect("bound study fixture")
}
fn graph(n: &[&str], e: &[(&str, &str)], entry: &str, outcomes: &[&str]) -> Result<usize, String> {
    complexity(
        &n.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        &e.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect::<Vec<_>>(),
        entry,
        &outcomes.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    )
}
fn rejects(change: impl FnOnce(&mut Document)) {
    let mut d = doc();
    change(&mut d);
    assert!(d.validate().is_err());
}
#[test]
fn fixture_validates_with_production_model() {
    assert!(doc().validate().is_ok());
}
#[test]
fn source_is_the_exact_original_s2_object() {
    assert_eq!(
        doc().study.source,
        serde_json::from_str::<Value>(model::SOURCE).unwrap()
    );
}
#[test]
fn original_has_ten_steps_eleven_transitions() {
    let f = doc().original();
    assert_eq!((f.steps.len(), f.transitions.len()), (10, 11));
}
#[test]
fn decomposed_parent_has_eight_steps() {
    assert_eq!(doc().study.flows[0].local_steps(), 8);
}
#[test]
fn subprocess_keeps_three_distinct_material_steps() {
    let d = doc();
    let f = &d.study.flows[1];
    assert_eq!(f.local_steps(), 3);
    assert_eq!(f.steps.len(), 5);
}
#[test]
fn boundary_does_not_hide_cyclomatic_decisions() {
    let d = doc();
    assert_eq!(d.original().metric(), Ok(4));
    assert_eq!(d.study.flows[0].metric(), Ok(4));
    assert_eq!(d.study.flows[1].metric(), Ok(1));
}
#[test]
fn old_parser_rejects_experimental_study() {
    assert!(system_designer::model::parse(STUDY).is_err());
}
#[test]
fn interface_file_remains_version_one() {
    let p = system_designer::model::parse(INTERFACES).unwrap();
    assert_eq!(p.version, 1);
    assert_eq!(p.systems.len(), 2);
}
#[test]
fn interface_snapshot_roundtrips_without_data_loss() {
    let d = doc();
    let text = serde_json::to_string(&d.project).unwrap();
    assert_eq!(system_designer::model::parse(&text).unwrap(), d.project);
}
#[test]
fn one_transition_retains_two_distinct_contracts() {
    let d = doc();
    let t = d.study.flows[0].transition("S2.e3").unwrap();
    assert_eq!(t.exchanges.len(), 2);
    assert_eq!(
        d.source_edge("S2.e3").unwrap().contracts,
        vec!["C07", "C11"]
    );
}
#[test]
fn renewal_preserves_both_decision_and_grant() {
    let d = doc();
    let e = d.source_edge("S2.e10").unwrap();
    assert_eq!(e.contracts, vec!["C03", "C04"]);
    assert_eq!(e.condition, "inconclusive; renewed bound");
}
#[test]
fn retry_is_not_an_unconditional_loop() {
    rejects(|d| {
        d.study.flows[0]
            .transitions
            .iter_mut()
            .find(|e| e.id == "S2.e10")
            .unwrap()
            .condition
            .clear()
    });
}
#[test]
fn honest_stop_is_not_removed() {
    rejects(|d| d.study.flows[0].transitions.retain(|e| e.id != "S2.e11"));
}
#[test]
fn prior_art_bypass_is_not_removed() {
    rejects(|d| d.study.flows[0].transitions.retain(|e| e.id != "S2.e3"));
}
#[test]
fn sealed_plan_cannot_be_bypassed() {
    rejects(|d| {
        d.study.flows[1]
            .transitions
            .iter_mut()
            .find(|e| e.id == "S2.e5")
            .unwrap()
            .to = "step.S2.run".into()
    });
}
#[test]
fn source_step_text_is_not_silently_rewritten() {
    rejects(|d| d.study.flows[0].steps[0].purpose = "Invented rule".into());
}
#[test]
fn missing_exchange_is_rejected() {
    rejects(|d| d.study.flows[0].transitions[0].exchanges[0] = "not-an-edge".into());
}
#[test]
fn removing_one_payload_from_a_handoff_is_rejected() {
    rejects(|d| {
        d.study.flows[0]
            .transitions
            .iter_mut()
            .find(|e| e.id == "S2.e3")
            .unwrap()
            .exchanges
            .pop();
    });
}
#[test]
fn source_catalog_edit_is_detected() {
    rejects(|d| d.project.contracts[0].purpose.push_str("New semantics"));
}
#[test]
fn cross_level_step_reference_is_rejected() {
    rejects(|d| d.study.flows[0].steps[0].component = Some("S2.hyp".into()));
}
#[test]
fn duplicate_occurrence_identity_is_rejected() {
    rejects(|d| d.study.flows[0].steps[1].id = d.study.flows[0].steps[0].id.clone());
}
#[test]
fn dangling_control_endpoint_is_rejected() {
    rejects(|d| d.study.flows[0].transitions[0].to = "missing-step".into());
}
#[test]
fn wrong_child_flow_is_rejected() {
    rejects(|d| {
        d.study.flows[0]
            .steps
            .iter_mut()
            .find(|s| s.kind == Kind::Call)
            .unwrap()
            .call = Some("behavior.S2".into())
    });
}
#[test]
fn boundary_physical_identity_is_checked() {
    rejects(|d| d.study.call_binding.entry = "step.S2.hyp".into());
}
#[test]
fn wrong_return_binding_is_rejected() {
    rejects(|d| d.study.call_binding.r#return = "step.S2.run".into());
}
#[test]
fn malformed_missing_digest_is_error_not_panic() {
    rejects(|d| {
        d.study.provenance.remove("source_S2_sha256");
    });
}
#[test]
fn empty_scopes_are_error_not_panic() {
    rejects(|d| d.study.flows.clear());
}
#[test]
fn metric_chain_is_one() {
    assert_eq!(
        graph(&["a", "b", "c"], &[("a", "b"), ("b", "c")], "a", &["c"]),
        Ok(1)
    );
}
#[test]
fn metric_binary_choice_is_two() {
    assert_eq!(
        graph(
            &["a", "b", "c", "d"],
            &[("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")],
            "a",
            &["d"]
        ),
        Ok(2)
    );
}
#[test]
fn metric_normalizes_multiple_outcomes() {
    assert_eq!(
        graph(
            &["a", "b", "c"],
            &[("a", "b"), ("a", "c")],
            "a",
            &["b", "c"]
        ),
        Ok(2)
    );
}
#[test]
fn metric_preserves_retry_edge() {
    assert_eq!(
        graph(
            &["a", "b", "c"],
            &[("a", "b"), ("b", "a"), ("b", "c")],
            "a",
            &["c"]
        ),
        Ok(2)
    );
}
#[test]
fn metric_rejects_disconnected_work_instead_of_ignoring_it() {
    assert!(graph(&["a", "b", "c"], &[("a", "b")], "a", &["b"]).is_err());
}
#[test]
fn metric_refuses_nonterminating_component_without_outcome_path() {
    assert!(graph(&["a", "b"], &[("a", "b"), ("b", "a")], "a", &[]).is_err());
}
#[test]
fn metric_refuses_outgoing_outcome() {
    assert!(graph(&["a", "b"], &[("a", "b"), ("b", "a")], "a", &["b"]).is_err());
}
#[test]
fn path_picking_uses_drawn_polyline() {
    let p = [
        egui::pos2(0.0, 0.0),
        egui::pos2(10.0, 0.0),
        egui::pos2(10.0, 10.0),
    ];
    assert_eq!(view::length(&p), 20.0);
    assert_eq!(view::at(&p, 15.0).0, egui::pos2(10.0, 5.0));
    assert_eq!(view::distance(&p, egui::pos2(8.0, 5.0)), 2.0);
}
#[test]
fn enter_and_back_restore_exact_view_and_selection() {
    let mut a = App::new(doc());
    a.at.selection = Selection::Transition("S2.e3".into());
    a.at.tab = Tab::Interfaces;
    a.enter(1);
    a.back();
    assert_eq!(a.at.scope, 0);
    assert_eq!(a.at.tab, Tab::Interfaces);
    assert_eq!(a.at.selection, Selection::Transition("S2.e3".into()));
}
#[test]
fn interface_selection_maps_to_control_transition() {
    let mut a = App::new(doc());
    a.at.tab = Tab::Interfaces;
    a.at.selection = Selection::Exchange("exchange.S2.e3.C11".into());
    a.set_tab(Tab::Control);
    assert_eq!(a.at.selection, Selection::Transition("S2.e3".into()));
}
#[test]
fn original_internal_step_locates_correct_interface_scope() {
    let mut a = App::new(doc());
    a.at.original = true;
    a.at.selection = Selection::Step("step.S2.plan".into());
    a.set_tab(Tab::Interfaces);
    assert_eq!(a.at.scope, 1);
    assert!(!a.at.original);
}
#[test]
fn views_and_selections_never_change_project_or_exports() {
    let mut a = App::new(doc());
    let before = serde_json::to_value(&a.document.project).unwrap();
    let study = serde_json::to_value(&a.document.study).unwrap();
    for scope in 0..2 {
        for tab in [Tab::Control, Tab::Interfaces] {
            for selection in [
                Selection::None,
                Selection::Transition("S2.e3".into()),
                Selection::Contract("C07".into()),
            ] {
                a.at.scope = scope;
                a.set_tab(tab);
                a.at.selection = selection;
                assert_eq!(serde_json::to_value(&a.document.project).unwrap(), before);
                assert_eq!(serde_json::to_value(&a.document.study).unwrap(), study);
            }
        }
    }
}
#[test]
fn native_egui_views_render_without_a_window() {
    let mut a = App::new(doc());
    let ctx = egui::Context::default();
    for (scope, original, tab) in [
        (0, false, Tab::Control),
        (0, false, Tab::Interfaces),
        (1, false, Tab::Control),
        (1, false, Tab::Interfaces),
        (0, true, Tab::Control),
    ] {
        a.at.scope = scope;
        a.at.original = original;
        a.at.tab = tab;
        a.at.selection = Selection::None;
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1800.0, 1080.0),
                )),
                ..Default::default()
            },
            |ctx| a.show(ctx),
        );
        assert!(!output.shapes.is_empty());
    }
}
#[test]
fn embedded_prompt_names_the_correct_protocol_boundary() {
    assert!(model::PROMPT.contains("not a version-1 replacement packet"));
    assert!(model::PROMPT.contains("Do not infer permission"));
}
