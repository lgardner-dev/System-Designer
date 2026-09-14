use super::{Action, Actions};
use crate::ui::*;
use crate::{
    edit::{self, LayoutScope},
    model::{Position, Result},
};
use std::collections::BTreeMap;

pub(in crate::ui) struct PositionForm {
    pub scope: LayoutScope,
    pub positions: BTreeMap<String, Position>,
}
impl Designer {
    fn selected_positions(&self) -> Option<PositionForm> {
        let p = self.store.project();
        match self.layer {
            Layer::Flow => {
                let f = p.behavior.get(&self.owner)?;
                let positions =
                    crate::behavior::effective_positions(f, p.flow_layout.get(&self.owner))
                        .into_iter()
                        .filter(|(id, _)| self.flow.selection.steps.contains(id))
                        .collect();
                Some(PositionForm {
                    scope: LayoutScope::Flow(self.owner.clone()),
                    positions,
                })
            }
            Layer::Interfaces => {
                let Selection::Node(id) = &self.selected else {
                    return None;
                };
                let mut positions = auto_layout(p, &self.current);
                if let Some(saved) = p.layout.get(&self.current) {
                    positions.extend(saved.clone());
                }
                Some(PositionForm {
                    scope: LayoutScope::Interfaces(self.current.clone()),
                    positions: BTreeMap::from([(id.clone(), *positions.get(id)?)]),
                })
            }
        }
    }
    pub(in crate::ui) fn position_dialog(&mut self) {
        if let Some(form) = self
            .selected_positions()
            .filter(|f| !f.positions.is_empty())
        {
            self.dialog = Some(Dialog::Position(form));
        }
    }
    pub(in crate::ui) fn nudge_selection(&mut self, delta: Position) {
        if let Some(mut form) = self
            .selected_positions()
            .filter(|f| !f.positions.is_empty())
        {
            for pos in form.positions.values_mut() {
                pos.x += delta.x;
                pos.y += delta.y;
            }
            self.publish(
                "Move selection",
                edit::move_elements(self.store.project(), &form.scope, form.positions),
            );
        }
    }
    pub(super) fn position_form(
        &mut self,
        ui: &mut egui::Ui,
        form: &mut PositionForm,
        actions: &mut Actions,
    ) -> bool {
        ui.label("World coordinates. Arrow keys move selected elements 1 unit; Shift+Arrow moves 10. Negative positions use project version 3.");
        for (id, pos) in &mut form.positions {
            ui.push_id(id, |ui| {
                ui.label(id);
                ui.horizontal(|ui| {
                    ui.label("X");
                    ui.add(egui::DragValue::new(&mut pos.x).speed(1.0));
                    ui.label("Y");
                    ui.add(egui::DragValue::new(&mut pos.y).speed(1.0));
                });
            });
        }
        if actions.take(Action::Primary, true) {
            let result: Result<_> =
                edit::move_elements(self.store.project(), &form.scope, form.positions.clone());
            self.publish("Move selection", result);
            return self.error.is_none();
        }
        actions.cancel
    }
}
