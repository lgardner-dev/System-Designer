//! Exact local membership and reversible navigation; never causal inference.
use super::*;
use crate::model::Edge;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui) enum Focus {
    #[default]
    Off,
    Selection,
    Incoming,
    Outgoing,
}
impl Focus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Selection => "Selection",
            Self::Incoming => "Incoming",
            Self::Outgoing => "Outgoing",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(in crate::ui) struct Viewport {
    pub pan: Vec2,
    pub zoom: f32,
    pub fit: bool,
}
#[derive(Clone, Debug)]
pub(in crate::ui) struct Location {
    pub system: String,
    pub selection: Selection,
    pub view: View,
    pub focus: Focus,
    pub viewport: Viewport,
}
#[derive(Default)]
pub(in crate::ui) struct Session {
    pub view: View,
    pub focus: Focus,
    pub history: Vec<Location>,
    pub viewports: BTreeMap<String, Viewport>,
}
impl Session {
    pub fn normalize(&mut self, selection: &Selection) {
        if *selection == Selection::None {
            self.focus = Focus::Off;
        } else if !matches!(selection, Selection::Node(_))
            && matches!(self.focus, Focus::Incoming | Focus::Outgoing)
        {
            self.focus = Focus::Selection;
        }
    }
}
pub(in crate::ui) fn valid(p: &Project, sid: &str, selection: &Selection) -> bool {
    let Some(system) = p.system(sid) else {
        return false;
    };
    match selection {
        Selection::None => true,
        Selection::Node(id) => system.nodes.iter().any(|n| n.id == *id),
        Selection::Edge(id) => system.edges.iter().any(|e| e.id == *id),
        Selection::Boundary(id) => p.boundary(sid).iter().any(|port| port.id == *id),
        Selection::Port(ep) => p.port(sid, ep).is_some(),
        Selection::Summary(a, b) => {
            a != b
                && system
                    .edges
                    .iter()
                    .any(|e| e.from.node.as_ref() == Some(a) && e.to.node.as_ref() == Some(b))
        }
    }
}
pub(super) fn matches(edge: &Edge, selection: &Selection, direction: Focus) -> bool {
    match selection {
        Selection::None => false,
        Selection::Edge(id) => edge.id == *id,
        Selection::Node(id) => {
            (direction != Focus::Incoming && edge.from.node.as_ref() == Some(id))
                || (direction != Focus::Outgoing && edge.to.node.as_ref() == Some(id))
        }
        Selection::Boundary(id) => {
            (edge.from.node.is_none() && edge.from.port == *id)
                || (edge.to.node.is_none() && edge.to.port == *id)
        }
        Selection::Port(ep) => edge.from == *ep || edge.to == *ep,
        Selection::Summary(a, b) => {
            edge.from.node.as_ref() == Some(a) && edge.to.node.as_ref() == Some(b)
        }
    }
}
#[derive(Default, Debug)]
pub(super) struct Emphasis {
    pub active: bool,
    pub edges: BTreeSet<String>,
    nodes: BTreeSet<String>,
    ports: BTreeSet<String>,
}
impl Emphasis {
    pub fn new(p: &Project, sid: &str, selected: &Selection, focus: Focus) -> Self {
        let mut result = Self {
            active: focus != Focus::Off && *selected != Selection::None,
            ..Self::default()
        };
        match selected {
            Selection::Node(id) => {
                result.nodes.insert(id.clone());
            }
            Selection::Boundary(id) => {
                result.ports.insert(id.clone());
            }
            Selection::Port(ep) => {
                result.ports.insert(ep.port.clone());
                if let Some(n) = &ep.node {
                    result.nodes.insert(n.clone());
                }
            }
            _ => {}
        }
        if let Some(system) = p.system(sid) {
            for edge in &system.edges {
                if matches(edge, selected, focus) {
                    result.edges.insert(edge.id.clone());
                    for ep in [&edge.from, &edge.to] {
                        if let Some(id) = &ep.node {
                            result.nodes.insert(id.clone());
                        }
                        result.ports.insert(ep.port.clone());
                    }
                }
            }
        }
        result
    }
    pub fn edge(&self, id: &str) -> bool {
        !self.active || self.edges.contains(id)
    }
    pub fn node(&self, id: &str) -> bool {
        !self.active || self.nodes.contains(id)
    }
    pub fn port(&self, id: &str) -> bool {
        !self.active || self.ports.contains(id)
    }
}
pub(super) fn endpoint_name(p: &Project, sid: &str, ep: &Endpoint) -> String {
    let node = ep
        .node
        .as_ref()
        .and_then(|id| p.node(id))
        .map(|(_, n)| n.name.as_str())
        .unwrap_or("System boundary");
    let port = p
        .port(sid, ep)
        .map(|r| r.name.as_str())
        .unwrap_or("Missing port");
    format!("{node} / {port}")
}
pub(super) fn description(p: &Project, sid: &str, selection: &Selection) -> String {
    match selection {
        Selection::None => "No selection".into(),
        Selection::Node(id) => p
            .node(id)
            .map(|(_, n)| n.name.clone())
            .unwrap_or_else(|| id.clone()),
        Selection::Edge(id) => format!("Connection {id}"),
        Selection::Boundary(id) => endpoint_name(
            p,
            sid,
            &Endpoint {
                node: None,
                port: id.clone(),
            },
        ),
        Selection::Port(ep) => endpoint_name(p, sid, ep),
        Selection::Summary(a, b) => format!(
            "{} → {}",
            description(p, sid, &Selection::Node(a.clone())),
            description(p, sid, &Selection::Node(b.clone()))
        ),
    }
}
fn port_selection(ep: Endpoint) -> Selection {
    if ep.node.is_none() {
        Selection::Boundary(ep.port)
    } else {
        Selection::Port(ep)
    }
}
pub(super) fn through_boundary(
    p: &Project,
    sid: &str,
    ep: &Endpoint,
) -> Option<(String, Selection)> {
    p.port(sid, ep)?;
    if let Some(id) = &ep.node {
        let (_, owner) = p.node(id)?;
        let child = owner.child.clone()?;
        Some((child, Selection::Boundary(ep.port.clone())))
    } else {
        let (parent, owner) = p.owner(sid)?;
        Some((
            parent.id.clone(),
            Selection::Port(Endpoint {
                node: Some(owner.id.clone()),
                port: ep.port.clone(),
            }),
        ))
    }
}
impl Designer {
    pub(in crate::ui) fn canvas_location(&self) -> Location {
        Location {
            system: self.current.clone(),
            selection: self.selected.clone(),
            view: self.canvas_session.view,
            focus: self.canvas_session.focus,
            viewport: Viewport {
                pan: self.canvas.pan,
                zoom: self.canvas.zoom,
                fit: self.canvas.fit_requested,
            },
        }
    }
    fn remember_trace(&mut self) {
        let here = self.canvas_location();
        if self.canvas_session.history.len() == 64 {
            self.canvas_session.history.remove(0);
        }
        self.canvas_session.history.push(here);
    }
    pub(in crate::ui) fn trace_visit(&mut self, sid: String, selection: Selection) {
        if !valid(self.store.project(), &sid, &selection) {
            self.status = "Trace target is no longer present; no navigation was performed.".into();
            return;
        }
        if sid == self.current && selection == self.selected {
            return;
        }
        self.remember_trace();
        let focus = self.canvas_session.focus;
        if sid != self.current {
            self.navigate(sid);
        }
        self.canvas.cancel();
        self.selected = selection;
        self.canvas_session.focus = focus;
        self.canvas_session.normalize(&self.selected);
    }
    pub(in crate::ui) fn trace_back(&mut self) {
        let Some(previous) = self.canvas_session.history.pop() else {
            return;
        };
        if !valid(self.store.project(), &previous.system, &previous.selection) {
            self.canvas.cancel();
            self.selected = Selection::None;
            self.canvas_session.focus = Focus::Off;
            self.status =
                "Previous trace target was changed or removed; trace selection cleared.".into();
            return;
        }
        if previous.system != self.current {
            self.navigate(previous.system);
        }
        self.canvas.cancel();
        self.selected = previous.selection;
        self.canvas_session.view = previous.view;
        self.canvas_session.focus = previous.focus;
        self.canvas.pan = previous.viewport.pan;
        self.canvas.zoom = previous.viewport.zoom;
        self.canvas.fit_requested = previous.viewport.fit;
        self.canvas_session.normalize(&self.selected);
    }
    pub(in crate::ui) fn show_exact(&mut self, id: &str, edit: bool) {
        let target = Selection::Edge(id.into());
        if !valid(self.store.project(), &self.current, &target) {
            self.status = "That connection no longer exists.".into();
            return;
        }
        if self.selected != target || self.canvas_session.view != View::Detail {
            self.remember_trace();
        }
        self.canvas.cancel();
        self.selected = target;
        self.canvas_session.view = View::Detail;
        self.canvas_session.normalize(&self.selected);
        if edit {
            self.dialog = Some(Dialog::Connection(ConnectionDialog::new(
                self.store.project(),
                &self.current,
                None,
                Some(id),
            )));
        }
    }
}

