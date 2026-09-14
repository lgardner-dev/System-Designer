use system_designer::{behavior, model::*};

#[test]
fn signed_layout_is_explicitly_versioned() {
    let mut p = parse(include_str!("fixtures/project.json")).expect("legacy fixture");
    let sid = p.root.clone();
    let id = p.system(&sid).expect("root").nodes[0].id.clone();
    p.layout.entry(sid).or_default().insert(
        id,
        Position {
            x: -120.5,
            y: -90.25,
        },
    );
    assert!(validate(&p).is_err());
    p.version = 2;
    assert!(validate(&p).is_err());
    p.version = 3;
    validate(&p).expect("v3 supports signed interface positions without creating behavior");
    assert!(p.behavior.is_empty());
    assert_eq!(
        parse(&serde_json::to_string(&p).expect("serialize")).expect("reopen"),
        p
    );
    p.version = 4;
    assert!(validate(&p).is_err());
}

#[test]
fn signed_flow_and_behavior_edits_preserve_version() {
    let mut p =
        behavior::set(&Project::blank(), behavior::ROOT, behavior::Flow::starter()).expect("flow");
    p.version = 3;
    let id = p.behavior[behavior::ROOT].steps[1].id.clone();
    p.flow_layout
        .entry(behavior::ROOT.into())
        .or_default()
        .insert(
            id,
            Position {
                x: -200.0,
                y: -100.0,
            },
        );
    validate(&p).expect("signed flow");
    let f = p.behavior[behavior::ROOT].clone();
    let q = behavior::set(&p, behavior::ROOT, f).expect("edit v3 flow");
    assert_eq!(q.version, 3);
    assert_eq!(q.flow_layout, p.flow_layout);
}

#[test]
fn promotion_is_atomic_undoable_and_rejects_nonfinite_out_of_range_and_reserved_ids() {
    use std::collections::BTreeMap;
    use system_designer::edit::{self, LayoutScope, Store};
    let p = parse(include_str!("fixtures/project.json")).expect("fixture");
    let id = p.system(&p.root).expect("root").nodes[0].id.clone();
    let scope = LayoutScope::Interfaces(p.root.clone());
    let at = Position {
        x: -100.25,
        y: -200.5,
    };
    let q =
        edit::move_elements(&p, &scope, BTreeMap::from([(id.clone(), at)])).expect("signed move");
    let mut store = Store::new(p.clone()).expect("store");
    store.publish("Move", q.clone()).expect("publish once");
    assert_eq!(store.generation, 1);
    assert_eq!(q.version, 3);
    assert!(q.behavior.is_empty());
    assert!(store.undo());
    assert_eq!(store.project(), &p);
    assert!(store.redo());
    assert_eq!(store.project(), &q);
    for x in [
        f64::NAN,
        f64::INFINITY,
        -f64::INFINITY,
        MAX_LAYOUT_COORDINATE + 1.0,
        -MAX_LAYOUT_COORDINATE - 1.0,
    ] {
        assert!(
            edit::move_elements(
                &q,
                &scope,
                BTreeMap::from([(id.clone(), Position { x, y: 0.0 })])
            )
            .is_err()
        );
        assert_eq!(store.project(), &q);
    }
    let mut reserved = Project::blank();
    let (draft, nid) = edit::add_node(&reserved, &reserved.root).expect("component");
    reserved = draft;
    reserved.systems[0].nodes[0].id = behavior::ROOT.into();
    validate(&reserved).expect("legacy reserved identity remains legal in v1");
    assert!(
        edit::move_elements(
            &reserved,
            &LayoutScope::Interfaces(reserved.root.clone()),
            BTreeMap::from([(behavior::ROOT.into(), at)])
        )
        .is_err()
    );
    assert!(reserved.node(&nid).is_none());
    assert!(reserved.node(behavior::ROOT).is_some());
}

#[test]
fn signed_positions_survive_extraction_replacement_save_and_recovery() {
    use std::collections::{BTreeMap, BTreeSet};
    use system_designer::{edit, exchange, storage};
    let mut p = parse(include_str!(
        "fixtures/control-flow-review/before-extraction.project.json"
    ))
    .expect("review fixture");
    p.version = 3;
    p.flow_layout.insert(
        behavior::ROOT.into(),
        behavior::effective_positions(
            &p.behavior[behavior::ROOT],
            p.flow_layout.get(behavior::ROOT),
        ),
    );
    for pos in p
        .flow_layout
        .get_mut(behavior::ROOT)
        .expect("saved layout")
        .values_mut()
    {
        pos.x -= 1500.0;
        pos.y -= 1000.0;
    }
    let plan = behavior::preview(
        &p,
        behavior::ROOT,
        &BTreeSet::from(["a".into(), "b".into(), "c".into()]),
        "Worker",
        "One duty",
    )
    .expect("negative extraction");
    let mut q = behavior::apply(&p, &plan).expect("apply");
    assert_eq!(q.version, 3);
    assert_eq!(
        q.flow_layout[behavior::ROOT][&plan.call],
        p.flow_layout[behavior::ROOT]["a"]
    );
    q = edit::move_elements(
        &q,
        &edit::LayoutScope::Interfaces(q.root.clone()),
        BTreeMap::from([(
            plan.component.clone(),
            Position {
                x: -400.0,
                y: -250.0,
            },
        )]),
    )
    .expect("interface move");
    let packet = behavior::export(&q, behavior::ROOT).expect("behavior packet");
    let replaced = behavior::replace(&q, &packet, behavior::ROOT).expect("behavior replacement");
    assert_eq!(replaced, q);
    let packet =
        exchange::export(&q, &q.root, exchange::Scope::Level, None).expect("interface packet");
    let replaced = exchange::replace(&q, &packet, &q.root).expect("interface replacement");
    assert_eq!(replaced, q);
    let text = serde_json::to_string_pretty(&q).expect("serialized");
    assert_eq!(parse(&text).expect("reopened"), q);
    let directory = tempfile::tempdir().expect("scratch directory");
    let recovery = directory.path().join("recovery.json");
    storage::write_recovery(&recovery, &q, None).expect("recovery snapshot");
    assert_eq!(
        storage::read_recovery(&recovery).expect("recovery").project,
        q
    );
}
