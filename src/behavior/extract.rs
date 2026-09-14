//! Bounded single-entry region extraction. Inspired by SESE analysis, not an RPST implementation.
use super::*;
use crate::model::{Direction, Kind, ModelError, Node, Port, Position, Project, Result, System};
use std::collections::BTreeSet;
#[derive(Clone, Debug)]
pub struct Region {
    pub members: BTreeSet<String>,
    pub entry: String,
    pub exits: Vec<Transition>,
}
#[derive(Clone)]
pub struct Extraction {
    pub component: String,
    pub call: String,
    pub owner: String,
    pub region: Region,
    pub requirements: Vec<Port>,
    pub unreviewed: usize,
    before: String,
    after: Project,
}
fn stamp(p: &Project) -> String {
    crate::exchange::digest(&serde_json::to_vec(p).expect("project serializes"))
}
fn inspect(f: &Flow, members: &BTreeSet<String>) -> Result<Region> {
    if members.is_empty() {
        return Err(ModelError::one(
            "Select at least one local action or decision.",
        ));
    }
    if members.len() > 512 {
        return Err(ModelError::one(
            "Extract at most 512 steps in one operation; split this review into smaller regions.",
        ));
    }
    for id in members {
        let s = f
            .step(id)
            .ok_or_else(|| ModelError::one("Selected step no longer exists"))?;
        if !matches!(
            s.kind,
            StepKind::Action | StepKind::Decision | StepKind::Merge
        ) {
            return Err(ModelError::one(
                "Select local actions, decisions and merges only. Entry/outcome markers stay at the boundary. Existing component calls are not reparented automatically; refine inside them instead.",
            ));
        }
    }
    let incoming: BTreeSet<_> = f
        .transitions
        .iter()
        .filter(|t| !members.contains(&t.from) && members.contains(&t.to))
        .map(|t| t.to.clone())
        .collect();
    if incoming.len() != 1 {
        return Err(ModelError::one(
            "This region needs exactly one entry step reached from outside. Include the full branch or choose a smaller region.",
        ));
    }
    let entry = incoming.into_iter().next().expect("one entry");
    let mut seen = BTreeSet::new();
    let mut todo = vec![entry.clone()];
    while let Some(id) = todo.pop() {
        if seen.insert(id.clone()) {
            for t in &f.transitions {
                if t.from == id && members.contains(&t.to) {
                    todo.push(t.to.clone());
                }
            }
        }
    }
    if seen != *members {
        return Err(ModelError::one(
            "The selected region contains steps unreachable from its entry.",
        ));
    }
    let mut exits: Vec<_> = f
        .transitions
        .iter()
        .filter(|t| members.contains(&t.from) && !members.contains(&t.to))
        .cloned()
        .collect();
    exits.sort_by(|a, b| a.id.cmp(&b.id));
    if exits.is_empty() {
        return Err(ModelError::one(
            "This region has no declared return to the parent. Add an outcome path first.",
        ));
    }
    for id in members {
        let s = f.step(id).expect("checked step");
        let next: Vec<_> = f.transitions.iter().filter(|t| &t.from == id).collect();
        if next.is_empty() {
            return Err(ModelError::one(
                "A selected step has an unconnected exit. Connect it before extraction.",
            ));
        }
        if s.kind == StepKind::Decision
            && (next.len() < 2 || next.iter().any(|t| t.condition.trim().is_empty()))
        {
            return Err(ModelError::one(
                "Label every decision alternative before extracting it.",
            ));
        }
        if s.kind != StepKind::Decision && next.len() != 1 {
            return Err(ModelError::one(
                "Multiple action successors are ambiguous. Use an explicit decision; parallel execution is not inferred.",
            ));
        }
    }
    for d in &f.data {
        if d.exchange.is_some()
            && (d.from.step.as_ref().is_some_and(|id| members.contains(id))
                || d.to.step.as_ref().is_some_and(|id| members.contains(id)))
        {
            return Err(ModelError::one(
                "This region has an existing interface-edge association. Reconcile that binding explicitly before changing its scope; extraction will not silently reroute existing wires.",
            ));
        }
    }
    Ok(Region {
        members: members.clone(),
        entry,
        exits,
    })
}
/// Deterministic suggestions, sorted by size then stable IDs; explicit invocation, never on every frame.
/// Enumerates bounded graph regions by walking from each start up to each possible stop.
/// It neither finds a canonical RPST nor judges semantic single responsibility.
pub fn candidates(f: &Flow) -> Result<Vec<Region>> {
    if f.steps.len() > 80 {
        return Err(ModelError::one(
            "Automatic suggestions are limited to 80 local steps. Select a region manually, or refine the scope first.",
        ));
    }
    let mut unique = BTreeSet::new();
    let mut out = vec![];
    for a in &f.steps {
        for b in &f.steps {
            if a.id == b.id {
                continue;
            }
            let mut selected = BTreeSet::new();
            let mut todo = vec![a.id.clone()];
            while let Some(id) = todo.pop() {
                if id == b.id {
                    continue;
                }
                let Some(s) = f.step(&id) else {
                    continue;
                };
                if !matches!(
                    s.kind,
                    StepKind::Action | StepKind::Decision | StepKind::Merge
                ) {
                    continue;
                }
                if selected.insert(id.clone()) {
                    for t in f.transitions.iter().filter(|t| t.from == id) {
                        todo.push(t.to.clone());
                    }
                }
            }
            if selected.len() >= 2
                && unique.insert(selected.clone())
                && let Ok(r) = inspect(f, &selected)
            {
                out.push(r);
            }
        }
    }
    out.sort_by(|a, b| {
        a.members
            .len()
            .cmp(&b.members.len())
            .then(a.members.cmp(&b.members))
    });
    out.truncate(24);
    Ok(out)
}
pub fn preview(
    p: &Project,
    owner: &str,
    members: &BTreeSet<String>,
    name: &str,
    responsibility: &str,
) -> Result<Extraction> {
    crate::model::validate(p)?;
    if name.trim().is_empty() || responsibility.trim().is_empty() {
        return Err(ModelError::one(
            "Name this responsibility and explain its single purpose.",
        ));
    }
    let f = p
        .behavior
        .get(owner)
        .ok_or_else(|| ModelError::one("Start the scoped flow first"))?;
    let region = inspect(f, members)?;
    let before = stamp(p);
    let mut q = p.clone();
    if q.version == 1 {
        q.version = 2;
    }
    let sid = if let Some(s) = system(&q, owner) {
        s.id.clone()
    } else {
        let id = q.fresh("system");
        q.node_mut(owner)
            .ok_or_else(|| ModelError::one("Missing owner"))?
            .child = Some(id.clone());
        q.systems.push(System {
            id: id.clone(),
            nodes: vec![],
            edges: vec![],
        });
        id
    };
    let component = q.fresh("component");
    q.system_mut(&sid)
        .expect("existing scope")
        .nodes
        .push(Node {
            id: component.clone(),
            name: name.trim().into(),
            purpose: responsibility.trim().into(),
            kind: Kind::Component,
            ports: vec![],
            child: None,
        });
    let mut parent = f.clone();
    let call = parent.fresh("call");
    let mut child = Flow {
        steps: f
            .steps
            .iter()
            .filter(|s| members.contains(&s.id))
            .cloned()
            .collect(),
        transitions: f
            .transitions
            .iter()
            .filter(|t| members.contains(&t.from) && members.contains(&t.to))
            .cloned()
            .collect(),
        ..Default::default()
    };
    // Reserve the entire source namespace before allocating synthetic identities.
    // Crossing transitions and data links enter the child later in construction.
    let mut reserved: BTreeSet<String> = f
        .steps
        .iter()
        .map(|s| s.id.clone())
        .chain(f.transitions.iter().map(|t| t.id.clone()))
        .chain(f.data.iter().map(|d| d.id.clone()))
        .collect();
    let mut fresh = |prefix: &str| {
        let id = (1u64..)
            .map(|i| format!("{prefix}.{i}"))
            .find(|id| !reserved.contains(id))
            .expect("finite namespace");
        reserved.insert(id.clone());
        id
    };
    let entry = fresh("entry");
    child
        .steps
        .push(Step::new(&entry, StepKind::Entry, "Begin"));
    child.transitions.push(Transition {
        id: fresh("transition"),
        from: entry,
        to: region.entry.clone(),
        condition: String::new(),
        outcome: None,
    });
    parent.steps.retain(|s| !members.contains(&s.id));
    let mut occurrence = Step::new(&call, StepKind::Call, name.trim());
    occurrence.target = Some(component.clone());
    occurrence.purpose = responsibility.trim().into();
    parent.steps.push(occurrence);
    parent
        .transitions
        .retain(|t| !(members.contains(&t.from) && members.contains(&t.to)));
    for t in &mut parent.transitions {
        if members.contains(&t.to) {
            t.to = call.clone();
        }
    }
    for exit in &region.exits {
        let out = fresh("outcome");
        let label = if exit.condition.trim().is_empty() {
            f.step(&exit.to)
                .map(|s| s.name.as_str())
                .unwrap_or("Return")
        } else {
            exit.condition.as_str()
        };
        child.steps.push(Step::new(&out, StepKind::Outcome, label));
        let mut inner = exit.clone();
        inner.to = out.clone();
        child.transitions.push(inner);
        let t = parent
            .transitions
            .iter_mut()
            .find(|t| t.id == exit.id)
            .expect("retained crossing");
        t.from = call.clone();
        t.outcome = Some(out);
    }
    parent.data.clear();
    let mut requirements = vec![];
    for d in &f.data {
        let from = d.from.step.as_ref().is_some_and(|id| members.contains(id));
        let to = d.to.step.as_ref().is_some_and(|id| members.contains(id));
        match (from, to) {
            (false, false) => parent.data.push(d.clone()),
            (true, true) => child.data.push(d.clone()),
            _ => {
                let pid = q.fresh("port");
                let port = Port {
                    id: pid.clone(),
                    name: d.name.clone(),
                    direction: if to { Direction::In } else { Direction::Out },
                    contract: d.contract.clone(),
                };
                q.node_mut(&component)
                    .expect("created component")
                    .ports
                    .push(port.clone());
                requirements.push(port);
                let mut outer = d.clone();
                let mut inner = d.clone();
                if to {
                    outer.to = DataEnd {
                        step: Some(call.clone()),
                        port: Some(pid.clone()),
                    };
                    inner.from = DataEnd {
                        step: None,
                        port: Some(pid),
                    };
                } else {
                    outer.from = DataEnd {
                        step: Some(call.clone()),
                        port: Some(pid.clone()),
                    };
                    inner.to = DataEnd {
                        step: None,
                        port: Some(pid),
                    };
                }
                parent.data.push(outer);
                child.data.push(inner);
            }
        }
    }
    child.extraction = Some(Provenance {
        rule_version: 1,
        parent: owner.into(),
        source_steps: members.iter().cloned().collect(),
        responsibility: responsibility.trim().into(),
        base: before.clone(),
    });
    let unreviewed = child
        .steps
        .iter()
        .filter(|s| {
            matches!(s.kind, StepKind::Action | StepKind::Decision) && !s.information_reviewed
        })
        .count();
    // Freeze the effective original scene before removing/reordering membership.
    let original = effective_positions(f, p.flow_layout.get(owner));
    // The Call fits inside the replaced entry's footprint (including a decision).
    // Independent minimum X/Y can describe an occupied, unselected parent corner.
    let call_position = original[&region.entry];
    let x = members
        .iter()
        .filter_map(|id| original.get(id))
        .map(|v| v.x)
        .fold(f64::INFINITY, f64::min);
    let y = members
        .iter()
        .filter_map(|id| original.get(id))
        .map(|v| v.y)
        .fold(f64::INFINITY, f64::min);
    let gap = 70.0;
    let work_top = 70.0 + footprint(StepKind::Entry).1 + gap;
    let mut child_layout = std::collections::BTreeMap::new();
    for id in members {
        let pos = original[id];
        child_layout.insert(
            id.clone(),
            Position {
                x: pos.x - x + 80.0,
                y: pos.y - y + work_top,
            },
        );
    }
    let bottom = child
        .steps
        .iter()
        .filter(|s| members.contains(&s.id))
        .map(|s| child_layout[&s.id].y + footprint(s.kind).1)
        .fold(work_top, f64::max);
    let entry_x = child_layout[&region.entry].x;
    // Synthetic markers have their own rows outside all moved work. Outcome
    // identity order is deterministic and leaves room between marker footprints.
    let mut outcomes: Vec<_> = child
        .steps
        .iter()
        .filter(|s| s.kind == StepKind::Outcome)
        .collect();
    outcomes.sort_by(|a, b| a.id.cmp(&b.id));
    for (i, s) in outcomes.iter().enumerate() {
        child_layout.insert(
            s.id.clone(),
            Position {
                x: 80.0 + i as f64 * (footprint(s.kind).0 + gap),
                y: bottom + gap,
            },
        );
    }
    for s in child.steps.iter().filter(|s| s.kind == StepKind::Entry) {
        child_layout.insert(
            s.id.clone(),
            Position {
                x: entry_x,
                y: 70.0,
            },
        );
    }
    let mut parent_layout = original;
    parent_layout.retain(|id, _| !members.contains(id));
    parent_layout.insert(call.clone(), call_position);
    q.behavior.insert(owner.into(), parent);
    q.behavior.insert(component.clone(), child);
    q.flow_layout.insert(owner.into(), parent_layout);
    q.flow_layout.insert(component.clone(), child_layout);
    crate::model::validate(&q)?;
    Ok(Extraction {
        component,
        call,
        owner: owner.into(),
        region,
        requirements,
        unreviewed,
        before,
        after: q,
    })
}
pub fn apply(p: &Project, plan: &Extraction) -> Result<Project> {
    if stamp(p) != plan.before {
        return Err(ModelError::one(
            "The design changed after preview. Preview the extraction again.",
        ));
    }
    crate::model::validate(&plan.after)?;
    Ok(plan.after.clone())
}
