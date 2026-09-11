//! A view over exact local connections, not dependency or execution inference.
use super::*;
use crate::model::Edge;
use std::collections::BTreeSet;

const ID: &str = "system-designer.focus-trace";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DirectionFilter {
    #[default]
    Both,
    Incoming,
    Outgoing,
}
#[derive(Clone, Debug, PartialEq)]
struct Location {
    system: String,
    selection: Selection,
    port: Option<Endpoint>,
}
#[derive(Clone)]
struct State {
    project: String,
    enabled: bool,
    direction: DirectionFilter,
    current: Option<Location>,
    history: Vec<Location>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            project: String::new(),
            enabled: true,
            direction: DirectionFilter::Both,
            current: None,
            history: vec![],
        }
    }
}
fn load(ctx: &egui::Context) -> State {
    ctx.data(|d| d.get_temp::<State>(egui::Id::new(ID)))
        .unwrap_or_default()
}
fn save(ctx: &egui::Context, state: State) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(ID), state));
}
fn valid(p: &Project, location: &Location) -> bool {
    if let Some(port) = &location.port {
        return p.port(&location.system, port).is_some();
    }
    match &location.selection {
        Selection::None => p.system(&location.system).is_some(),
        Selection::Node(id) => p.node(id).is_some_and(|(s, _)| s.id == location.system),
        Selection::Edge(id) => p
            .system(&location.system)
            .is_some_and(|s| s.edges.iter().any(|e| e.id == *id)),
        Selection::Boundary(id) => p.boundary(&location.system).iter().any(|r| r.id == *id),
    }
}
fn location(app: &Designer, state: &State) -> Location {
    let base = Location {
        system: app.current.clone(),
        selection: app.selected.clone(),
        port: None,
    };
    state
        .current
        .as_ref()
        .filter(|old| {
            old.system == base.system
                && old.selection == base.selection
                && valid(app.store.project(), old)
        })
        .cloned()
        .unwrap_or(base)
}
fn port_location(sid: &str, ep: &Endpoint) -> Location {
    Location {
        system: sid.into(),
        selection: ep
            .node
            .as_ref()
            .map(|n| Selection::Node(n.clone()))
            .unwrap_or_else(|| Selection::Boundary(ep.port.clone())),
        port: Some(ep.clone()),
    }
}
fn visit(
    app: &mut Designer,
    ctx: &egui::Context,
    state: &mut State,
    target: Location,
    remember: bool,
) {
    if !valid(app.store.project(), &target) {
        return;
    }
    let old = location(app, state);
    if remember && old != target {
        if state.history.len() == 64 {
            state.history.remove(0);
        }
        state.history.push(old);
    }
    if target.system != app.current {
        app.navigate(target.system.clone());
    }
    app.selected = target.selection.clone();
    state.current = Some(target);
    state.enabled = true;
    save(ctx, state.clone());
}
/// A pointer selection must override any prior port-specific trace, even on the same node.
pub(super) fn clear_port(ctx: &egui::Context) {
    let mut state = load(ctx);
    state.current = None;
    save(ctx, state);
}
pub(super) fn select_port(app: &mut Designer, ctx: &egui::Context, endpoint: &Endpoint) {
    let mut state = load(ctx);
    state.project = app.store.project().id.clone();
    let target = port_location(&app.current, endpoint);
    visit(app, ctx, &mut state, target, true);
}

