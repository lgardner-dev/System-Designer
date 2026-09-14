use system_designer::{behavior, model::*};

#[test]
fn signed_layout_is_explicitly_versioned() {
    let mut p = parse(include_str!("fixtures/project.json")).expect("legacy fixture");
    let sid = p.root.clone();
    let id = p.system(&sid).expect("root").nodes[0].id.clone();
    p.layout.entry(sid).or_default().insert(id, Position { x: -120.5, y: -90.25 });
    assert!(validate(&p).is_err());
    p.version = 2;
    assert!(validate(&p).is_err());
    p.version = 3;
    validate(&p).expect("v3 supports signed interface positions without creating behavior");
    assert!(p.behavior.is_empty());
    assert_eq!(parse(&serde_json::to_string(&p).expect("serialize")).expect("reopen"), p);
    p.version = 4;
    assert!(validate(&p).is_err());
}

#[test]
fn signed_flow_and_behavior_edits_preserve_version() {
    let mut p = behavior::set(&Project::blank(), behavior::ROOT, behavior::Flow::starter()).expect("flow");
    p.version = 3;
    let id = p.behavior[behavior::ROOT].steps[1].id.clone();
    p.flow_layout.entry(behavior::ROOT.into()).or_default().insert(id, Position { x: -200.0, y: -100.0 });
    validate(&p).expect("signed flow");
    let f = p.behavior[behavior::ROOT].clone();
    let q = behavior::set(&p, behavior::ROOT, f).expect("edit v3 flow");
    assert_eq!(q.version, 3);
    assert_eq!(q.flow_layout, p.flow_layout);
}