/// Full member rows are always available, even when a different subset is focused.
/// Return true for selections handled entirely here, rather than pretending they are nodes.
pub(in crate::ui) fn details(app: &mut Designer, ui: &mut egui::Ui) -> bool {
    let p = app.store.snapshot();
    let sid = app.current.clone();
    let selection = app.selected.clone();
    let special = matches!(selection, Selection::Port(_) | Selection::Summary(..));
    let exact_port = match &selection {
        Selection::Port(ep) => Some(ep.clone()),
        Selection::Boundary(id) => Some(Endpoint {
            node: None,
            port: id.clone(),
        }),
        _ => None,
    };
    let mut visit = None;
    let mut exact = None;
    if special {
        ui.heading(description(&p, &sid, &selection));
    }
    if let Some(ep) = &exact_port {
        if let Some(port) = p.port(&sid, ep) {
            ui.label(endpoint_name(&p, &sid, ep));
            ui.monospace(&port.id);
            ui.label(format!(
                "{} at this level",
                p.effective_direction(&sid, ep)
                    .map(|d| d.label())
                    .unwrap_or("Unknown")
            ));
            ui.monospace(
                port.contract
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "Unassigned".into()),
            );
            if let Some(reference) = &port.contract {
                if ui.button("Inspect type").clicked() {
                    if let Some(c) = p.contract(reference) {
                        app.dialog = Some(Dialog::Contract(ContractDialog::edit(c)));
                    }
                }
            }
            if let Some((next_sid, next)) = through_boundary(&p, &sid, ep) {
                if ui
                    .button(if ep.node.is_some() {
                        "Enter at this port"
                    } else {
                        "Follow in parent"
                    })
                    .clicked()
                {
                    visit = Some((next_sid, next));
                }
            } else {
                ui.label("No declared internal continuation. Choosing another output is not a proven continuation.");
            }
            if let Some(id) = &ep.node {
                if ui.button("Whole component").clicked() {
                    visit = Some((sid.clone(), Selection::Node(id.clone())));
                }
            }
        }
    }
    if let Selection::Node(id) = &selection {
        if let Some((_, node)) = p.node(id) {
            egui::ComboBox::from_id_salt("trace-port-picker")
                .selected_text("Inspect a specific port…")
                .show_ui(ui, |ui| {
                    for port in &node.ports {
                        if ui
                            .selectable_label(
                                false,
                                format!("{} · {}", port.direction.label(), port.name),
                            )
                            .clicked()
                        {
                            visit = Some((
                                sid.clone(),
                                port_selection(Endpoint {
                                    node: Some(id.clone()),
                                    port: port.id.clone(),
                                }),
                            ));
                        }
                    }
                });
        }
    }
    let mut members: Vec<_> = p
        .system(&sid)
        .into_iter()
        .flat_map(|s| &s.edges)
        .filter(|e| selection == Selection::None || matches(e, &selection, Focus::Selection))
        .collect();
    members.sort_by(|a, b| a.id.cmp(&b.id));
    let title = if matches!(selection, Selection::Summary(..)) {
        format!("{} exact members", members.len())
    } else if selection == Selection::None {
        format!("Connections at this level ({})", members.len())
    } else {
        format!("Exact connections ({})", members.len())
    };
    egui::CollapsingHeader::new(title).id_salt(("exact-connection-list",format!("{selection:?}"))).default_open(selection!=Selection::None).show(ui,|ui|{
        if matches!(selection,Selection::Summary(..)){ui.label("Visual summary only. Choose a specific member before editing or deleting.");}
        if members.is_empty(){ui.label("No connections at this selection. Unconnected does not automatically mean invalid.");}
        egui::ScrollArea::vertical().id_salt("exact-connection-rows").max_height(280.0).show(ui,|ui|{
            for edge in &members {
                ui.push_id(&edge.id,|ui|{
                    ui.monospace(&edge.id);
                    ui.label(format!("{} → {}",endpoint_name(&p,&sid,&edge.from),endpoint_name(&p,&sid,&edge.to)));
                    let contract=p.port(&sid,&edge.from).and_then(|r|r.contract.as_ref()).map(ToString::to_string).unwrap_or_default();
                    ui.monospace(contract);
                    ui.label(edge.label.as_deref().unwrap_or("Unlabeled connection"));
                    ui.horizontal_wrapped(|ui|{
                        if ui.button("Source").clicked(){visit=Some((sid.clone(),port_selection(edge.from.clone())));}
                        if ui.button("Destination").clicked(){visit=Some((sid.clone(),port_selection(edge.to.clone())));}
                        if ui.button("Show exact wire").clicked(){exact=Some((edge.id.clone(),false));}
                        if ui.button("Edit this wire").clicked(){exact=Some((edge.id.clone(),true));}
                    });
                    ui.separator();
                });
            }
        });
    });
    if let Some((sid, selection)) = visit {
        app.trace_visit(sid, selection);
    }
    if let Some((id, edit)) = exact {
        app.show_exact(&id, edit);
    }
    special
}
