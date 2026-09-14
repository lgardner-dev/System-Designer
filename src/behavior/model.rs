use crate::model::{ContractRef, ModelError, Project, Result, System};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const ROOT: &str = "@root";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Entry,
    Action,
    Decision,
    Merge,
    Outcome,
    Call,
}
impl StepKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Entry => "Entry",
            Self::Action => "Action",
            Self::Decision => "Decision",
            Self::Merge => "Merge alternatives",
            Self::Outcome => "Outcome",
            Self::Call => "Use component",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub name: String,
    pub kind: StepKind,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub target: Option<String>,
    /// Human review state, not a proof of dependency completeness.
    #[serde(default)]
    pub information_reviewed: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub id: String,
    pub from: String,
    pub to: String,
    /// Descriptive condition, never evaluated.
    #[serde(default)]
    pub condition: String,
    /// Exact outcome step in the referenced child's flow, for a call return.
    #[serde(default)]
    pub outcome: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataEnd {
    /// None is the current scope's boundary, not an anonymous producer.
    pub step: Option<String>,
    /// Optional exact interface port; local actions do not have interface ports.
    pub port: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataLink {
    pub id: String,
    pub name: String,
    pub from: DataEnd,
    pub to: DataEnd,
    pub contract: Option<ContractRef>,
    /// Exact, optional, parent-local interface edge association.
    pub exchange: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Flow {
    pub steps: Vec<Step>,
    pub transitions: Vec<Transition>,
    #[serde(default)]
    pub data: Vec<DataLink>,
    #[serde(default)]
    pub primitive: String,
    #[serde(default)]
    pub extraction: Option<Provenance>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub rule_version: u32,
    pub parent: String,
    pub source_steps: Vec<String>,
    pub responsibility: String,
    /// Historical references only; source steps now live in this child.
    pub base: String,
}
impl Step {
    pub fn new(id: impl Into<String>, kind: StepKind, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            purpose: String::new(),
            target: None,
            information_reviewed: false,
        }
    }
}
impl Flow {
    pub fn step(&self, id: &str) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }
    pub fn fresh(&self, prefix: &str) -> String {
        (1u64..)
            .map(|i| format!("{prefix}.{i}"))
            .find(|id| {
                !self.steps.iter().any(|s| s.id == *id)
                    && !self.transitions.iter().any(|s| s.id == *id)
                    && !self.data.iter().any(|s| s.id == *id)
            })
            .expect("finite model")
    }
    pub fn starter() -> Self {
        Self {
            steps: vec![
                Step::new("entry", StepKind::Entry, "Begin"),
                Step::new("action", StepKind::Action, "Describe the work"),
                Step::new("done", StepKind::Outcome, "Complete"),
            ],
            transitions: vec![
                Transition {
                    id: "begin".into(),
                    from: "entry".into(),
                    to: "action".into(),
                    condition: String::new(),
                    outcome: None,
                },
                Transition {
                    id: "finish".into(),
                    from: "action".into(),
                    to: "done".into(),
                    condition: String::new(),
                    outcome: None,
                },
            ],
            ..Default::default()
        }
    }
}
pub fn system<'a>(p: &'a Project, owner: &str) -> Option<&'a System> {
    if owner == ROOT {
        p.system(&p.root)
    } else {
        p.node(owner)
            .and_then(|(_, n)| n.child.as_deref())
            .and_then(|sid| p.system(sid))
    }
}
pub fn exists(p: &Project, owner: &str) -> bool {
    owner == ROOT || p.node(owner).is_some()
}
pub fn name<'a>(p: &'a Project, owner: &str) -> &'a str {
    if owner == ROOT {
        &p.name
    } else {
        p.node(owner)
            .map(|(_, n)| n.name.as_str())
            .unwrap_or("Missing component")
    }
}
pub fn set(p: &Project, owner: &str, flow: Flow) -> Result<Project> {
    if !exists(p, owner) {
        return Err(ModelError::one("Missing behavior owner"));
    }
    crate::edit::candidate(p, |q| {
        if q.version == 1 {
            q.version = 2;
        }
        q.behavior.insert(owner.into(), flow);
        prune_layout(q);
        Ok(())
    })
}
pub fn prune_layout(p: &mut Project) {
    p.flow_layout.retain(|owner, positions| {
        let Some(flow) = p.behavior.get(owner) else {
            return false;
        };
        positions.retain(|id, _| flow.step(id).is_some());
        true
    });
}
pub fn fingerprint(p: &Project) -> String {
    let mut q = p.clone();
    q.layout.clear();
    q.flow_layout.clear();
    crate::exchange::digest(
        crate::exchange::canonical_json(&serde_json::to_value(q).expect("project serializes"))
            .as_bytes(),
    )
}

/// Material edits invalidate human information-use review on every incident local
/// action/decision. This is conservative review state, never a completeness proof.
pub fn invalidate_reviews(before: &Project, after: &mut Project) {
    for (owner, flow) in &mut after.behavior {
        let Some(old) = before.behavior.get(owner) else {
            continue;
        };
        for step in &mut flow.steps {
            if !matches!(step.kind, StepKind::Action | StepKind::Decision) {
                continue;
            }
            let meaning_changed = old.step(&step.id).is_some_and(|s| {
                s.name != step.name
                    || s.purpose != step.purpose
                    || s.kind != step.kind
                    || s.target != step.target
            });
            let incident = |d: &&DataLink| {
                d.from.step.as_ref() == Some(&step.id) || d.to.step.as_ref() == Some(&step.id)
            };
            let a: BTreeMap<_, _> = old
                .data
                .iter()
                .filter(incident)
                .map(|d| (&d.id, d))
                .collect();
            let b: BTreeMap<_, _> = flow
                .data
                .iter()
                .filter(incident)
                .map(|d| (&d.id, d))
                .collect();
            let definition_changed = b.values().filter_map(|d| d.contract.as_ref()).any(|r| {
                before.contract(r) != after.contracts.iter().find(|c| c.reference() == *r)
            });
            if meaning_changed || a != b || definition_changed {
                step.information_reviewed = false;
            }
        }
    }
}
