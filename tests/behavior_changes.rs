//! Semantic review of validated handoff candidates, independent of UI rendering.
use system_designer::{behavior::*, model::*};

fn fixture() -> Project {
    parse(include_str!(
        "fixtures/control-flow-review/before-extraction.project.json"
    ))
    .expect("fixture")
}
fn returns() -> Project {
    parse(include_str!(
        "fixtures/control-flow-review/coincident-returns.project.json"
    ))
    .expect("returns")
}
fn diff(p: &Project, q: &Project) -> BehaviorChanges {
    validate(p).expect("valid before");
    validate(q).expect("valid candidate, possibly incomplete");
    BehaviorChanges::between(p, q)
}
#[test]
fn supplied_removed_return_reports_exact_transition_and_new_draft_issues() {
    let p = parse(include_str!(
        "fixtures/control-flow-review/after-extraction.project.json"
    ))
    .expect("fixture");
    let packet = serde_json::from_str(include_str!(
        "fixtures/control-flow-review/returned-delete-transition.behavior.json"
    ))
    .expect("packet");
    let q = replace(&p, &packet, ROOT).expect("valid incomplete replacement");
    let d = diff(&p, &q);
    assert_eq!((d.added, d.removed, d.edited), (0, 1, 0));
    assert!(d.details[0].contains("Removed transition t3: call.1 → done"));
    assert!(d.details[0].contains("outcome.1"));
    assert!(d.new_issues.iter().any(|s| s.contains("no next step")));
    assert!(d.new_issues.iter().any(|s| s.contains("unhandled outcome")));
    assert!(d.clears.is_empty());
}
#[test]
fn unchanged_and_reordered_packets_are_quiet_guard_only_edits_are_named() {
    let p = fixture();
    let q = replace(&p, &export(&p, ROOT).expect("export"), ROOT).expect("unchanged");
    assert_eq!(diff(&p, &q), BehaviorChanges::default());
    let mut q = q;
    let f = q.behavior.get_mut(ROOT).expect("flow");
    f.steps.reverse();
    f.transitions.reverse();
    assert_eq!(diff(&p, &q), BehaviorChanges::default());
    let t = q
        .behavior
        .get_mut(ROOT)
        .expect("flow")
        .transitions
        .iter_mut()
        .find(|t| t.id == "t1")
        .expect("transition");
    t.condition = "valid record".into();
    let d = diff(&p, &q);
    assert_eq!((d.added, d.removed, d.edited), (0, 0, 1));
    assert!(d.details[0].contains("transition t1: condition: \"\" → \"valid record\""));
    assert!(d.new_issues.is_empty());
}
#[test]
fn call_target_and_outcome_changes_without_renames_are_explicit() {
    let mut p = returns();
    let mut second = p.node("worker").expect("node").1.clone();
    second.id = "other.worker".into();
    p.systems[0].nodes.push(second);
    p.behavior
        .insert("other.worker".into(), p.behavior["worker"].clone());
    let mut q = p.clone();
    q.behavior
        .get_mut(ROOT)
        .expect("flow")
        .steps
        .iter_mut()
        .find(|s| s.id == "call")
        .expect("call")
        .target = Some("other.worker".into());
    let d = diff(&p, &q);
    assert_eq!(d.edited, 1);
    assert!(d.details[0].contains("step call: target:"));
    assert!(d.details[0].contains("worker") && d.details[0].contains("other.worker"));
    let mut q = p.clone();
    q.behavior
        .get_mut(ROOT)
        .expect("flow")
        .transitions
        .iter_mut()
        .find(|t| t.id == "accepted")
        .expect("return")
        .outcome = Some("failure".into());
    let d = diff(&p, &q);
    assert!(d.details[0].contains("transition accepted: outcome:"));
    assert!(d.new_issues.iter().any(|s| s.contains("unhandled outcome")));
}
#[test]
fn purpose_kind_review_primitive_and_endpoint_edits_have_actual_values() {
    let p = fixture();
    let mut q = p.clone();
    let f = q.behavior.get_mut(ROOT).expect("flow");
    let a = f.steps.iter_mut().find(|s| s.id == "a").expect("action");
    a.purpose = "Validate every field".into();
    a.kind = StepKind::Decision;
    a.information_reviewed = true;
    f.primitive = "A single responsibility".into();
    f.transitions[0].to = "b".into();
    f.transitions[1].from = "b".into();
    let d = diff(&p, &q);
    let details = d.details.join("\n");
    for expected in [
        "purpose:",
        "Validate every field",
        "kind: Action → Decision",
        "information reviewed: false → true",
        "primitive criteria:",
        "from:",
        "to:",
    ] {
        assert!(details.contains(expected), "missing {expected}: {details}");
    }
}
#[test]
fn removing_an_action_incident_transitions_and_data_lists_every_record() {
    let mut p = fixture();
    p.behavior.get_mut(ROOT).expect("flow").data.push(DataLink {
        id: "record".into(),
        name: "Record".into(),
        from: DataEnd {
            step: None,
            port: None,
        },
        to: DataEnd {
            step: Some("a".into()),
            port: None,
        },
        contract: None,
        exchange: None,
    });
    let q = delete_step(&p, ROOT, "a").expect("remove action and incident objects");
    let d = diff(&p, &q);
    assert_eq!((d.added, d.removed, d.edited), (0, 4, 0));
    let text = d.details.join("\n");
    for label in [
        "Removed step a",
        "Removed transition t0",
        "Removed transition t1",
        "Removed information requirement record",
    ] {
        assert!(text.contains(label));
    }
}
#[test]
fn information_diff_names_exact_ports_contract_versions_and_wire_associations() {
    let mut p = parse(include_str!("fixtures/project.json")).expect("interfaces");
    let mut f = Flow::default();
    for target in ["A", "B"] {
        let mut call = Step::new(format!("call.{target}"), StepKind::Call, target);
        call.target = Some(target.into());
        f.steps.push(call);
    }
    let mut link = DataLink {
        id: "record".into(),
        name: "Record".into(),
        from: DataEnd {
            step: Some("call.B".into()),
            port: None,
        },
        to: DataEnd {
            step: Some("call.A".into()),
            port: None,
        },
        contract: None,
        exchange: None,
    };
    f.data.push(link.clone());
    p = set(&p, ROOT, f).expect("declarations");
    link.name = "Bound record".into();
    link.from.port = Some("B.out".into());
    link.to.port = Some("A.in".into());
    link.contract = Some(ContractRef {
        id: "T".into(),
        version: 1,
    });
    link.exchange = Some("incoming".into());
    let mut q = p.clone();
    q.behavior.get_mut(ROOT).expect("flow").data[0] = link;
    let d = diff(&p, &q);
    assert_eq!(d.edited, 1);
    let text = d.details.join("\n");
    for label in [
        "record",
        "name:",
        "producer / port:",
        "B.out",
        "consumer / port:",
        "A.in",
        "contract:",
        "T@1",
        "interface wire:",
        "incoming",
    ] {
        assert!(text.contains(label), "missing {label}: {text}");
    }
    let reversed = diff(&q, &p);
    assert!(reversed.details[0].contains("incoming"));
    let added = diff(&set(&p, ROOT, Flow::default()).expect("empty"), &p);
    assert!(
        added
            .details
            .iter()
            .any(|s| s.contains("Added information requirement record"))
    );
    // A literal identity that resembles a display placeholder is still distinct
    // from the actual scope boundary. Compare typed endpoints, not display text.
    let mut p = fixture();
    let f = p.behavior.get_mut(ROOT).expect("flow");
    f.steps.push(Step::new(
        "scope boundary",
        StepKind::Action,
        "Explicit producer",
    ));
    f.data.push(DataLink {
        id: "record".into(),
        name: "Record".into(),
        from: DataEnd {
            step: None,
            port: None,
        },
        to: DataEnd {
            step: Some("a".into()),
            port: None,
        },
        contract: None,
        exchange: None,
    });
    let mut q = p.clone();
    q.behavior.get_mut(ROOT).expect("flow").data[0].from.step = Some("scope boundary".into());
    let details = diff(&p, &q).details.join("\n");
    assert!(
        details.contains("producer / port: [scope boundary;")
            && details.contains("[step \"scope boundary\";"),
        "{details}"
    );
}
#[test]
fn empty_existing_flow_and_null_clear_require_review_but_empty_creation_does_not() {
    let p = fixture();
    let mut packet = export(&p, ROOT).expect("export");
    packet["flow"] = serde_json::to_value(Flow::default()).expect("empty");
    let empty = replace(&p, &packet, ROOT).expect("empty draft");
    let d = diff(&p, &empty);
    assert_eq!(d.clears.len(), 1);
    assert!(d.clears[0].contains("empty draft flow object"));
    assert!(d.removed >= 9);
    packet["flow"] = serde_json::Value::Null;
    let absent = replace(&p, &packet, ROOT).expect("null clear");
    assert!(diff(&p, &absent).clears[0].contains("explicit null"));
    assert!(diff(&absent, &empty).clears.is_empty());
    assert!(diff(&empty, &empty).clears.is_empty());
}
#[test]
fn renaming_existing_draft_issues_or_reducing_review_count_does_not_introduce_issues() {
    let mut p = fixture();
    p.behavior
        .get_mut(ROOT)
        .expect("flow")
        .transitions
        .retain(|t| t.id != "t3");
    let mut q = p.clone();
    let f = q.behavior.get_mut(ROOT).expect("flow");
    f.steps.iter_mut().find(|s| s.id == "c").expect("step").name = "Renamed incomplete work".into();
    f.steps
        .iter_mut()
        .find(|s| s.id == "a")
        .expect("step")
        .information_reviewed = true;
    assert!(diff(&p, &q).new_issues.is_empty());
    let mut r = q.clone();
    r.behavior
        .get_mut(ROOT)
        .expect("flow")
        .steps
        .iter_mut()
        .find(|s| s.id == "a")
        .expect("step")
        .information_reviewed = false;
    assert!(
        diff(&q, &r)
            .new_issues
            .iter()
            .any(|s| s.contains("information-use review"))
    );
}
