//! Small renderer-independent flow measurements used by defaults and extraction.
use super::{Flow, StepKind};
use crate::model::Position;
use std::collections::BTreeMap;

/// Occupied card bounds in project coordinates, shared with the native canvas.
pub fn footprint(kind: StepKind) -> (f64, f64) {
    (
        260.0,
        if kind == StepKind::Decision {
            150.0
        } else {
            112.0
        },
    )
}

pub fn positions(flow: &Flow) -> BTreeMap<String, Position> {
    flow.steps
        .iter()
        .enumerate()
        .map(|(i, s)| {
            (
                s.id.clone(),
                Position {
                    x: 80.0 + (i % 3) as f64 * 340.0,
                    y: 70.0 + (i / 3) as f64 * 230.0,
                },
            )
        })
        .collect()
}

/// Resolve sparse saved coordinates before an operation changes step membership.
pub fn effective_positions(
    flow: &Flow,
    saved: Option<&BTreeMap<String, Position>>,
) -> BTreeMap<String, Position> {
    let mut result = positions(flow);
    if let Some(saved) = saved {
        for (id, pos) in &mut result {
            if let Some(at) = saved.get(id) {
                *pos = *at;
            }
        }
    }
    result
}
