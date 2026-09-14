//! Deliberate layout changes share one version-aware candidate boundary.
use super::candidate;
use crate::model::*;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LayoutScope {
    Interfaces(String),
    Flow(String),
}

pub fn move_elements(
    p: &Project,
    scope: &LayoutScope,
    positions: BTreeMap<String, Position>,
) -> Result<Project> {
    candidate(p, |q| {
        if positions.values().any(|pos| pos.x < 0.0 || pos.y < 0.0) && matches!(q.version, 1 | 2) {
            q.version = 3;
        }
        let (layout, owner) = match scope {
            LayoutScope::Interfaces(sid) => (&mut q.layout, sid),
            LayoutScope::Flow(owner) => (&mut q.flow_layout, owner),
        };
        layout.entry(owner.clone()).or_default().extend(positions);
        Ok(())
    })
}