#[derive(Default, Debug)]
pub(super) struct Emphasis {
    pub active: bool,
    pub edges: BTreeSet<String>,
    nodes: BTreeSet<String>,
    ports: BTreeSet<String>,
}
impl Emphasis {
    fn new(p: &Project, at: &Location, enabled: bool, direction: DirectionFilter) -> Self {
        let mut result = Self {
            active: enabled && at.selection != Selection::None,
            ..Self::default()
        };
        let Some(system) = p.system(&at.system) else {
            result.active = false;
            return result;
        };
        if let Selection::Node(id) = &at.selection {
            result.nodes.insert(id.clone());
        }
        if let Some(port) = &at.port {
            result.ports.insert(port.port.clone());
        }
        for edge in &system.edges {
            let matches = if let Some(port) = &at.port {
                edge.from == *port || edge.to == *port
            } else {
                match &at.selection {
                    Selection::None => false,
                    Selection::Edge(id) => edge.id == *id,
                    Selection::Node(id) => {
                        (direction != DirectionFilter::Incoming
                            && edge.from.node.as_ref() == Some(id))
                            || (direction != DirectionFilter::Outgoing
                                && edge.to.node.as_ref() == Some(id))
                    }
                    Selection::Boundary(id) => {
                        (edge.from.node.is_none() && edge.from.port == *id)
                            || (edge.to.node.is_none() && edge.to.port == *id)
                    }
                }
            };
            if matches {
                result.edges.insert(edge.id.clone());
                for ep in [&edge.from, &edge.to] {
                    if let Some(id) = &ep.node {
                        result.nodes.insert(id.clone());
                    }
                    result.ports.insert(ep.port.clone());
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
fn endpoint_name(p: &Project, sid: &str, ep: &Endpoint) -> String {
    let node = ep
        .node
        .as_ref()
        .and_then(|id| p.node(id))
        .map(|(_, n)| n.name.as_str())
        .unwrap_or("System boundary");
    let port = p
        .port(sid, ep)
        .map(|p| p.name.as_str())
        .unwrap_or("Missing port");
    format!("{node} / {port}")
}
fn through_boundary(p: &Project, at: &Location) -> Option<Location> {
    let ep = at.port.as_ref()?;
    if let Some(id) = &ep.node {
        let (_, node) = p.node(id)?;
        Some(port_location(
            node.child.as_deref()?,
            &Endpoint {
                node: None,
                port: ep.port.clone(),
            },
        ))
    } else {
        let (parent, owner) = p.owner(&at.system)?;
        Some(port_location(
            &parent.id,
            &Endpoint {
                node: Some(owner.id.clone()),
                port: ep.port.clone(),
            },
        ))
    }
}
pub(super) fn controls(app: &mut Designer, ui: &mut egui::Ui) -> Emphasis {
    let ctx = ui.ctx().clone();
    let p = app.store.snapshot();
    let mut state = load(&ctx);
    if state.project != p.id {
        state = State {
            project: p.id.clone(),
            ..State::default()
        };
    }
    state.history.retain(|at| valid(&p, at));
    let at = location(app, &state);
    state.current = Some(at.clone());
    let mut back = false;
    ui.horizontal_wrapped(|ui| {
        ui.strong("A · Focus & trace");
        ui.checkbox(&mut state.enabled, "Focus selection");
        ui.selectable_value(&mut state.direction, DirectionFilter::Both, "Both");
        ui.selectable_value(&mut state.direction, DirectionFilter::Incoming, "Incoming");
        ui.selectable_value(&mut state.direction, DirectionFilter::Outgoing, "Outgoing");
        if ui
            .add_enabled(!state.history.is_empty(), egui::Button::new("Back trace"))
            .clicked()
        {
            back = true;
        }
        if ui.button("Clear focus").clicked() {
            app.selected = Selection::None;
            state.current = None;
            state.history.clear();
        }
    });
    let at = location(app, &state);
    let mut emphasis = Emphasis::new(&p, &at, state.enabled, state.direction);
    if matches!(app.canvas.gesture, Some(Gesture::Wire { .. })) || app.canvas.pending.is_some() {
        emphasis.active = false;
    }
    if emphasis.active {
        let total = p.system(&at.system).map_or(0, |s| s.edges.len());
        ui.small(format!("{} / {total} connections emphasized; {} context wires muted. Exports still include the full selected scope.", emphasis.edges.len(), total.saturating_sub(emphasis.edges.len())));
    } else {
        ui.small("Select a component or wire; right-click a port for exact tracing. No layout changes, no inferred execution path.");
    }
    save(&ctx, state.clone());
    if back && let Some(previous) = state.history.pop() {
        visit(app, &ctx, &mut state, previous, false);
    }
    // Navigation/selection actions are reflected in this frame, not a stale lens.
    let now = location(app, &state);
    let mut result = Emphasis::new(app.store.project(), &now, state.enabled, state.direction);
    if matches!(app.canvas.gesture, Some(Gesture::Wire { .. })) || app.canvas.pending.is_some() {
        result.active = false;
    }
    result
}

/// Inspector content does not change the canvas rectangle or its saved positions.
pub(in crate::ui) fn details(app: &mut Designer, ui: &mut egui::Ui) {
    let ctx = ui.ctx().clone();
    let p = app.store.snapshot();
    let mut state = load(&ctx);
    if state.project != p.id {
        state = State {
            project: p.id.clone(),
            ..State::default()
        };
    }
    let at = location(app, &state);
    state.current = Some(at.clone());
    let emphasis = Emphasis::new(&p, &at, true, state.direction);
    let mut target = None;
    if at.selection != Selection::None {
        egui::CollapsingHeader::new("Trace exact connections").id_salt("focus-details").default_open(true).show(ui, |ui| {
            if let Some(port) = &at.port {
                ui.horizontal_wrapped(|ui| {
                    ui.label(endpoint_name(&p, &at.system, port));
                    if let Some(next) = through_boundary(&p, &at) {
                        if ui.button(if port.node.is_some() { "Enter at this port" } else { "Follow in parent" }).clicked() { target = Some(next); }
                    } else {
                        ui.small("No declared internal continuation; choose another port explicitly.");
                    }
                    if ui.button("Whole component").clicked() { target = Some(Location { port: None, ..at.clone() }); }
                });
            } else if let Selection::Node(id) = &at.selection {
                if let Some((_, node)) = p.node(id) {
                    egui::ComboBox::from_id_salt("focus-port-picker").selected_text("Trace a specific port…").show_ui(ui, |ui| {
                        for port in &node.ports {
                            if ui.selectable_label(false, format!("{} · {}", port.direction.label(), port.name)).clicked() {
                                target = Some(port_location(&at.system, &Endpoint { node: Some(id.clone()), port: port.id.clone() }));
                            }
                        }
                    });
                }
            } else if let Selection::Boundary(id) = &at.selection
                && ui.button("Trace this boundary port").clicked() {
                target = Some(port_location(&at.system, &Endpoint { node: None, port: id.clone() }));
            }
            egui::ScrollArea::vertical().id_salt("focus-wires").max_height(132.0).show(ui, |ui| {
                let edges: Vec<&Edge> = p.system(&at.system).map(|s| s.edges.iter().filter(|e| emphasis.edges.contains(&e.id)).collect()).unwrap_or_default();
                if edges.is_empty() { ui.label("No matching connections at this level."); }
                for edge in edges {
                    ui.push_id(&edge.id, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("Source").clicked() { target = Some(port_location(&at.system, &edge.from)); }
                            if ui.button("Destination").clicked() { target = Some(port_location(&at.system, &edge.to)); }
                            if ui.selectable_label(at.selection == Selection::Edge(edge.id.clone()), &edge.id).clicked() {
                                target = Some(Location { system: at.system.clone(), selection: Selection::Edge(edge.id.clone()), port: None });
                            }
                            if ui.button("Edit wire").clicked() {
                                app.dialog = Some(Dialog::Connection(ConnectionDialog::new(&p, &at.system, None, Some(&edge.id))));
                            }
                        });
                        ui.label(format!("{} -> {}", endpoint_name(&p, &at.system, &edge.from), endpoint_name(&p, &at.system, &edge.to)));
                        let contract = p.port(&at.system, &edge.from).and_then(|p| p.contract.as_ref()).map(ToString::to_string).unwrap_or_default();
                        ui.small(format!("{contract} · {}", edge.label.as_deref().unwrap_or("Unlabeled connection")));
                        ui.separator();
                    });
                }
            });
        });
    }
    save(&ctx, state.clone());
    if let Some(next) = target {
        visit(app, &ctx, &mut state, next, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn project() -> Project {
        parse(include_str!("../../../tests/fixtures/project.json")).expect("fixture")
    }
    #[test]
    fn focusing_a_component_never_walks_through_neighbors() {
        let p = project();
        let at = Location {
            system: "root".into(),
            selection: Selection::Node("A".into()),
            port: None,
        };
        let view = Emphasis::new(&p, &at, true, DirectionFilter::Both);
        for edge in &p.system("root").expect("root").edges {
            assert_eq!(
                view.edges.contains(&edge.id),
                edge.from.node.as_deref() == Some("A") || edge.to.node.as_deref() == Some("A")
            );
        }
    }
    #[test]
    fn incoming_and_outgoing_filters_are_directed() {
        let p = project();
        let at = Location {
            system: "root".into(),
            selection: Selection::Node("A".into()),
            port: None,
        };
        let incoming = Emphasis::new(&p, &at, true, DirectionFilter::Incoming);
        let outgoing = Emphasis::new(&p, &at, true, DirectionFilter::Outgoing);
        for edge in &p.system("root").expect("root").edges {
            assert_eq!(
                incoming.edges.contains(&edge.id),
                edge.to.node.as_deref() == Some("A")
            );
            assert_eq!(
                outgoing.edges.contains(&edge.id),
                edge.from.node.as_deref() == Some("A")
            );
        }
    }
    #[test]
    fn edge_focus_keeps_exact_endpoints_only() {
        let p = project();
        let edge = &p.system("root").expect("root").edges[0];
        let at = Location {
            system: "root".into(),
            selection: Selection::Edge(edge.id.clone()),
            port: None,
        };
        let view = Emphasis::new(&p, &at, true, DirectionFilter::Both);
        assert_eq!(view.edges.len(), 1);
        assert!(view.port(&edge.from.port) && view.port(&edge.to.port));
    }
    #[test]
    fn boundary_round_trip_preserves_physical_port() {
        let p = project();
        let node = p
            .systems
            .iter()
            .flat_map(|s| &s.nodes)
            .find(|n| n.child.is_some() && !n.ports.is_empty())
            .expect("owner");
        let sid = &p.node(&node.id).expect("node").0.id;
        let before = port_location(
            sid,
            &Endpoint {
                node: Some(node.id.clone()),
                port: node.ports[0].id.clone(),
            },
        );
        let inner = through_boundary(&p, &before).expect("child");
        assert!(inner.port.as_ref().expect("port").node.is_none());
        assert_eq!(through_boundary(&p, &inner), Some(before));
    }
    #[test]
    fn view_and_tracing_leave_serialized_project_unchanged() {
        let p = project();
        let before = serde_json::to_string(&p).expect("json");
        for s in &p.systems {
            for n in &s.nodes {
                let at = Location {
                    system: s.id.clone(),
                    selection: Selection::Node(n.id.clone()),
                    port: None,
                };
                let _ = Emphasis::new(&p, &at, true, DirectionFilter::Both);
            }
        }
        assert_eq!(serde_json::to_string(&p).expect("json"), before);
    }
    #[test]
    fn off_and_empty_selection_show_everything() {
        let p = project();
        let at = Location {
            system: "root".into(),
            selection: Selection::Node("A".into()),
            port: None,
        };
        assert!(Emphasis::new(&p, &at, false, DirectionFilter::Both).edge("unrelated"));
        assert!(
            !Emphasis::new(
                &p,
                &Location {
                    selection: Selection::None,
                    ..at
                },
                true,
                DirectionFilter::Both
            )
            .active
        );
    }
}
