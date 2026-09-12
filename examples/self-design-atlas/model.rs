//! Read-only, source-mapped self-design. This is NOT the production project format.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub use system_designer::model::*;

pub const ATLAS: &str = include_str!("../../design/system-designer.atlas.json");
pub const PROMPT: &str = include_str!("initialization.txt");
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Atlas {
    pub format: String,
    pub version: u32,
    pub baseline: String,
    pub purpose: String,
    pub project: Project,
    pub scopes: Vec<Behavior>,
    pub source_hashes: BTreeMap<String, String>,
    pub limits: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: String,
    pub symbol: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Behavior {
    pub owner: String,
    pub title: String,
    pub notes: String,
    pub sources: Vec<Source>,
    pub primitive_stop: Option<String>,
    pub planned: Vec<String>,
    pub steps: Vec<Step>,
    pub transitions: Vec<Transition>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Entry,
    Outcome,
    Action,
    Decision,
    Call,
    Io,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub label: String,
    pub kind: Kind,
    pub rule: String,
    pub inputs: String,
    pub outputs: String,
    pub target: Option<String>,
    pub source: Option<Source>,
    pub status: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String,
    pub exchanges: Vec<String>,
}
impl Atlas {
    pub fn load() -> std::result::Result<Self, String> {
        Self::parse(ATLAS)
    }
    pub fn parse(text: &str) -> std::result::Result<Self, String> {
        let a: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        a.check()?;
        Ok(a)
    }
    pub fn scope(&self, owner: &str) -> Option<&Behavior> {
        self.scopes.iter().find(|s| s.owner == owner)
    }
    pub fn name(&self, owner: &str) -> &str {
        if owner == "@root" {
            "System Designer"
        } else {
            self.project
                .node(owner)
                .map(|(_, n)| n.name.as_str())
                .unwrap_or("Missing component")
        }
    }
    pub fn system(&self, owner: &str) -> Option<&System> {
        let sid = if owner == "@root" {
            Some(self.project.root.as_str())
        } else {
            self.project.node(owner)?.1.child.as_deref()
        };
        sid.and_then(|id| self.project.system(id))
    }
    pub fn lineage(&self, owner: &str) -> Vec<String> {
        let mut result = vec![owner.to_owned()];
        let mut current = owner;
        while current != "@root" {
            let Some((system, _)) = self.project.node(current) else {
                break;
            };
            current = self
                .project
                .owner(&system.id)
                .map(|(_, n)| n.id.as_str())
                .unwrap_or("@root");
            result.push(current.into());
        }
        result.reverse();
        result
    }
    pub fn check(&self) -> std::result::Result<(), String> {
        let err = |s: &str| Err(s.to_owned());
        if self.format != "system-designer-self-design-atlas" || self.version != 1 {
            return err("Unsupported self-design atlas");
        }
        validate(&self.project).map_err(|e| e.to_string())?;
        let expected: BTreeSet<_> = std::iter::once("@root")
            .chain(
                self.project
                    .systems
                    .iter()
                    .flat_map(|s| s.nodes.iter().map(|n| n.id.as_str())),
            )
            .collect();
        let owners: BTreeSet<_> = self.scopes.iter().map(|s| s.owner.as_str()).collect();
        if owners != expected || owners.len() != self.scopes.len() {
            return err("Exactly one behavior scope per component and root is required");
        }
        for b in &self.scopes {
            if b.title.trim().is_empty() || b.sources.is_empty() {
                return err("Scope needs a title and source evidence");
            }
            if self.system(&b.owner).is_none()
                && b.primitive_stop
                    .as_deref()
                    .is_none_or(|r| r.trim().is_empty())
            {
                return err("Leaf scope needs an explicit primitive stopping rule");
            }
            let ids: BTreeSet<_> = b.steps.iter().map(|s| s.id.as_str()).collect();
            if ids.len() != b.steps.len() {
                return err("Duplicate local step identity");
            }
            if b.steps.iter().filter(|s| s.kind == Kind::Entry).count() != 1
                || !b.steps.iter().any(|s| s.kind == Kind::Outcome)
            {
                return err("A scoped flow needs one entry and at least one outcome");
            }
            let mut edges = BTreeSet::new();
            for s in &b.steps {
                if s.id.trim().is_empty() || s.label.trim().is_empty() || s.rule.trim().is_empty() {
                    return err("Step needs identity, name and explicit rule");
                }
                if s.kind == Kind::Call {
                    let Some(target) = &s.target else {
                        return err("Call without target");
                    };
                    if !self
                        .system(&b.owner)
                        .is_some_and(|sys| sys.nodes.iter().any(|n| &n.id == target))
                    {
                        return err(
                            "Call must reference an immediate child, not a grandchild or copied component",
                        );
                    }
                } else if s.target.is_some() {
                    return err("Only calls own component references");
                }
                if !["mapped", "proposed"].contains(&s.status.as_str()) {
                    return err("Unknown implementation status");
                }
                if let Some(source) = &s.source {
                    if !self.source_hashes.contains_key(&source.path)
                        || source.symbol.trim().is_empty()
                    {
                        return err("Unbound step source");
                    }
                }
            }
            for t in &b.transitions {
                if t.id.is_empty()
                    || !edges.insert(&t.id)
                    || !ids.contains(t.from.as_str())
                    || !ids.contains(t.to.as_str())
                {
                    return err("Invalid transition identity or local endpoints");
                }
                for e in &t.exchanges {
                    if !self
                        .system(&b.owner)
                        .is_some_and(|s| s.edges.iter().any(|x| &x.id == e))
                    {
                        return err("Transition exchange must be an exact local interface edge");
                    }
                }
            }
            for s in &b.steps {
                let incoming = b.transitions.iter().filter(|t| t.to == s.id).count();
                let outgoing = b.transitions.iter().filter(|t| t.from == s.id).count();
                if s.kind == Kind::Entry && incoming != 0 {
                    return err("Entry has an incoming transition");
                }
                if s.kind == Kind::Outcome && outgoing != 0 {
                    return err("Outcome has an outgoing transition");
                }
                if s.kind != Kind::Outcome && outgoing == 0 {
                    return err("Unspecified dead end; declare an outcome");
                }
                if s.kind == Kind::Decision
                    && (outgoing < 2
                        || b.transitions
                            .iter()
                            .filter(|t| t.from == s.id)
                            .any(|t| t.label.trim().is_empty()))
                {
                    return err("Decision alternatives must be distinct labeled transitions");
                }
            }
            let entry = b
                .steps
                .iter()
                .find(|s| s.kind == Kind::Entry)
                .ok_or("Missing entry")?;
            let mut seen = BTreeSet::new();
            let mut todo = vec![entry.id.as_str()];
            while let Some(id) = todo.pop() {
                if seen.insert(id) {
                    todo.extend(
                        b.transitions
                            .iter()
                            .filter(|t| t.from == id)
                            .map(|t| t.to.as_str()),
                    );
                }
            }
            if seen.len() != ids.len() {
                return err("Unreachable local behavior step");
            }
            let mut seen = BTreeSet::new();
            let mut todo: Vec<_> = b
                .steps
                .iter()
                .filter(|s| s.kind == Kind::Outcome)
                .map(|s| s.id.as_str())
                .collect();
            while let Some(id) = todo.pop() {
                if seen.insert(id) {
                    todo.extend(
                        b.transitions
                            .iter()
                            .filter(|t| t.to == id)
                            .map(|t| t.from.as_str()),
                    );
                }
            }
            if seen.len() != ids.len() {
                return err("A step has no structural path to an outcome");
            }
            for s in &b.sources {
                if !self.source_hashes.contains_key(&s.path) {
                    return err("Unbound scope source");
                }
            }
        }
        Ok(())
    }
}
impl Behavior {
    pub fn step(&self, id: &str) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }
    pub fn metric(&self) -> usize {
        // A virtual common exit joins declared outcomes for the E-N+2 convention.
        self.transitions.len()
            + self
                .steps
                .iter()
                .filter(|s| s.kind == Kind::Outcome)
                .count()
            + 1
            - self.steps.len()
    }
    pub fn work_steps(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| !matches!(s.kind, Kind::Entry | Kind::Outcome))
            .count()
    }
}
