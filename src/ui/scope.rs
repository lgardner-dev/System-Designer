//! Semantic behavior owners are distinct from their optional interface systems.
use super::*;
use crate::behavior::{self, ROOT};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Layer {
    Flow,
    Interfaces,
}
#[derive(Clone)]
pub(super) struct Location {
    owner: String,
    layer: Layer,
    selection: Selection,
    flow_selection: flow::FlowSelection,
    viewport: canvas::Viewport,
    view: canvas::View,
    focus: canvas::Focus,
}
#[derive(Default)]
pub(super) struct Navigation {
    views: BTreeMap<(String, Layer), Location>,
    history: Vec<Location>,
}
impl Designer {
    pub(super) fn interface_system(&self) -> Option<&System> {
        behavior::system(self.store.project(), &self.owner)
    }
    fn location(&self) -> Location {
        Location {
            owner: self.owner.clone(),
            layer: self.layer,
            selection: self.selected.clone(),
            flow_selection: self.flow.selection.clone(),
            viewport: canvas::Viewport {
                pan: self.canvas.pan,
                zoom: self.canvas.zoom,
                fit: self.canvas.fit_requested,
            },
            view: self.canvas_session.view,
            focus: self.canvas_session.focus,
        }
    }
    fn restore_location(&mut self, location: Location) {
        self.owner = location.owner;
        self.layer = location.layer;
        self.current = behavior::system(self.store.project(), &self.owner)
            .map(|s| s.id.clone())
            .unwrap_or_default();
        self.selected = location.selection;
        self.flow.selection = location.flow_selection;
        self.flow.cancel();
        self.canvas = CanvasState::default();
        self.canvas.pan = location.viewport.pan;
        self.canvas.zoom = location.viewport.zoom;
        self.canvas.fit_requested = location.viewport.fit;
        self.canvas_session.view = location.view;
        self.canvas_session.focus = location.focus;
    }
    pub(super) fn go_scope(&mut self, owner: String, layer: Layer) {
        if !behavior::exists(self.store.project(), &owner)
            || (owner == self.owner && layer == self.layer)
        {
            return;
        }
        let previous = self.location();
        self.navigation
            .views
            .insert((self.owner.clone(), self.layer), previous.clone());
        self.navigation.history.push(previous);
        let next = self
            .navigation
            .views
            .get(&(owner.clone(), layer))
            .cloned()
            .unwrap_or(Location {
                owner,
                layer,
                selection: Selection::None,
                flow_selection: flow::FlowSelection::default(),
                viewport: canvas::Viewport {
                    pan: egui::Vec2::ZERO,
                    zoom: 1.0,
                    fit: true,
                },
                view: canvas::View::Detail,
                focus: canvas::Focus::Off,
            });
        self.restore_location(next);
        for a in self.store.project().ancestors(&self.current) {
            if let Some(id) = a["node"].as_str() {
                self.collapsed.remove(id);
            }
        }
        self.normalize_selection();
    }
    pub(super) fn back_scope(&mut self) {
        if let Some(previous) = self.navigation.history.pop() {
            self.navigation
                .views
                .insert((self.owner.clone(), self.layer), self.location());
            self.restore_location(previous);
            self.normalize_selection();
        }
    }
    pub(super) fn can_go_back(&self) -> bool {
        !self.navigation.history.is_empty()
    }
    pub(super) fn normalize_scope(&mut self) {
        let p = self.store.project();
        self.navigation.views.retain(|(owner, _), location| {
            if !behavior::exists(p, owner) {
                return false;
            }
            normalize_location(p, location);
            true
        });
        self.navigation.history.retain_mut(|location| {
            if !behavior::exists(p, &location.owner) {
                return false;
            }
            normalize_location(p, location);
            true
        });
        self.canvas_session
            .viewports
            .retain(|sid, _| p.system(sid).is_some());
        self.canvas_session.history.retain(|l| {
            p.system(&l.system).is_some() && canvas::selection_valid(p, &l.system, &l.selection)
        });
        if !behavior::exists(p, &self.owner) {
            self.owner = ROOT.into();
            self.selected = Selection::None;
            self.flow = flow::FlowState::default();
            self.canvas = CanvasState::default();
            self.status = "The previous component was removed; returned to root.".into();
        }
        self.current = behavior::system(p, &self.owner)
            .map(|s| s.id.clone())
            .unwrap_or_default();
        self.flow.selection.normalize(p.behavior.get(&self.owner));
    }
    pub(super) fn show_component_interfaces(&mut self, component: &str) {
        self.go_scope(self.owner.clone(), Layer::Interfaces);
        self.selected = Selection::Node(component.into());
        self.canvas_session.focus = canvas::Focus::Selection;
    }
}
fn normalize_location(p: &Project, l: &mut Location) {
    let sid = behavior::system(p, &l.owner)
        .map(|s| s.id.as_str())
        .unwrap_or("");
    if !canvas::selection_valid(p, sid, &l.selection) {
        l.selection = Selection::None;
        l.focus = canvas::Focus::Off;
    }
    l.flow_selection.normalize(p.behavior.get(&l.owner));
}
