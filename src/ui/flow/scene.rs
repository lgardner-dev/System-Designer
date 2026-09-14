//! Control semantics project to derived ingress/egress handles, never typed ports.
use crate::{
    behavior::{Flow, StepKind, Transition},
    ui::diagram::{
        anchors::{Outline, ResolvedAnchor, Side, choose_side},
        routes::{self, Lane, Path},
    },
};
use eframe::egui;
use egui::{Rect, Vec2};
use std::collections::BTreeMap;

pub(super) type Anchors = BTreeMap<(String, bool), ResolvedAnchor>;
pub(super) fn outline(kind: StepKind) -> Outline {
    match kind {
        StepKind::Decision => Outline::Diamond,
        StepKind::Entry | StepKind::Outcome => Outline::Ellipse,
        _ => Outline::Rectangle,
    }
}
pub(super) fn anchors(flow: &Flow, rects: &BTreeMap<String, Rect>) -> Anchors {
    let mut result = Anchors::new();
    for step in &flow.steps {
        let rect = rects[&step.id];
        let side = |output| {
            let peers: Vec<_> = flow
                .transitions
                .iter()
                .filter_map(|t| {
                    let remote = if output && t.from == step.id {
                        &t.to
                    } else if !output && t.to == step.id {
                        &t.from
                    } else {
                        return None;
                    };
                    (remote != &step.id).then(|| rects[remote].center())
                })
                .collect();
            let default = if output {
                Side::Right
            } else if flow
                .transitions
                .iter()
                .any(|t| t.from == step.id && t.to == step.id)
            {
                Side::Top
            } else {
                Side::Left
            };
            choose_side(rect, &peers, default, false)
        };
        let input = side(false);
        let output = side(true);
        for (is_output, side) in [(false, input), (true, output)] {
            if (is_output && step.kind == StepKind::Outcome)
                || (!is_output && step.kind == StepKind::Entry)
            {
                continue;
            }
            let fraction = if input == output {
                if is_output { 0.67 } else { 0.33 }
            } else {
                0.5
            };
            result.insert(
                (step.id.clone(), is_output),
                outline(step.kind).anchor(rect, side, fraction),
            );
        }
    }
    result
}
pub(super) fn routes<'a>(
    flow: &'a Flow,
    rects: &BTreeMap<String, Rect>,
    anchors: &Anchors,
) -> Vec<(&'a Transition, Path)> {
    let mut groups: BTreeMap<(&str, &str), Vec<&Transition>> = BTreeMap::new();
    for t in &flow.transitions {
        let pair = if t.from <= t.to {
            (t.from.as_str(), t.to.as_str())
        } else {
            (t.to.as_str(), t.from.as_str())
        };
        groups.entry(pair).or_default().push(t);
    }
    let mut result = vec![];
    for ((first, last), mut group) in groups {
        group.sort_by(|a, b| a.id.cmp(&b.id));
        let count = group.len();
        for (index, t) in group.into_iter().enumerate() {
            let (Some(a), Some(b)) = (
                anchors.get(&(t.from.clone(), true)),
                anchors.get(&(t.to.clone(), false)),
            ) else {
                continue;
            };
            let axis = rects[last].center() - rects[first].center();
            result.push((
                t,
                routes::route(
                    *a,
                    *b,
                    Lane {
                        index,
                        count,
                        axis: if axis.length_sq() < 0.001 {
                            Vec2::X
                        } else {
                            axis
                        },
                        loop_rect: (first == last).then_some(rects[first]),
                    },
                ),
            ));
        }
    }
    result
}
