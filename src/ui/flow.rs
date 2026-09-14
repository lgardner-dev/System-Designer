//! Guided authoring in the ordinary Designer, using the shared Project and Store.
use super::*;
use crate::{
    behavior::{self, *},
    edit,
};
use std::collections::BTreeSet;
mod drawing;
mod forms;
mod inspect;
mod scene;
#[cfg(test)]
mod tests;

#[derive(Clone, Default, Debug, PartialEq)]
pub(super) struct FlowSelection {
    pub steps: BTreeSet<String>,
    pub transition: Option<String>,
    pub data: Option<String>,
}
impl FlowSelection {
    pub fn normalize(&mut self, flow: Option<&Flow>) {
        let Some(f) = flow else {
            *self = Self::default();
            return;
        };
        self.steps.retain(|id| f.step(id).is_some());
        if !f
            .transitions
            .iter()
            .any(|t| Some(&t.id) == self.transition.as_ref())
        {
            self.transition = None;
        }
        if !f.data.iter().any(|d| Some(&d.id) == self.data.as_ref()) {
            self.data = None;
        }
    }
    fn step(&mut self, id: String, additive: bool) {
        if !additive {
            self.steps.clear();
        }
        if !self.steps.insert(id.clone()) {
            self.steps.remove(&id);
        }
        self.transition = None;
        self.data = None;
    }
}
#[derive(Default)]
pub(super) struct FlowState {
    pub selection: FlowSelection,
    gesture: Option<drawing::Gesture>,
    frozen: Option<(
        std::collections::BTreeMap<String, egui::Rect>,
        scene::Anchors,
    )>,
    pending: Option<(String, bool)>,
    pub focus: bool,
    pub success: Option<(String, String)>,
    #[cfg(test)]
    pub handles: std::collections::BTreeMap<(String, bool), egui::Pos2>,
    #[cfg(test)]
    pub rects: std::collections::BTreeMap<String, egui::Rect>,
    #[cfg(test)]
    pub paths: std::collections::BTreeMap<String, canvas::geometry::Path>,
}
impl FlowState {
    pub fn cancel(&mut self) {
        self.gesture = None;
        self.frozen = None;
        self.pending = None;
    }
    pub fn has_gesture(&self) -> bool {
        self.gesture.is_some() || self.pending.is_some()
    }
}
pub(super) struct ExtractDialog {
    pub members: BTreeSet<String>,
    pub name: String,
    pub purpose: String,
    pub plan: Option<Extraction>,
    pub problem: Option<String>,
    pub suggestions: Vec<Region>,
}
pub(super) struct InformationDialog {
    pub link: DataLink,
    pub define_new: bool,
    pub definition: Contract,
    pub reviewed: Option<String>,
}
pub(super) struct PortDialog {
    pub id: String,
    pub chosen: Option<ContractRef>,
    pub define_new: bool,
    pub definition: Contract,
    pub reviewed: Option<String>,
}
pub(super) enum FlowDialog {
    Start,
    Step(Step),
    Transition(Transition),
    Primitive(String),
    Extract(Box<ExtractDialog>),
    Information(InformationDialog),
    Port(PortDialog),
    Delete(FlowSelection),
    Uses(String),
}
impl Designer {
    pub(super) fn open_port_refinement(&mut self, id: &str) {
        let port = self
            .store
            .project()
            .systems
            .iter()
            .flat_map(|s| &s.nodes)
            .flat_map(|n| &n.ports)
            .find(|p| p.id == id);
        if let Some(port) = port {
            self.dialog = Some(Dialog::Flow(FlowDialog::Port(PortDialog {
                id: id.into(),
                chosen: port.contract.clone(),
                define_new: false,
                definition: Contract::draft(format!("Contract.{}", uuid::Uuid::new_v4())),
                reviewed: None,
            })));
        }
    }
    fn new_step(&mut self, kind: StepKind) {
        let Some(f) = self.store.project().behavior.get(&self.owner) else {
            return;
        };
        let mut step = Step::new(
            f.fresh("step"),
            kind,
            match kind {
                StepKind::Action => "What happens?",
                StepKind::Decision => "What question is answered?",
                StepKind::Merge => "Merge alternatives",
                StepKind::Entry => "Begin",
                StepKind::Outcome => "Complete",
                StepKind::Call => "Use component",
            },
        );
        if kind == StepKind::Call {
            step.target = self
                .interface_system()
                .and_then(|s| s.nodes.first().map(|n| n.id.clone()));
        }
        self.dialog = Some(Dialog::Flow(FlowDialog::Step(step)));
    }
    fn transition_dialog(&mut self, endpoints: Option<(String, String)>) {
        let Some(f) = self.store.project().behavior.get(&self.owner) else {
            return;
        };
        let (from, to) = endpoints.unwrap_or_else(|| {
            (
                self.flow
                    .selection
                    .steps
                    .iter()
                    .next()
                    .cloned()
                    .or_else(|| {
                        f.steps
                            .iter()
                            .find(|s| s.kind != StepKind::Outcome)
                            .map(|s| s.id.clone())
                    })
                    .unwrap_or_default(),
                f.steps
                    .iter()
                    .find(|s| s.kind != StepKind::Entry)
                    .map(|s| s.id.clone())
                    .unwrap_or_default(),
            )
        });
        self.dialog = Some(Dialog::Flow(FlowDialog::Transition(Transition {
            id: f.fresh("transition"),
            from,
            to,
            condition: String::new(),
            outcome: None,
        })));
    }
    fn extraction_dialog(&mut self, suggest: bool) {
        let Some(f) = self.store.project().behavior.get(&self.owner) else {
            return;
        };
        let (suggestions, problem) = if suggest {
            match behavior::candidates(f) { Ok(v) if v.is_empty() => (v,Some("No structural candidates found. Select a local region manually and review its boundary.".into())), Ok(v)=>(v,None),Err(e)=>(vec![],Some(e.to_string())) }
        } else {
            (vec![], None)
        };
        self.dialog = Some(Dialog::Flow(FlowDialog::Extract(Box::new(ExtractDialog {
            members: self.flow.selection.steps.clone(),
            name: String::new(),
            purpose: String::new(),
            plan: None,
            problem,
            suggestions,
        }))));
    }
    fn information_dialog(&mut self, existing: Option<&DataLink>) {
        let Some(f) = self.store.project().behavior.get(&self.owner) else {
            return;
        };
        let link = existing.cloned().unwrap_or_else(|| DataLink {
            id: f.fresh("information"),
            name: "What information is needed?".into(),
            from: DataEnd {
                step: None,
                port: None,
            },
            to: DataEnd {
                step: self
                    .flow
                    .selection
                    .steps
                    .iter()
                    .next()
                    .cloned()
                    .or_else(|| {
                        f.steps
                            .iter()
                            .find(|s| s.kind == StepKind::Action)
                            .map(|s| s.id.clone())
                    }),
                port: None,
            },
            contract: None,
            exchange: None,
        });
        self.dialog = Some(Dialog::Flow(FlowDialog::Information(InformationDialog {
            link,
            define_new: false,
            definition: Contract::draft(format!("Contract.{}", uuid::Uuid::new_v4())),
            reviewed: None,
        })));
    }
    pub(super) fn ask_delete_flow(&mut self) {
        if self.flow.selection != FlowSelection::default() {
            self.dialog = Some(Dialog::Flow(FlowDialog::Delete(
                self.flow.selection.clone(),
            )));
        }
    }
    fn accept_extraction(&mut self, plan: &Extraction) {
        let owner = self.owner.clone();
        self.publish(
            "Extract responsibility",
            behavior::apply(self.store.project(), plan),
        );
        if self.error.is_none() {
            self.flow.selection = FlowSelection::default();
            self.flow.selection.steps.insert(plan.call.clone());
            self.flow.success = Some((owner, plan.component.clone()));
        }
    }
}
