use super::*;
use crate::edit;
use crate::ui::dialogs::{Action, Actions};
use egui::{ComboBox, DragValue, TextEdit, Ui};
pub(super) struct ConnectionDialog {
    pub id: Option<String>,
    pub from: Option<Endpoint>,
    pub to: Option<Endpoint>,
    pub label: String,
    pub selected: Option<ContractRef>,
    pub define_new: bool,
    pub draft: Contract,
    pub consent: bool,
}
impl ConnectionDialog {
    pub fn new(
        p: &Project,
        sid: &str,
        endpoints: Option<(Endpoint, Endpoint)>,
        editing: Option<&str>,
    ) -> Self {
        let edge = editing.and_then(|id| p.system(sid)?.edges.iter().find(|e| e.id == id));
        let (from, to) = if let Some((a, b)) = endpoints {
            (Some(a), Some(b))
        } else if let Some(e) = edge {
            (Some(e.from.clone()), Some(e.to.clone()))
        } else {
            (
                p.endpoints(sid, Direction::Out)
                    .first()
                    .map(|r| r.endpoint.clone()),
                p.endpoints(sid, Direction::In)
                    .first()
                    .map(|r| r.endpoint.clone()),
            )
        };
        Self {
            id: edge.map(|e| e.id.clone()),
            from,
            to,
            label: edge.and_then(|e| e.label.clone()).unwrap_or_default(),
            selected: edge
                .and_then(|e| p.port(sid, &e.from))
                .and_then(|r| r.contract.clone()),
            define_new: false,
            draft: Contract::draft(format!("Contract.{}", uuid::Uuid::new_v4())),
            consent: false,
        }
    }
}
pub(super) struct ContractDialog {
    pub draft: Contract,
    pub editing: Option<ContractRef>,
    pub consent: bool,
}
impl ContractDialog {
    pub fn new() -> Self {
        Self {
            draft: Contract::draft(format!("Contract.{}", uuid::Uuid::new_v4())),
            editing: None,
            consent: false,
        }
    }
    pub fn edit(c: &Contract) -> Self {
        Self {
            draft: c.clone(),
            editing: Some(c.reference()),
            consent: false,
        }
    }
}
fn endpoint_picker(
    ui: &mut Ui,
    id: &str,
    label: &str,
    value: &mut Option<Endpoint>,
    choices: &[AvailablePort],
) {
    ui.label(label);
    let name = value
        .as_ref()
        .and_then(|e| choices.iter().find(|x| x.endpoint == *e))
        .map(|r| r.name.clone())
        .unwrap_or_else(|| "Choose port…".into());
    ComboBox::from_id_salt(id)
        .selected_text(name)
        .width(500.0)
        .show_ui(ui, |ui| {
            for r in choices {
                ui.selectable_value(value, Some(r.endpoint.clone()), &r.name);
            }
        });
}
pub(super) fn definition_form(ui: &mut Ui, c: &mut Contract, identity_editable: bool) {
    ui.add_enabled_ui(identity_editable, |ui| {
        ui.horizontal(|ui| {
            ui.label("Stable ID");
            ui.text_edit_singleline(&mut c.id);
            ui.label("Version");
            ui.add(
                DragValue::new(&mut c.version)
                    .range(1..=MAX_SAFE_INTEGER)
                    .speed(1),
            );
        });
    });
    ui.label("Human name");
    ui.text_edit_singleline(&mut c.name);
    ui.label("Purpose / meaning");
    ui.add(
        TextEdit::multiline(&mut c.purpose)
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    ui.separator();
    ui.strong("Structured data definition");
    shape_form(ui, &mut c.definition);
}
/// Small structural shape language, not JSON Schema or a runtime validation engine.
fn shape_form(ui: &mut Ui, shape: &mut Shape) {
    let old = shape.label();
    let mut kind = old.to_owned();
    ComboBox::from_id_salt("shape_type")
        .selected_text(&kind)
        .show_ui(ui, |ui| {
            for t in [
                "string", "integer", "number", "boolean", "enum", "object", "array",
            ] {
                ui.selectable_value(&mut kind, t.to_owned(), t);
            }
        });
    if kind != old {
        *shape = Shape::of_type(&kind);
    }
    match shape {
        Shape::Object { fields } => {
            let mut remove = None;
            let mut swap = None;
            let count = fields.len();
            for (i, field) in fields.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Field");
                            ui.text_edit_singleline(&mut field.name);
                            ui.checkbox(&mut field.required, "Required");
                            if ui.add_enabled(i > 0, egui::Button::new("↑")).clicked() {
                                swap = Some((i, i - 1));
                            }
                            if ui
                                .add_enabled(i + 1 < count, egui::Button::new("↓"))
                                .clicked()
                            {
                                swap = Some((i, i + 1));
                            }
                            if ui.small_button("Remove").clicked() {
                                remove = Some(i);
                            }
                        });
                        let mut description = field.description.clone().unwrap_or_default();
                        if ui
                            .add(
                                TextEdit::singleline(&mut description)
                                    .hint_text("Optional field description"),
                            )
                            .changed()
                        {
                            field.description = Some(description);
                        }
                        ui.indent("nested_schema", |ui| shape_form(ui, &mut field.schema));
                    });
                });
            }
            if let Some(i) = remove {
                fields.remove(i);
            } else if let Some((a, b)) = swap {
                fields.swap(a, b);
            }
            if ui.button("+ Field").clicked() {
                fields.push(Field {
                    name: format!("field_{}", fields.len() + 1),
                    required: true,
                    schema: Shape::String,
                    description: None,
                });
            }
        }
        Shape::Array { items } => {
            ui.label("Item shape");
            ui.indent("array_item", |ui| shape_form(ui, items.as_mut()));
        }
        Shape::Enum { values } => {
            let mut remove = None;
            for (i, value) in values.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(value);
                        if ui.small_button("Remove").clicked() {
                            remove = Some(i);
                        }
                    });
                });
            }
            if let Some(i) = remove {
                values.remove(i);
            }
            if ui.button("+ Enum value").clicked() {
                values.push(format!("value_{}", values.len() + 1));
            }
        }
        _ => {}
    }
}
impl Designer {
    pub(super) fn connection_form(
        &mut self,
        ui: &mut Ui,
        d: &mut ConnectionDialog,
        actions: &mut Actions,
    ) -> bool {
        let p = self.store.snapshot();
        let sid = self.current.clone();
        ui.heading(if d.id.is_some() {
            "Change connection"
        } else {
            "Choose the connection contract"
        });
        ui.label("No connection is committed until you choose or define a contract and apply. Cancel leaves the project unchanged.");
        let before = (
            d.from.clone(),
            d.to.clone(),
            d.selected.clone(),
            d.define_new,
            d.draft.reference(),
        );
        endpoint_picker(
            ui,
            "from",
            "From — producer",
            &mut d.from,
            &p.endpoints(&sid, Direction::Out),
        );
        endpoint_picker(
            ui,
            "to",
            "To — consumer",
            &mut d.to,
            &p.endpoints(&sid, Direction::In),
        );
        ui.label("Optional edge / branch label");
        ui.text_edit_singleline(&mut d.label);
        ui.separator();
        ui.horizontal(|ui| {
            ui.selectable_value(&mut d.define_new, false, "Select existing");
            ui.selectable_value(&mut d.define_new, true, "Define new");
        });
        if d.define_new {
            definition_form(ui, &mut d.draft, true);
        } else {
            ComboBox::from_id_salt("connection_contract")
                .selected_text(
                    d.selected
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| "Choose a contract…".into()),
                )
                .width(500.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut d.selected, None, "Choose a contract…");
                    for c in &p.contracts {
                        let r = c.reference();
                        ui.selectable_value(
                            &mut d.selected,
                            Some(r.clone()),
                            format!("{} — {r}", c.name),
                        );
                    }
                });
            if p.contracts.is_empty() {
                ui.label("The catalog is empty. Use Define new to describe the first payload.");
            }
        }
        let after = (
            d.from.clone(),
            d.to.clone(),
            d.selected.clone(),
            d.define_new,
            d.draft.reference(),
        );
        if before != after {
            d.consent = false;
        }
        let chosen = if d.define_new {
            Some(d.draft.reference())
        } else {
            d.selected.clone()
        };
        let impact = if let (Some(a), Some(b), Some(c)) = (&d.from, &d.to, &chosen) {
            edit::connection_impact(&p, &sid, a, b, c, d.id.as_deref()).ok()
        } else {
            None
        };
        if let Some(impact) = &impact {
            if !impact.ports.is_empty() {
                ui.separator();
                ui.strong(format!("Port bindings to change: {}", impact.ports.len()));
                for r in &impact.ports {
                    ui.label(format!(
                        "{} [{}] · {} → {}",
                        r.name,
                        r.system,
                        r.previous
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "Unassigned".into()),
                        chosen.as_ref().map(ToString::to_string).unwrap_or_default()
                    ));
                }
            }
            if !impact.edges.is_empty() {
                ui.strong("Other connections that share these bindings");
                for e in &impact.edges {
                    ui.monospace(format!("{} · {}", e.system, e.id));
                }
            }
            for d in &impact.data {
                ui.label(format!("Control Flow {} · {} [{}]", d.owner, d.name, d.id));
            }
            if impact.requires_consent {
                ui.checkbox(
                    &mut d.consent,
                    "I confirm all listed port, connection and information changes",
                );
            }
        }
        let ready = chosen.is_some()
            && d.from.is_some()
            && d.to.is_some()
            && impact
                .as_ref()
                .is_some_and(|x| !x.requires_consent || d.consent);
        let mut close = false;
        ui.separator();
        {
            if actions.take(Action::Primary, ready)
                && let (Some(from), Some(to), Some(contract)) =
                    (d.from.clone(), d.to.clone(), chosen.clone())
            {
                let request = edit::Connection {
                    id: d.id.clone(),
                    from,
                    to,
                    label: d.label.clone(),
                    contract,
                    new_contract: if d.define_new {
                        Some(d.draft.clone())
                    } else {
                        None
                    },
                    consent: d.consent,
                };
                self.publish("Set connection contract", edit::connect(&p, &sid, &request));
                close = self.error.is_none();
            }
            if actions.cancel {
                close = true;
            }
        }
        close
    }
    pub(super) fn contract_form(
        &mut self,
        ui: &mut Ui,
        d: &mut ContractDialog,
        actions: &mut Actions,
    ) -> bool {
        ui.heading(if d.editing.is_some() {
            "Edit shared contract definition"
        } else {
            "Define contract"
        });
        definition_form(ui, &mut d.draft, d.editing.is_none());
        if let Some(reference) = &d.editing {
            let users = edit::contract_users(self.store.project(), reference);
            if !users.is_empty() {
                ui.separator();
                ui.strong(format!(
                    "This exact definition is used by {} port(s)",
                    users.len()
                ));
                for name in users {
                    ui.label(name);
                }
                ui.checkbox(
                    &mut d.consent,
                    "Update this shared definition for every existing reference",
                );
            }
            if ui.button("New version instead").clicked() {
                d.draft.version = self
                    .store
                    .project()
                    .contracts
                    .iter()
                    .filter(|c| c.id == d.draft.id)
                    .map(|c| c.version)
                    .max()
                    .unwrap_or(0)
                    + 1;
                d.editing = None;
                d.consent = false;
            }
        }
        let mut close = false;
        ui.separator();
        {
            if actions.take(Action::Primary, true) {
                let q = edit::save_contract(
                    self.store.project(),
                    d.draft.clone(),
                    d.editing.as_ref(),
                    d.consent,
                );
                self.publish("Save contract definition", q);
                close = self.error.is_none();
            }
            if actions.cancel {
                close = true;
            }
        }
        close
    }
}
