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

#[test]
fn r1_missing_flow_is_not_clear() {
    let p = fixture();
    let mut packet = export(&p, ROOT).unwrap();
    packet.as_object_mut().unwrap().remove("flow");
    assert!(replace(&p, &packet, ROOT).is_err());
}
#[test]
fn r2_reserve_crossing_transition_identity() {
    let mut p = fixture();
    p.behavior.get_mut(ROOT).unwrap().transitions[1].id = "transition.1".into();
    let plan = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    validate(&apply(&p, &plan).unwrap()).unwrap();
}
#[test]
fn r3_connection_refines_extracted_information_atomically() {
    let mut p = fixture();
    p.behavior.get_mut(ROOT).unwrap().data.push(DataLink {
        id: "input".into(),
        name: "Input".into(),
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
    let (q, source) = edit::add_node(&q, &q.root).unwrap();
    let q = edit::add_port(&q, &source, Direction::Out).unwrap();
    let from = Endpoint {
        node: Some(source.clone()),
        port: q.node(&source).unwrap().1.ports[0].id.clone(),
    };
    let to = Endpoint {
        node: Some(plan.component.clone()),
        port: plan.requirements[0].id.clone(),
    };
    let c = Contract::draft("Input".into());
    let r = c.reference();
    let q = edit::connect(
        &q,
        &q.root,
        &edit::Connection {
            id: None,
            from,
            to,
            label: "Input".into(),
            contract: r.clone(),
            new_contract: Some(c),
            consent: true,
        },
    )
    .unwrap();
    assert_eq!(q.behavior[ROOT].data[0].contract, Some(r.clone()));
    assert_eq!(q.behavior[&plan.component].data[0].contract, Some(r));
}
#[test]
fn r4_missing_and_duplicate_returns_are_draft_issues() {
    let p = fixture();
    let plan = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    let mut q = apply(&p, &plan).unwrap();
    q.behavior
        .get_mut(&plan.component)
        .unwrap()
        .steps
        .push(Step::new("rejection", StepKind::Outcome, "Rejected"));
    let mut duplicate = q.behavior[ROOT]
        .transitions
        .iter()
        .find(|t| t.from == plan.call)
        .unwrap()
        .clone();
    duplicate.id = "duplicate".into();
    q.behavior
        .get_mut(ROOT)
        .unwrap()
        .transitions
        .push(duplicate);
    validate(&q).unwrap();
    let messages = issues(&q, ROOT).join("\n");
    assert!(messages.contains("unhandled outcome"), "{messages}");
    assert!(messages.contains("duplicate unguarded"), "{messages}");
}

#[test]
fn explicit_clear_empty_and_rejected_packets_preserve_history() {
    let p = fixture();
    let mut store = Store::new(p.clone()).unwrap();
    let mut packet = export(&p, ROOT).unwrap();
    packet["flow"] = serde_json::Value::Null;
    let q = replace(&p, &packet, ROOT).unwrap();
    assert!(q.behavior.is_empty());
    assert_eq!(q.version, 2);
    store.publish("Clear", q).unwrap();
    store.undo();
    let generation = store.generation;
    let redo = store.redo_label().map(str::to_owned);
    packet.as_object_mut().unwrap().remove("flow");
    assert!(replace(store.project(), &packet, ROOT).is_err());
    assert_eq!(store.project(), &p);
    assert_eq!(store.generation, generation);
    assert_eq!(store.redo_label(), redo.as_deref());
    packet["flow"] = serde_json::to_value(Flow::default()).unwrap();
    assert!(
        replace(&p, &packet, ROOT)
            .unwrap()
            .behavior
            .contains_key(ROOT)
    );
    let plan = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    let q = apply(&p, &plan).unwrap();
    let mut packet = export(&q, &plan.component).unwrap();
    packet["flow"] = serde_json::Value::Null;
    assert!(replace(&q, &packet, &plan.component).is_err());
}

/// Expand only this bounded local-region transform, retaining original stable IDs.
/// Compare alternatives, descriptive labels and declared data uses, not execution.
fn assert_expansion(p: &Project, owner: &str, plan: &Extraction) {
    let q = apply(p, plan).unwrap();
    let original = &p.behavior[owner];
    let parent = &q.behavior[owner];
    let child = &q.behavior[&plan.component];
    let mut expanded = parent.clone();
    expanded.steps.retain(|s| s.id != plan.call);
    expanded.steps.extend(
        child
            .steps
            .iter()
            .filter(|s| plan.region.members.contains(&s.id))
            .cloned(),
    );
    for t in &mut expanded.transitions {
        if t.to == plan.call {
            t.to = plan.region.entry.clone();
        }
        if t.from == plan.call {
            let inner = child
                .transitions
                .iter()
                .find(|x| x.to == *t.outcome.as_ref().unwrap())
                .unwrap();
            t.from = inner.from.clone();
            t.condition = inner.condition.clone();
            t.outcome = inner.outcome.clone();
        }
    }
    expanded.transitions.extend(
        child
            .transitions
            .iter()
            .filter(|t| {
                plan.region.members.contains(&t.from) && plan.region.members.contains(&t.to)
            })
            .cloned(),
    );
    for d in &mut expanded.data {
        if d.from.step.as_ref() == Some(&plan.call) {
            d.from = child
                .data
                .iter()
                .find(|x| x.id == d.id)
                .unwrap()
                .from
                .clone();
        }
        if d.to.step.as_ref() == Some(&plan.call) {
            d.to = child.data.iter().find(|x| x.id == d.id).unwrap().to.clone();
        }
    }
    expanded.data.extend(
        child
            .data
            .iter()
            .filter(|d| {
                d.from
                    .step
                    .as_ref()
                    .is_some_and(|s| plan.region.members.contains(s))
                    && d.to
                        .step
                        .as_ref()
                        .is_some_and(|s| plan.region.members.contains(s))
            })
            .cloned(),
    );
    let mut expected = original.clone();
    for f in [&mut expanded, &mut expected] {
        f.steps.sort_by(|a, b| a.id.cmp(&b.id));
        f.transitions.sort_by(|a, b| a.id.cmp(&b.id));
        f.data.sort_by(|a, b| a.id.cmp(&b.id));
    }
    assert_eq!(expanded.steps, expected.steps);
    assert_eq!(expanded.transitions, expected.transitions);
    assert_eq!(expanded.data, expected.data);
}
#[test]
fn extraction_expansion_covers_loops_alternatives_merges_and_data_collisions() {
    for data_id in ["entry.1", "transition.1", "outcome.1"] {
        let mut p = fixture();
        let f = p.behavior.get_mut(ROOT).unwrap();
        f.steps[1].kind = StepKind::Decision;
        f.steps.extend([
            Step::new("retry", StepKind::Action, "Retry"),
            Step::new("merge", StepKind::Merge, "Merge alternatives"),
            Step::new("rejected", StepKind::Outcome, "Rejected"),
        ]);
        f.transitions[1].id = "transition.1".into();
        f.transitions[1].condition = "valid".into();
        for (id, from, to, condition) in [
            ("again", "action", "retry", "retryable"),
            ("loop", "retry", "merge", ""),
            ("back", "merge", "action", ""),
            ("reject", "action", "rejected", "invalid"),
        ] {
            f.transitions.push(Transition {
                id: id.into(),
                from: from.into(),
                to: to.into(),
                condition: condition.into(),
                outcome: None,
            });
        }
        f.data.push(DataLink {
            id: data_id.into(),
            name: "Record".into(),
            from: DataEnd {
                step: None,
                port: None,
            },
            to: DataEnd {
                step: Some("retry".into()),
                port: None,
            },
            contract: None,
            exchange: None,
        });
        // Avoid an invalid original collision: transition and data share namespace.
        if data_id == "transition.1" {
            f.transitions[1].id = "outcome.1".into();
        }
        let members = BTreeSet::from(["action".into(), "retry".into(), "merge".into()]);
        let plan = preview(
            &p,
            ROOT,
            &members,
            "Validation",
            "Validate with bounded retries",
        )
        .unwrap();
        assert_eq!(plan.region.exits.len(), 2);
        assert_expansion(&p, ROOT, &plan);
        let q = apply(&p, &plan).unwrap();
        let nested = preview(&q, &plan.component, &members, "Inner", "Review input").unwrap();
        assert_expansion(&q, &plan.component, &nested);
        let f = p.behavior.get_mut(ROOT).unwrap();
        f.transitions.push(Transition {
            id: "side".into(),
            from: "entry".into(),
            to: "retry".into(),
            condition: String::new(),
            outcome: None,
        });
        assert!(preview(&p, ROOT, &members, "Bad", "Multiple entries").is_err());
        assert!(
            preview(
                &q,
                ROOT,
                &BTreeSet::from([plan.call]),
                "Bad",
                "Existing call"
            )
            .is_err()
        );
    }
}
#[test]
fn review_is_invalidated_by_meaning_and_incident_data_edits() {
    let mut p = fixture();
    p.behavior.get_mut(ROOT).unwrap().steps[1].information_reviewed = true;
    let mut step = p.behavior[ROOT].steps[1].clone();
    step.name = "Other work".into();
    let q = save_step(&p, ROOT, step).unwrap();
    assert!(!q.behavior[ROOT].steps[1].information_reviewed);
    let (q, _) = information_candidate(
        &p,
        ROOT,
        DataLink {
            id: "need".into(),
            name: "Input".into(),
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
        },
        None,
    )
    .unwrap();
    assert!(!q.behavior[ROOT].steps[1].information_reviewed);
}
#[test]
fn exact_refinement_preserves_unrelated_same_named_channels_and_cancel() {
    let mut p = fixture();
    let link = DataLink {
        id: "input".into(),
        name: "Input".into(),
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
    p.behavior.get_mut(ROOT).unwrap().data.push(link.clone());
    let plan = preview(&p, ROOT, &selected(), "Worker", "One duty").unwrap();
    let mut p = apply(&p, &plan).unwrap();
    let mut unrelated = link;
    unrelated.id = "unrelated".into();
    unrelated.to.step = Some("done".into());
    p.behavior
        .get_mut(ROOT)
        .unwrap()
        .data
        .push(unrelated.clone());
    let store = Store::new(p.clone()).unwrap();
    let c = Contract::draft("Record".into());
    let impact = edit::binding_impact(
        &p,
        &[plan.requirements[0].id.clone()],
        &[],
        Some(&c.reference()),
        None,
    );
    assert_eq!(impact.data.len(), 2);
    let q = edit::refine_port(&p, &plan.requirements[0].id, Some(c.reference()), Some(c)).unwrap();
    assert_eq!(
        q.behavior[ROOT].data.iter().find(|d| d.id == "unrelated"),
        Some(&unrelated)
    );
    assert_eq!(store.project(), &p);
    assert_eq!(store.generation, 0);
    assert!(store.undo_label().is_none());
}
