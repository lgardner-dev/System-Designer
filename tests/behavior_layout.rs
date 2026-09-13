//! Extraction layout regressions from the f307fe7 native review.
use std::collections::{BTreeMap, BTreeSet};
use system_designer::{behavior::*, edit::Store, model::*};

fn fixture() -> Project {
    parse(include_str!(
        "fixtures/control-flow-review/before-extraction.project.json"
    ))
    .expect("review fixture")
}
fn effective(p: &Project, owner: &str) -> BTreeMap<String, Position> {
    let mut result = positions(&p.behavior[owner]);
    result.extend(p.flow_layout.get(owner).into_iter().flat_map(|m| m.clone()));
    result
}
fn assert_no_overlaps(p: &Project, owner: &str) {
    let f = &p.behavior[owner];
    let positions = effective(p, owner);
    for (i, a) in f.steps.iter().enumerate() {
        for b in &f.steps[i + 1..] {
            let x = positions[&a.id];
            let y = positions[&b.id];
            let height = |s: &Step| {
                if s.kind == StepKind::Decision {
                    150.0
                } else {
                    112.0
                }
            };
            assert!(
                x.x + 260.0 <= y.x
                    || y.x + 260.0 <= x.x
                    || x.y + height(a) <= y.y
                    || y.y + height(b) <= x.y,
                "{owner}: {} at {x:?} overlaps {} at {y:?}",
                a.id,
                b.id
            );
        }
    }
}
fn extract(p: &Project, members: &[&str]) -> (Project, Extraction) {
    let plan = preview(
        p,
        ROOT,
        &members.iter().map(|s| (*s).into()).collect(),
        "Import record",
        "Prepare one record",
    )
    .expect("preview");
    (apply(p, &plan).expect("apply"), plan)
}
#[test]
fn f1_review_parent_keeps_nonoverlapping_positions() {
    let p = fixture();
    assert_no_overlaps(&p, ROOT);
    let (q, plan) = extract(&p, &["a", "b", "c"]);
    assert_no_overlaps(&q, ROOT);
    assert_eq!(effective(&q, ROOT)[&plan.call], effective(&p, ROOT)["a"]);
}
#[test]
fn f1_review_child_markers_do_not_overlap_moved_work() {
    let (q, plan) = extract(&fixture(), &["a", "b", "c"]);
    assert_no_overlaps(&q, &plan.component);
}

fn sequence(n: usize) -> Project {
    let mut f = Flow::default();
    f.steps.push(Step::new("entry", StepKind::Entry, "Begin"));
    for i in 0..n {
        f.steps
            .push(Step::new(format!("work.{i}"), StepKind::Action, "Work"));
    }
    f.steps
        .push(Step::new("done", StepKind::Outcome, "Complete"));
    f.transitions = f
        .steps
        .windows(2)
        .enumerate()
        .map(|(i, pair)| Transition {
            id: format!("t.{i}"),
            from: pair[0].id.clone(),
            to: pair[1].id.clone(),
            condition: String::new(),
            outcome: None,
        })
        .collect();
    set(&Project::blank(), ROOT, f).expect("sequence")
}

fn check_layout_and_history(p: &Project, members: &BTreeSet<String>) {
    assert_no_overlaps(p, ROOT);
    let plan = preview(p, ROOT, members, "Work", "Perform the selected work").expect("preview");
    let q = apply(p, &plan).expect("apply");
    let again =
        preview(p, ROOT, members, "Work", "Perform the selected work").expect("repeat preview");
    assert_eq!(q, apply(p, &again).expect("repeat apply"));
    assert_no_overlaps(&q, ROOT);
    assert_no_overlaps(&q, &plan.component);
    let before = effective(p, ROOT);
    let after = effective(&q, ROOT);
    for step in &p.behavior[ROOT].steps {
        if !members.contains(&step.id) {
            assert_eq!(
                after[&step.id], before[&step.id],
                "unselected {} moved",
                step.id
            );
        }
    }
    assert_eq!(after[&plan.call], before[&plan.region.entry]);
    let child = effective(&q, &plan.component);
    let anchor = members.first().expect("member");
    for id in members {
        assert_eq!(
            before[id].x - before[anchor].x,
            child[id].x - child[anchor].x
        );
        assert_eq!(
            before[id].y - before[anchor].y,
            child[id].y - child[anchor].y
        );
    }
    let mut store = Store::new(p.clone()).expect("store");
    store.publish("Extract", q.clone()).expect("publish");
    assert_eq!(store.generation, 1);
    store.undo();
    assert_eq!(store.project(), p);
    store.redo();
    assert_eq!(store.project(), &q);
}

#[test]
fn extraction_layout_one_three_eight_steps_default_manual_sparse_and_history() {
    for n in [1, 3, 8] {
        for layout in ["default", "manual", "sparse"] {
            let mut p = sequence(n);
            let f = &p.behavior[ROOT];
            let members = f
                .steps
                .iter()
                .filter(|s| s.kind == StepKind::Action)
                .map(|s| s.id.clone())
                .collect();
            if layout == "manual" {
                p.flow_layout.insert(
                    ROOT.into(),
                    f.steps
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            (
                                s.id.clone(),
                                Position {
                                    x: 90.0 + (i % 2) as f64 * 510.0,
                                    y: 45.0 + i as f64 * 190.0,
                                },
                            )
                        })
                        .collect(),
                );
            } else if layout == "sparse" {
                // Leave both the selected entry and retained outcome unsaved. Their
                // fallback grid slots must be resolved before extraction changes order.
                p.flow_layout.insert(
                    ROOT.into(),
                    BTreeMap::from([
                        (
                            "entry".into(),
                            Position {
                                x: 1700.0,
                                y: 420.0,
                            },
                        ),
                        (
                            format!("work.{}", n - 1),
                            Position {
                                x: 1410.0,
                                y: 770.0,
                            },
                        ),
                    ]),
                );
            }
            check_layout_and_history(&p, &members);
        }
    }
}

#[test]
fn extraction_layout_loop_and_multiple_outcomes_stay_clear() {
    let mut p = sequence(3);
    let f = p.behavior.get_mut(ROOT).expect("flow");
    f.steps[1].kind = StepKind::Decision;
    f.transitions[1].condition = "accepted".into();
    f.transitions.push(Transition {
        id: "reject".into(),
        from: "work.0".into(),
        to: "done".into(),
        condition: "rejected".into(),
        outcome: None,
    });
    f.steps[3].kind = StepKind::Decision;
    f.transitions[3].condition = "complete".into();
    f.transitions.push(Transition {
        id: "retry".into(),
        from: "work.2".into(),
        to: "work.0".into(),
        condition: "retry".into(),
        outcome: None,
    });
    check_layout_and_history(
        &p,
        &BTreeSet::from(["work.0".into(), "work.1".into(), "work.2".into()]),
    );
}
