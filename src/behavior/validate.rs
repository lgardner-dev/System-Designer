use super::*;
use crate::model::{Direction, Endpoint, ModelError, Project, Result};
use std::collections::{BTreeMap, BTreeSet};
fn require(b: bool, msg: impl Into<String>) -> Result<()> {
    if b {
        Ok(())
    } else {
        Err(ModelError::one(msg.into()))
    }
}
fn port<'a>(
    p: &'a Project,
    owner: &str,
    f: &Flow,
    end: &DataEnd,
    source: bool,
) -> Result<Option<&'a crate::model::Port>> {
    let Some(pid) = &end.port else {
        return Ok(None);
    };
    let (node, boundary) = if let Some(step) = &end.step {
        let s = f
            .step(step)
            .ok_or_else(|| ModelError::one("Missing data step"))?;
        (
            s.target
                .as_deref()
                .ok_or_else(|| ModelError::one("Only a component call can bind a public port"))?,
            false,
        )
    } else {
        (owner, true)
    };
    let n = p
        .node(node)
        .ok_or_else(|| ModelError::one("Root/local logic has no declared public component port"))?
        .1;
    let r = n
        .ports
        .iter()
        .find(|r| r.id == *pid)
        .ok_or_else(|| ModelError::one(format!("Missing data port {pid}")))?;
    let direction = if boundary {
        r.direction.opposite()
    } else {
        r.direction
    };
    require(
        direction
            == if source {
                Direction::Out
            } else {
                Direction::In
            },
        "Data binding direction is incompatible",
    )?;
    Ok(Some(r))
}
pub fn check(p: &Project) -> Result<()> {
    require(
        p.version == 2 || (p.behavior.is_empty() && p.flow_layout.is_empty()),
        "Control flow requires project version 2; use Start flow to upgrade",
    )?;
    require(
        p.version != 2 || p.node(ROOT).is_none(),
        "@root is reserved as the behavior root",
    )?;
    for (owner, f) in &p.behavior {
        require(
            exists(p, owner),
            format!("Behavior owner {owner} was removed; remove/reconcile its behavior explicitly"),
        )?;
        let mut ids = BTreeSet::new();
        for s in &f.steps {
            require(
                !s.id.trim().is_empty() && !s.name.trim().is_empty() && ids.insert(s.id.clone()),
                "Empty/duplicate flow step ID or name",
            )?;
            if s.kind == StepKind::Call {
                let target = s
                    .target
                    .as_deref()
                    .ok_or_else(|| ModelError::one("A call needs a component target"))?;
                require(
                    system(p, owner).is_some_and(|sys| sys.nodes.iter().any(|n| n.id == target)),
                    format!(
                        "Call {} must target an immediate child of {}",
                        s.name,
                        name(p, owner)
                    ),
                )?;
            } else {
                require(
                    s.target.is_none(),
                    "Only call steps may reference a component",
                )?;
            }
        }
        require(
            f.steps.iter().filter(|s| s.kind == StepKind::Entry).count() <= 1,
            "Use one entry per scoped flow",
        )?;
        for t in &f.transitions {
            require(
                !t.id.trim().is_empty() && ids.insert(t.id.clone()),
                "Duplicate/empty transition ID",
            )?;
            let a = f
                .step(&t.from)
                .ok_or_else(|| ModelError::one("Transition source was removed"))?;
            let b = f
                .step(&t.to)
                .ok_or_else(|| ModelError::one("Transition destination was removed"))?;
            require(
                a.kind != StepKind::Outcome && b.kind != StepKind::Entry,
                "Control cannot leave an outcome or enter an entry marker",
            )?;
            if let Some(outcome) = &t.outcome {
                require(
                    a.kind == StepKind::Call,
                    "Only a call return may name a child outcome",
                )?;
                require(
                    a.target
                        .as_ref()
                        .and_then(|id| p.behavior.get(id))
                        .and_then(|f| f.step(outcome))
                        .is_some_and(|s| s.kind == StepKind::Outcome),
                    "Call return references a missing child outcome",
                )?;
            }
        }
        for d in &f.data {
            require(
                !d.id.trim().is_empty() && ids.insert(d.id.clone()) && !d.name.trim().is_empty(),
                "Duplicate/empty information requirement",
            )?;
            require(
                d.from.step.is_some() || d.to.step.is_some(),
                "Information link must involve a step",
            )?;
            for e in [&d.from, &d.to] {
                if let Some(id) = &e.step {
                    require(
                        f.step(id).is_some(),
                        "Information requirement refers to a missing occurrence",
                    )?;
                }
            }
            if let Some(r) = &d.contract {
                require(
                    p.contract(r).is_some(),
                    format!("Missing information contract {r}"),
                )?;
            }
            let a = port(p, owner, f, &d.from, true)?;
            let b = port(p, owner, f, &d.to, false)?;
            for r in [a, b].into_iter().flatten() {
                require(
                    r.contract == d.contract,
                    "Information requirement and bound port must share the exact contract (or both remain unassigned)",
                )?;
            }
            if let Some(eid) = &d.exchange {
                let sys = system(p, owner)
                    .ok_or_else(|| ModelError::one("No internal interface scope for exchange"))?;
                let e = sys
                    .edges
                    .iter()
                    .find(|e| e.id == *eid)
                    .ok_or_else(|| ModelError::one("Bound interface edge was removed"))?;
                let endpoint = |end: &DataEnd| -> Option<Endpoint> {
                    let port = end.port.clone()?;
                    let node = match &end.step {
                        Some(id) => Some(f.step(id)?.target.clone()?),
                        None => None,
                    };
                    Some(Endpoint { node, port })
                };
                require(
                    endpoint(&d.from).as_ref() == Some(&e.from)
                        && endpoint(&d.to).as_ref() == Some(&e.to),
                    "Exchange endpoints must match the exact occurrence port bindings",
                )?;
            }
        }
    }
    for (owner, positions) in &p.flow_layout {
        let f = p
            .behavior
            .get(owner)
            .ok_or_else(|| ModelError::one("Flow layout has no behavior scope"))?;
        for (id, pos) in positions {
            require(
                f.step(id).is_some()
                    && pos.x.is_finite()
                    && pos.y.is_finite()
                    && pos.x >= 0.0
                    && pos.y >= 0.0,
                "Invalid flow layout step or coordinates",
            )?;
        }
    }
    Ok(())
}
fn reachable(f: &Flow, start: &str, reverse: bool) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut todo = vec![start.to_owned()];
    while let Some(id) = todo.pop() {
        if seen.insert(id.clone()) {
            for t in &f.transitions {
                if (if reverse { &t.to } else { &t.from }) == &id {
                    todo.push(if reverse {
                        t.from.clone()
                    } else {
                        t.to.clone()
                    });
                }
            }
        }
    }
    seen
}
/// Draft diagnostics are visible guidance, not structural publication errors.
pub fn issues(p: &Project, owner: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    issue_details(p, owner)
        .into_iter()
        .filter_map(|(key, message)| {
            let aggregate = matches!(
                key[0].as_str(),
                "unreachable" | "information_review" | "unassigned_contract"
            );
            (!aggregate || seen.insert(key[0].clone())).then_some(message)
        })
        .collect()
}
/// Stable category/identity keys distinguish new draft issues from renamed text.
/// Aggregate guidance retains one display message while tracking affected IDs.
pub(super) fn issue_details(p: &Project, owner: &str) -> Vec<(Vec<String>, String)> {
    let Some(f) = p.behavior.get(owner) else {
        return vec![(
            vec!["unspecified".into()],
            "Control flow is not specified. Start a flow; interfaces stay unchanged.".into(),
        )];
    };
    let mut out = vec![];
    let mut issue = |category: &str, ids: &[&str], message: String| {
        out.push((
            std::iter::once(category)
                .chain(ids.iter().copied())
                .map(str::to_owned)
                .collect(),
            message,
        ));
    };
    if let Some(entry) = f.steps.iter().find(|s| s.kind == StepKind::Entry) {
        let seen = reachable(f, &entry.id, false);
        for s in f.steps.iter().filter(|s| !seen.contains(&s.id)) {
            issue(
                "unreachable",
                &[&s.id],
                "Some steps are not reachable from the entry.".into(),
            );
        }
    } else {
        issue("missing_entry", &[], "Add an entry marker.".into());
    }
    if !f.steps.iter().any(|s| s.kind == StepKind::Outcome) {
        issue(
            "missing_outcome",
            &[],
            "Add at least one explicit outcome.".into(),
        );
    }
    for s in &f.steps {
        let edges: Vec<_> = f.transitions.iter().filter(|t| t.from == s.id).collect();
        if s.kind != StepKind::Outcome && edges.is_empty() {
            issue("no_next", &[&s.id], format!("{} has no next step.", s.name));
        }
        if s.kind == StepKind::Decision
            && (edges.len() < 2 || edges.iter().any(|t| t.condition.trim().is_empty()))
        {
            issue(
                "alternatives",
                &[&s.id],
                format!("{} needs labeled alternatives.", s.name),
            );
        }
        if !matches!(
            s.kind,
            StepKind::Decision | StepKind::Call | StepKind::Outcome
        ) && edges.len() > 1
        {
            issue(
                "ambiguous",
                &[&s.id],
                format!(
                    "{} has ambiguous multiple successors; insert a decision. This editor does not infer parallel execution.",
                    s.name
                ),
            );
        }
        if s.kind == StepKind::Call && edges.iter().any(|t| t.outcome.is_none()) {
            issue(
                "unresolved_return",
                &[&s.id],
                format!("{} has an unresolved return outcome.", s.name),
            );
        }
        if s.kind == StepKind::Call
            && let Some(child) = s.target.as_ref().and_then(|id| p.behavior.get(id))
        {
            for outcome in child.steps.iter().filter(|s| s.kind == StepKind::Outcome) {
                let handlers: Vec<_> = edges
                    .iter()
                    .filter(|t| t.outcome.as_ref() == Some(&outcome.id))
                    .collect();
                if handlers.is_empty() {
                    issue(
                        "unhandled_outcome",
                        &[
                            &s.id,
                            s.target.as_deref().expect("call target"),
                            &outcome.id,
                        ],
                        format!(
                            "{} [{}] has an unhandled outcome: {} [{}].",
                            s.name, s.id, outcome.name, outcome.id
                        ),
                    );
                }
                if handlers
                    .iter()
                    .filter(|t| t.condition.trim().is_empty())
                    .count()
                    > 1
                {
                    issue(
                        "duplicate_return",
                        &[
                            &s.id,
                            s.target.as_deref().expect("call target"),
                            &outcome.id,
                        ],
                        format!(
                            "{} [{}] has duplicate unguarded returns for {} [{}]; use one continuation and an explicit parent decision.",
                            s.name, s.id, outcome.name, outcome.id
                        ),
                    );
                }
            }
        }
    }
    let unreviewed: Vec<_> = f
        .steps
        .iter()
        .filter(|s| {
            matches!(s.kind, StepKind::Action | StepKind::Decision) && !s.information_reviewed
        })
        .collect();
    for s in &unreviewed {
        issue(
            "information_review",
            &[&s.id],
            format!(
                "{} action/decision steps still need an information-use review.",
                unreviewed.len()
            ),
        );
    }
    let n = f.data.iter().filter(|d| d.contract.is_none()).count();
    for d in f.data.iter().filter(|d| d.contract.is_none()) {
        issue(
            "unassigned_contract",
            &[&d.id],
            format!("{n} information requirements have unassigned contracts."),
        );
    }
    if f.steps
        .iter()
        .filter(|s| matches!(s.kind, StepKind::Action | StepKind::Decision))
        .count()
        > 8
    {
        issue("scope_size", &[], "More than eight local steps: consider a meaningful extraction, not arbitrary groups of eight.".into());
    }
    out
}
pub fn complexity(f: &Flow) -> Option<usize> {
    let entry = f.steps.iter().find(|s| s.kind == StepKind::Entry)?;
    if reachable(f, &entry.id, false).len() != f.steps.len() {
        return None;
    }
    let exits: Vec<_> = f
        .steps
        .iter()
        .filter(|s| s.kind == StepKind::Outcome)
        .collect();
    if exits.is_empty() {
        return None;
    }
    let returning: BTreeSet<_> = exits
        .iter()
        .flat_map(|s| reachable(f, &s.id, true))
        .collect();
    if returning.len() != f.steps.len() {
        return None;
    }
    let mut counts = BTreeMap::new();
    for t in &f.transitions {
        *counts.entry(&t.from).or_insert(0usize) += 1;
    }
    if f.steps
        .iter()
        .any(|s| s.kind != StepKind::Outcome && *counts.get(&s.id).unwrap_or(&0) == 0)
    {
        return None;
    }
    f.transitions
        .len()
        .checked_add(exits.len() + 1)?
        .checked_sub(f.steps.len())
}
