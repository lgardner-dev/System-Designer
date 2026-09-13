//! Regression tests for the recovered behavior-model integration.
use std::collections::BTreeSet;
use system_designer::{
    behavior::*,
    edit::{self, Store},
    exchange,
    model::*,
};
fn fixture() -> Project {
    set(&Project::blank(), ROOT, Flow::starter()).unwrap()
}
fn selected() -> BTreeSet<String> {
    BTreeSet::from(["action".into()])
}
#[test]
fn legacy_stays_version_one() {
    let p = Project::blank();
    assert_eq!(p.version, 1);
    let text = serde_json::to_string(&p).unwrap();
    assert!(!text.contains("behavior"));
    assert_eq!(parse(&text).unwrap(), p);
}
#[test]
fn behavior_promotes_explicitly() {
    let p = fixture();
    assert_eq!(p.version, 2);
    assert_eq!(parse(&serde_json::to_string(&p).unwrap()).unwrap(), p);
}
#[test]
fn v1_cannot_hide_behavior() {
    let mut p = fixture();
    p.version = 1;
    assert!(validate(&p).is_err());
}
#[test]
fn draft_decision_is_saved_but_diagnosed() {
    let mut p = fixture();
    p.behavior.get_mut(ROOT).unwrap().steps[1].kind = StepKind::Decision;
    validate(&p).unwrap();
    assert!(issues(&p, ROOT).iter().any(|s| s.contains("alternatives")));
}
#[test]
fn dangling_transition_rejected() {
    let mut p = fixture();
    p.behavior.get_mut(ROOT).unwrap().transitions[0].to = "missing".into();
    assert!(validate(&p).is_err());
}
#[test]
fn one_region_produces_one_shared_child_identity() {
    let p = fixture();
    let plan = preview(
        &p,
        ROOT,
        &selected(),
        "Worker",
        "Perform the named operation",
    )
    .unwrap();
    let q = apply(&p, &plan).unwrap();
    assert_eq!(q.system(&q.root).unwrap().nodes.len(), 1);
    assert_eq!(
        q.behavior[ROOT].step(&plan.call).unwrap().target.as_ref(),
        Some(&plan.component)
    );
    assert!(q.behavior[&plan.component].step("action").is_some());
    assert!(q.behavior[ROOT].step("action").is_none());
    assert!(q.node(&plan.component).unwrap().1.child.is_none());
}
#[test]
fn extraction_needs_responsibility() {
    assert!(preview(&fixture(), ROOT, &selected(), "Worker", "").is_err());
}
#[test]
fn repeat_preview_is_deterministic() {
    let p = fixture();
    let a = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    let b = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    assert_eq!(apply(&p, &a).unwrap(), apply(&p, &b).unwrap());
}
#[test]
fn changing_layout_invalidates_extraction_preview() {
    let p = fixture();
    let plan = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    let mut q = p.clone();
    q.flow_layout
        .entry(ROOT.into())
        .or_default()
        .insert("action".into(), Position { x: 42.0, y: 20.0 });
    assert!(apply(&q, &plan).is_err());
}
#[test]
fn invalid_extraction_does_not_mutate_store() {
    let p = fixture();
    let store = Store::new(p.clone()).unwrap();
    assert!(
        preview(
            store.project(),
            ROOT,
            &BTreeSet::from(["entry".into()]),
            "Bad",
            "Bad boundary"
        )
        .is_err()
    );
    assert_eq!(store.project(), &p);
}
#[test]
fn extraction_has_one_undoable_publication() {
    let p = fixture();
    let plan = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    let q = apply(&p, &plan).unwrap();
    let mut store = Store::new(p.clone()).unwrap();
    store.publish("Extract", q.clone()).unwrap();
    assert!(store.undo());
    assert_eq!(store.project(), &p);
    assert!(store.redo());
    assert_eq!(store.project(), &q);
}
#[test]
fn leaf_can_be_refined_recursively() {
    let p = fixture();
    let a = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    let q = apply(&p, &a).unwrap();
    let b = preview(
        &q,
        &a.component,
        &selected(),
        "Primitive",
        "Specify the operation",
    )
    .unwrap();
    let r = apply(&q, &b).unwrap();
    assert!(system(&r, &a.component).is_some());
    assert_eq!(
        r.behavior[&a.component]
            .step(&b.call)
            .unwrap()
            .target
            .as_ref(),
        Some(&b.component)
    );
}
#[test]
fn explicit_dependency_becomes_a_draft_boundary_port() {
    let mut p = fixture();
    p.behavior.get_mut(ROOT).unwrap().data.push(DataLink {
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
    });
    let plan = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    let q = apply(&p, &plan).unwrap();
    let port = &q.node(&plan.component).unwrap().1.ports[0];
    assert_eq!(port.name, "Input record");
    assert!(port.contract.is_none());
    assert_eq!(q.behavior[ROOT].data[0].to.port.as_ref(), Some(&port.id));
    assert_eq!(
        q.behavior[&plan.component].data[0].from.port.as_ref(),
        Some(&port.id)
    );
}
#[test]
fn behavior_exchange_preserves_interfaces() {
    let p = fixture();
    let mut packet = export(&p, ROOT).unwrap();
    packet["flow"]["steps"][1]["name"] = "Renamed work".into();
    let q = replace(&p, &packet, ROOT).unwrap();
    assert_eq!(p.systems, q.systems);
    assert_eq!(p.contracts, q.contracts);
    assert_eq!(q.behavior[ROOT].steps[1].name, "Renamed work");
}
#[test]
fn stale_behavior_exchange_rejected() {
    let p = fixture();
    let packet = export(&p, ROOT).unwrap();
    let mut q = p.clone();
    q.behavior.get_mut(ROOT).unwrap().steps[1].name = "Another edit".into();
    assert!(replace(&q, &packet, ROOT).is_err());
}
#[test]
fn behavior_exchange_context_is_read_only() {
    let p = fixture();
    let mut packet = export(&p, ROOT).unwrap();
    packet["context"]["project"]["name"] = "Other name".into();
    assert!(replace(&p, &packet, ROOT).is_err());
}
#[test]
fn simple_metric_is_one() {
    assert_eq!(complexity(&Flow::starter()), Some(1));
}
#[test]
fn no_metric_claim_for_unreachable_draft() {
    let mut f = Flow::starter();
    f.steps
        .push(Step::new("orphan", StepKind::Action, "Detached"));
    assert_eq!(complexity(&f), None);
}

