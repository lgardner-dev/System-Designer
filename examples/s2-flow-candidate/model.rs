//! Isolated study format. Production Project/version-1 and exchange are unchanged.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use system_designer::model::{self, Project};

pub const STUDY: &str = include_str!("control-flow.study.json");
pub const INTERFACES: &str = include_str!("interfaces.project.json");
pub const SOURCE: &str = include_str!("source-S2.json");
pub const PROMPT: &str = include_str!("initialization.txt");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Study {
    pub format: String,
    pub version: u32,
    pub status: String,
    pub baseline_commit: String,
    pub provenance: BTreeMap<String, String>,
    pub source: Value,
    pub flows: Vec<Flow>,
    pub call_binding: CallBinding,
    pub limits: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Flow {
    pub id: String,
    pub owner: Option<String>,
    pub system: String,
    pub name: String,
    pub entry: String,
    pub steps: Vec<Step>,
    pub transitions: Vec<Transition>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub id: String,
    pub component: Option<String>,
    pub kind: Kind,
    pub name: String,
    pub purpose: String,
    pub source_steps: Vec<String>,
    pub position: [f32; 2],
    pub call: Option<String>,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Entry,
    Action,
    Choice,
    Call,
    Outcome,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub id: String,
    pub from: String,
    pub to: String,
    pub condition: String,
    pub label: String,
    pub source_flow: String,
    pub exchanges: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CallBinding {
    pub step: String,
    pub child_flow: String,
    pub entry: String,
    pub r#return: String,
    pub incoming_source: String,
    pub outgoing_source: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct SourceStep {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub purpose: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct SourceEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub contracts: Vec<String>,
    pub label: String,
    pub condition: String,
}
#[derive(Clone)]
pub struct Document {
    pub study: Study,
    pub project: Project,
    pub source_steps: Vec<SourceStep>,
    pub source_edges: Vec<SourceEdge>,
}
fn digest(value: &Value) -> Result<String, String> {
    // This fixture has ASCII object keys; serde_json's sorted map canonicalizes
    // them. This is NOT a replacement for the production scope hash convention.
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
impl Flow {
    pub fn step(&self, id: &str) -> Option<&Step> {
        self.steps.iter().find(|s| s.id == id)
    }
    pub fn transition(&self, id: &str) -> Option<&Transition> {
        self.transitions.iter().find(|t| t.id == id)
    }
    pub fn metric(&self) -> Result<usize, String> {
        complexity(
            &self.steps.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
            &self
                .transitions
                .iter()
                .map(|e| (e.from.clone(), e.to.clone()))
                .collect::<Vec<_>>(),
            &self.entry,
            &self
                .steps
                .iter()
                .filter(|s| s.kind == Kind::Outcome)
                .map(|s| s.id.clone())
                .collect::<Vec<_>>(),
        )
    }
    pub fn local_steps(&self) -> usize {
        self.steps.iter().filter(|s| s.component.is_some()).count()
    }
}
impl Document {
    pub fn load() -> Result<Self, String> {
        Self::parse(STUDY, INTERFACES)
    }
    pub fn parse(study: &str, interfaces: &str) -> Result<Self, String> {
        let study: Study = serde_json::from_str(study).map_err(|e| e.to_string())?;
        let project = model::parse(interfaces).map_err(|e| e.to_string())?;
        let source_steps =
            serde_json::from_value(study.source["steps"].clone()).map_err(|e| e.to_string())?;
        let source_edges =
            serde_json::from_value(study.source["flows"].clone()).map_err(|e| e.to_string())?;
        let document = Self {
            study,
            project,
            source_steps,
            source_edges,
        };
        document.validate()?;
        Ok(document)
    }
    pub fn flow(&self, id: &str) -> Option<&Flow> {
        self.study.flows.iter().find(|f| f.id == id)
    }
    pub fn original(&self) -> Flow {
        let steps = self
            .source_steps
            .iter()
            .map(|s| {
                let (position, kind) = match s.id.as_str() {
                    "S2.frame" => ([55.0, 110.0], Kind::Action),
                    "S2.prior" => ([350.0, 110.0], Kind::Action),
                    "S2.enough" => ([645.0, 110.0], Kind::Choice),
                    "S2.hyp" => ([1010.0, 185.0], Kind::Action),
                    "S2.plan" => ([1010.0, 375.0], Kind::Action),
                    "S2.run" => ([1010.0, 565.0], Kind::Action),
                    "S2.interpret" => ([645.0, 475.0], Kind::Action),
                    "S2.decide" => ([350.0, 475.0], Kind::Choice),
                    "S2.exit" => ([55.0, 705.0], Kind::Outcome),
                    _ => ([645.0, 705.0], Kind::Outcome),
                };
                Step {
                    id: format!("step.{}", s.id),
                    component: Some(s.id.clone()),
                    kind,
                    name: s.name.clone(),
                    purpose: s.purpose.clone(),
                    source_steps: vec![s.id.clone()],
                    position,
                    call: None,
                }
            })
            .collect();
        let transitions = self
            .source_edges
            .iter()
            .map(|e| Transition {
                id: e.id.clone(),
                from: format!("step.{}", e.source),
                to: format!("step.{}", e.target),
                label: e.label.clone(),
                condition: e.condition.clone(),
                source_flow: e.id.clone(),
                exchanges: vec![],
            })
            .collect();
        Flow {
            id: "reference.S2".into(),
            owner: None,
            system: self.project.root.clone(),
            name: "Original Atlas S2 · comparison".into(),
            entry: "step.S2.frame".into(),
            steps,
            transitions,
        }
    }
    pub fn source_edge(&self, id: &str) -> Option<&SourceEdge> {
        self.source_edges.iter().find(|e| e.id == id)
    }
    pub fn validate(&self) -> Result<(), String> {
        let fail = |s: &str| Err(s.to_owned());
        if self.study.format != "system-designer-control-flow-study"
            || self.study.version != 1
            || self.study.status != "proposal-only"
        {
            return fail("Not a supported proposal-only study");
        }
        model::validate(&self.project).map_err(|e| e.to_string())?;
        if digest(&self.study.source)?
            != *self
                .study
                .provenance
                .get("source_S2_sha256")
                .ok_or("Missing source digest")?
        {
            return fail("Original S2 source digest mismatch");
        }
        if digest(&serde_json::to_value(&self.project.contracts).map_err(|e| e.to_string())?)?
            != *self
                .study
                .provenance
                .get("retained_catalog_sha256")
                .ok_or("Missing catalog digest")?
        {
            return fail("Retained contract catalog differs from the source binding");
        }
        if self.study.flows.len() != 2 {
            return fail("This worked candidate has exactly two declared scopes");
        }
        let original: Value = serde_json::from_str(SOURCE).map_err(|e| e.to_string())?;
        if self.study.source != original {
            return fail("Source is not the bound original Atlas S2");
        }
        let mut identities = BTreeSet::new();
        let mut origins = BTreeMap::<String, usize>::new();
        let mut reconstructed = BTreeSet::new();
        for flow in &self.study.flows {
            if !identities.insert(flow.id.clone()) {
                return fail("Duplicate flow ID");
            }
            let sys = self
                .project
                .system(&flow.system)
                .ok_or("Missing local system")?;
            if flow.owner.as_ref() != self.project.owner(&flow.system).map(|(_, n)| &n.id) {
                return fail("Flow owner differs from interface owner");
            }
            if flow.step(&flow.entry).is_none() {
                return fail("Missing flow entry");
            }
            let mut referenced_edges = BTreeSet::new();
            for step in &flow.steps {
                if !identities.insert(step.id.clone()) {
                    return fail("Duplicate step ID");
                }
                if step.name.trim().is_empty()
                    || step.position.iter().any(|x| !x.is_finite() || *x < 0.0)
                {
                    return fail("Invalid step name or position");
                }
                if let Some(component) = &step.component {
                    if !sys.nodes.iter().any(|n| n.id == *component) {
                        return fail("Step references a nonlocal component");
                    }
                } else if !matches!(step.kind, Kind::Entry | Kind::Outcome) {
                    return fail("Only the study's derived boundary markers lack components");
                }
                if step.kind == Kind::Call {
                    let child = self
                        .flow(step.call.as_deref().ok_or("Call has no target")?)
                        .ok_or("Missing called flow")?;
                    if child.owner != step.component {
                        return fail("Call target is not its declared component");
                    }
                    let target_node = self
                        .project
                        .node(step.component.as_deref().ok_or("Call has no component")?)
                        .ok_or("Missing call component")?
                        .1;
                    if target_node.child.as_deref() != Some(&child.system) {
                        return fail("Call bypasses its child boundary");
                    }
                } else {
                    if step.call.is_some() {
                        return fail("Non-call step has a call target");
                    }
                    for origin in &step.source_steps {
                        *origins.entry(origin.clone()).or_default() += 1;
                        let source = self
                            .source_steps
                            .iter()
                            .find(|s| s.id == *origin)
                            .ok_or("Missing source step")?;
                        if source.name != step.name || source.purpose != step.purpose {
                            return fail("A retained source step was rewritten");
                        }
                    }
                }
            }
            for edge in &flow.transitions {
                if !identities.insert(edge.id.clone()) {
                    return fail("Duplicate transition ID");
                }
                let from = flow.step(&edge.from).ok_or("Missing source step")?;
                let to = flow.step(&edge.to).ok_or("Missing destination step")?;
                let source = self
                    .source_edge(&edge.source_flow)
                    .ok_or("Missing source transition")?;
                let boundary = from.kind == Kind::Entry
                    || (to.component.is_none() && to.kind == Kind::Outcome);
                if edge.label != source.label || (!boundary && edge.condition != source.condition) {
                    return fail("Source transition label/condition was changed");
                }
                if boundary && !edge.condition.is_empty() {
                    return fail("Boundary projection must not repeat a parent guard");
                }
                let mut carried = Vec::new();
                for id in &edge.exchanges {
                    if !referenced_edges.insert(id.clone()) {
                        return fail("An interface exchange is assigned to multiple transitions");
                    }
                    let wire = sys
                        .edges
                        .iter()
                        .find(|e| e.id == *id)
                        .ok_or("Missing or nonlocal exchange")?;
                    if wire.from.node != from.component || wire.to.node != to.component {
                        return fail("Flow/interface endpoints disagree");
                    }
                    let contract = self
                        .project
                        .port(&flow.system, &wire.from)
                        .and_then(|p| p.contract.as_ref())
                        .ok_or("Unbound exchange")?;
                    carried.push(contract.id.clone());
                }
                if carried != source.contracts {
                    return fail("Transition's exchanged contracts differ from source");
                }
                if !boundary {
                    if !reconstructed.insert(source.id.clone()) {
                        return fail("Duplicated source transition");
                    }
                    let source_ids = |step: &Step, entering: bool| -> Result<String, String> {
                        if step.kind == Kind::Call {
                            let child = self
                                .flow(step.call.as_deref().ok_or("Missing call")?)
                                .ok_or("Missing child")?;
                            let marker = if entering {
                                &self.study.call_binding.entry
                            } else {
                                &self.study.call_binding.r#return
                            };
                            let link = child
                                .transitions
                                .iter()
                                .find(|e| {
                                    if entering {
                                        e.from == *marker
                                    } else {
                                        e.to == *marker
                                    }
                                })
                                .ok_or("Missing entry/return continuation")?;
                            let id = if entering { &link.to } else { &link.from };
                            child
                                .step(id)
                                .and_then(|s| s.source_steps.first())
                                .cloned()
                                .ok_or("Missing child source step".into())
                        } else {
                            step.source_steps
                                .first()
                                .cloned()
                                .ok_or("Missing original step".into())
                        }
                    };
                    if source_ids(from, false)? != source.source
                        || source_ids(to, true)? != source.target
                    {
                        return fail("Flattened control endpoints differ from original S2");
                    }
                }
            }
            if referenced_edges.len() != sys.edges.len() {
                return fail("Interface exchange omitted from study mapping");
            }
            flow.metric()?;
        }
        if origins.len() != self.source_steps.len() || origins.values().any(|n| *n != 1) {
            return fail("Source steps omitted or duplicated");
        }
        if reconstructed.len() != self.source_edges.len() {
            return fail("Source transition omitted");
        }
        let binding = &self.study.call_binding;
        let child = self
            .flow(&binding.child_flow)
            .ok_or("Invalid call binding")?;
        let parent = &self.study.flows[0];
        let call = parent.step(&binding.step).ok_or("Invalid call site")?;
        if call.call.as_deref() != Some(&binding.child_flow) || child.entry != binding.entry {
            return fail("Entry binding changed");
        }
        let ret = child
            .step(&binding.r#return)
            .ok_or("Missing returned outcome")?;
        if ret.kind != Kind::Outcome || ret.component.is_some() {
            return fail("Return is not a boundary outcome");
        }
        for (source, marker, incoming) in [
            (&binding.incoming_source, &binding.entry, true),
            (&binding.outgoing_source, &binding.r#return, false),
        ] {
            let outer = parent
                .transitions
                .iter()
                .find(|e| e.source_flow == *source)
                .ok_or("Missing parent handoff")?;
            let inner = child
                .transitions
                .iter()
                .find(|e| e.source_flow == *source)
                .ok_or("Missing child handoff")?;
            if (incoming && (outer.to != call.id || inner.from != *marker))
                || (!incoming && (outer.from != call.id || inner.to != *marker))
            {
                return fail("Invalid call/return mapping");
            }
            for (oe, ie) in outer.exchanges.iter().zip(&inner.exchanges) {
                let ow = self
                    .project
                    .system(&parent.system)
                    .ok_or("Missing system")?
                    .edges
                    .iter()
                    .find(|e| e.id == *oe)
                    .ok_or("Missing wire")?;
                let iw = self
                    .project
                    .system(&child.system)
                    .ok_or("Missing system")?
                    .edges
                    .iter()
                    .find(|e| e.id == *ie)
                    .ok_or("Missing wire")?;
                let (op, ip) = if incoming {
                    (&ow.to, &iw.from)
                } else {
                    (&ow.from, &iw.to)
                };
                if op.port != ip.port || ip.node.is_some() {
                    return fail("Boundary port identity drift");
                }
            }
        }
        Ok(())
    }
}

/// For this sequential study only. Normalize all outcomes to one virtual sink.
/// Reject a metric (not a project) when entry/outcome reachability is incomplete.
/// Count modeled branches, NOT feasible executions, tests, or semantic difficulty.
pub fn complexity(
    nodes: &[String],
    edges: &[(String, String)],
    entry: &str,
    outcomes: &[String],
) -> Result<usize, String> {
    let ids: BTreeSet<_> = nodes.iter().cloned().collect();
    if ids.len() != nodes.len() || !ids.contains(entry) || outcomes.is_empty() {
        return Err("Metric unavailable: incomplete entry/outcomes".into());
    }
    if outcomes.iter().any(|o| !ids.contains(o))
        || edges
            .iter()
            .any(|(a, b)| !ids.contains(a) || !ids.contains(b) || outcomes.contains(a))
    {
        return Err("Metric unavailable: malformed endpoints/outcomes".into());
    }
    let reach = |starts: BTreeSet<String>, reverse: bool| {
        let mut result = starts;
        loop {
            let before = result.len();
            for (a, b) in edges {
                let (a, b) = if reverse { (b, a) } else { (a, b) };
                if result.contains(a) {
                    result.insert(b.clone());
                }
            }
            if before == result.len() {
                break;
            }
        }
        result
    };
    if reach(BTreeSet::from([entry.into()]), false) != ids
        || reach(outcomes.iter().cloned().collect(), true) != ids
    {
        return Err("Metric unavailable: unreachable work or no path to an outcome".into());
    }
    (edges.len() + outcomes.len() + 1)
        .checked_sub(nodes.len())
        .ok_or("Metric unavailable".into())
}
