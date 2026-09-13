//! Identity-based review of validated behavior candidates. No packet/hash changes.
use super::{DataEnd, DataLink, Flow, Step, Transition, validate::issue_details};
use crate::model::Project;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BehaviorChanges {
    pub added: usize,
    pub removed: usize,
    pub edited: usize,
    pub details: Vec<String>,
    pub new_issues: Vec<String>,
    /// Destructive scope/content removals that need deliberate UI confirmation.
    pub clears: Vec<String>,
}

impl BehaviorChanges {
    pub fn summary(&self) -> String {
        if self.details.is_empty() {
            return "No behavior changes.".into();
        }
        format!(
            "Control Flow: {} added · {} removed · {} edited. {} newly introduced draft issues.",
            self.added,
            self.removed,
            self.edited,
            self.new_issues.len()
        )
    }
    fn add(&mut self, owner: &str, verb: &str, detail: String) {
        match verb {
            "Added" => self.added += 1,
            "Removed" => self.removed += 1,
            _ => self.edited += 1,
        }
        self.details.push(format!("{owner} · {verb} {detail}"));
    }
    /// Counts describe changed records (or scope metadata), not changed fields.
    /// Sorting by identity makes presentation independent of JSON array order.
    pub fn between(before: &Project, after: &Project) -> Self {
        let mut result = Self::default();
        let owners: BTreeSet<_> = before
            .behavior
            .keys()
            .chain(after.behavior.keys())
            .collect();
        let empty = Flow::default();
        for owner in owners {
            let old = before.behavior.get(owner);
            let new = after.behavior.get(owner);
            match (old, new) {
                (None, Some(_)) => result.add(owner, "Added", "flow scope".into()),
                (Some(_), None) => {
                    result.add(
                        owner,
                        "Removed",
                        "flow scope and its saved flow layout (explicit null)".into(),
                    );
                    result.clears.push(format!("{owner}: remove the entire control flow and its saved flow layout (explicit null)."));
                }
                (Some(a), Some(b)) if empties(a, b) => {
                    result.clears.push(format!("{owner}: remove all existing flow contents; keep an empty draft flow object and any unchanged scope metadata. Removed steps also lose their saved positions."));
                }
                _ => {}
            }
            let (a, b) = (old.unwrap_or(&empty), new.unwrap_or(&empty));
            records(
                &mut result,
                owner,
                "step",
                (&a.steps, &b.steps),
                |s| &s.id,
                step,
                step_edits,
            );
            records(
                &mut result,
                owner,
                "transition",
                (&a.transitions, &b.transitions),
                |t| &t.id,
                transition,
                transition_edits,
            );
            records(
                &mut result,
                owner,
                "information requirement",
                (&a.data, &b.data),
                |d| &d.id,
                data,
                data_edits,
            );
            if a.primitive != b.primitive {
                result.add(
                    owner,
                    "Edited",
                    format!("primitive criteria: {:?} → {:?}", a.primitive, b.primitive),
                );
            }
            if a.extraction != b.extraction {
                result.add(
                    owner,
                    "Edited",
                    format!(
                        "extraction provenance: {:?} → {:?}",
                        a.extraction, b.extraction
                    ),
                );
            }
            let previous: BTreeSet<_> = issue_details(before, owner)
                .into_iter()
                .map(|(key, _)| key)
                .collect();
            let newly_introduced: BTreeSet<_> = issue_details(after, owner)
                .into_iter()
                .filter(|(key, _)| !previous.contains(key))
                .map(|(_, text)| format!("{owner} · {text}"))
                .collect();
            result.new_issues.extend(newly_introduced);
        }
        result
    }
}
fn graph_empty(f: &Flow) -> bool {
    f.steps.is_empty() && f.transitions.is_empty() && f.data.is_empty()
}
fn empties(a: &Flow, b: &Flow) -> bool {
    graph_empty(b)
        && (!graph_empty(a)
            || ((!a.primitive.is_empty() || a.extraction.is_some())
                && b.primitive.is_empty()
                && b.extraction.is_none()))
}
fn records<T: PartialEq>(
    out: &mut BehaviorChanges,
    owner: &str,
    kind: &str,
    (a, b): (&[T], &[T]),
    id: fn(&T) -> &str,
    describe: fn(&T) -> String,
    edits: fn(&T, &T) -> Vec<String>,
) {
    let a: BTreeMap<_, _> = a.iter().map(|v| (id(v), v)).collect();
    let b: BTreeMap<_, _> = b.iter().map(|v| (id(v), v)).collect();
    for id in a.keys().chain(b.keys()).copied().collect::<BTreeSet<_>>() {
        match (a.get(id), b.get(id)) {
            (None, Some(v)) => out.add(owner, "Added", format!("{kind} {id}: {}", describe(v))),
            (Some(v), None) => out.add(owner, "Removed", format!("{kind} {id}: {}", describe(v))),
            (Some(a), Some(b)) if a != b => out.add(
                owner,
                "Edited",
                format!("{kind} {id}: {}", edits(a, b).join("; ")),
            ),
            _ => {}
        }
    }
}
fn field<T: Debug + PartialEq>(out: &mut Vec<String>, name: &str, a: &T, b: &T) {
    if a != b {
        out.push(format!("{name}: {a:?} → {b:?}"));
    }
}
fn step(s: &Step) -> String {
    format!(
        "{:?}, {}; purpose {:?}; target {:?}; information reviewed {}",
        s.name,
        s.kind.label(),
        s.purpose,
        s.target,
        s.information_reviewed
    )
}
fn step_edits(a: &Step, b: &Step) -> Vec<String> {
    let mut out = vec![];
    field(&mut out, "name", &a.name, &b.name);
    field(&mut out, "kind", &a.kind, &b.kind);
    field(&mut out, "purpose", &a.purpose, &b.purpose);
    field(&mut out, "target", &a.target, &b.target);
    field(
        &mut out,
        "information reviewed",
        &a.information_reviewed,
        &b.information_reviewed,
    );
    out
}
fn transition(t: &Transition) -> String {
    format!(
        "{} → {}; condition {:?}; outcome {:?}",
        t.from, t.to, t.condition, t.outcome
    )
}
fn transition_edits(a: &Transition, b: &Transition) -> Vec<String> {
    let mut out = vec![];
    field(&mut out, "from", &a.from, &b.from);
    field(&mut out, "to", &a.to, &b.to);
    field(&mut out, "condition", &a.condition, &b.condition);
    field(&mut out, "outcome", &a.outcome, &b.outcome);
    out
}
fn endpoint(e: &DataEnd) -> String {
    format!(
        "{}; port {}",
        e.step.as_deref().unwrap_or("scope boundary"),
        e.port.as_deref().unwrap_or("Unassigned")
    )
}
fn data(d: &DataLink) -> String {
    format!(
        "{:?}; producer [{}]; consumer [{}]; contract {:?}; interface wire {:?}",
        d.name,
        endpoint(&d.from),
        endpoint(&d.to),
        d.contract,
        d.exchange
    )
}
fn data_edits(a: &DataLink, b: &DataLink) -> Vec<String> {
    let mut out = vec![];
    field(&mut out, "name", &a.name, &b.name);
    field(
        &mut out,
        "producer / port",
        &endpoint(&a.from),
        &endpoint(&b.from),
    );
    field(
        &mut out,
        "consumer / port",
        &endpoint(&a.to),
        &endpoint(&b.to),
    );
    field(&mut out, "contract", &a.contract, &b.contract);
    field(&mut out, "interface wire", &a.exchange, &b.exchange);
    out
}