#[test]
fn interface_replacement_preserves_behavior() {
    let p = fixture();
    let root = p.root.clone();
    let (p, component) = edit::add_node(&p, &root).expect("add component");
    let p = set(&p, &component, Flow::starter()).expect("add component behavior");
    let behavior = p.behavior.clone();
    let mut packet = exchange::export(&p, &root, exchange::Scope::Component, Some(&component))
        .expect("export component interface");
    packet["component"]["purpose"] = "Reviewed interface".into();

    let q = exchange::replace(&p, &packet, &root).expect("replace component interface");

    assert_eq!(q.behavior, behavior);
    assert_eq!(q.version, 2);
    assert_eq!(
        q.node(&component).expect("component").1.purpose,
        "Reviewed interface"
    );
}

#[test]
fn interface_packet_stales_when_preserved_behavior_changes() {
    let p = fixture();
    let root = p.root.clone();
    let (p, component) = edit::add_node(&p, &root).expect("add component");
    let packet = exchange::export(&p, &root, exchange::Scope::Component, Some(&component))
        .expect("export component interface");
    let mut changed = p.clone();
    changed.behavior.get_mut(ROOT).expect("root behavior").steps[1].name =
        "Concurrent behavior edit".into();

    assert!(exchange::replace(&changed, &packet, &root).is_err());
}

#[test]
fn behavior_data_prevents_contract_deletion() {
    let mut p = Project::blank();
    let contract = Contract::draft("record".into());
    let reference = contract.reference();
    p.contracts.push(contract);
    let mut flow = Flow::starter();
    flow.data.push(DataLink {
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
        contract: Some(reference.clone()),
        exchange: None,
    });
    let p = set(&p, ROOT, flow).expect("add behavior");

    assert!(edit::delete_contract(&p, &reference).is_err());
}

#[test]
fn extraction_preserves_each_explicit_outcome_path() {
    let mut p = fixture();
    let flow = p.behavior.get_mut(ROOT).expect("root behavior");
    flow.steps[1].kind = StepKind::Decision;
    flow.steps
        .push(Step::new("rejected", StepKind::Outcome, "Rejected"));
    flow.transitions[1].condition = "accepted".into();
    flow.transitions.push(Transition {
        id: "reject".into(),
        from: "action".into(),
        to: "rejected".into(),
        condition: "rejected".into(),
        outcome: None,
    });
    let plan = preview(&p, ROOT, &selected(), "Decider", "Choose the outcome")
        .expect("preview extraction");
    let q = apply(&p, &plan).expect("apply extraction");
    let parent = &q.behavior[ROOT];
    let child = &q.behavior[&plan.component];

    assert_eq!(
        child
            .steps
            .iter()
            .filter(|step| step.kind == StepKind::Outcome)
            .count(),
        2
    );
    for transition_id in ["finish", "reject"] {
        let outer = parent
            .transitions
            .iter()
            .find(|transition| transition.id == transition_id)
            .expect("parent transition");
        let inner = child
            .transitions
            .iter()
            .find(|transition| transition.id == transition_id)
            .expect("child transition");
        assert_eq!(outer.from, plan.call);
        assert_eq!(inner.outcome, None);
        assert_eq!(outer.outcome.as_deref(), Some(inner.to.as_str()));
    }
}
