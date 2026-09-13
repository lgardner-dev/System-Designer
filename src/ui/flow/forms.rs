use super::*;
use egui::{ComboBox, TextEdit, Ui};
fn step_picker(ui: &mut Ui, id: &str, value: &mut String, f: &Flow, source: bool) {
    ComboBox::from_id_salt(id)
        .selected_text(
            f.step(value)
                .map(|s| s.name.as_str())
                .unwrap_or("Choose step…"),
        )
        .show_ui(ui, |ui| {
            for s in &f.steps {
                if (source && s.kind != StepKind::Outcome) || (!source && s.kind != StepKind::Entry)
                {
                    ui.selectable_value(value, s.id.clone(), format!("{} [{}]", s.name, s.id));
                }
            }
        });
}
fn contract_picker(
    ui: &mut Ui,
    chosen: &mut Option<ContractRef>,
    p: &Project,
    new: &mut bool,
    definition: &mut Contract,
) {
    ui.horizontal(|ui| {
        ui.selectable_value(new, false, "Unassigned / existing");
        ui.selectable_value(new, true, "New contract");
    });
    if *new {
        contracts::definition_form(ui, definition, true);
    } else {
        ComboBox::from_id_salt("information_contract")
            .selected_text(
                chosen
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "Unassigned".into()),
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(chosen, None, "Unassigned");
                for c in &p.contracts {
                    ui.selectable_value(
                        chosen,
                        Some(c.reference()),
                        format!("{} · {}", c.name, c.reference()),
                    );
                }
            });
    }
}
fn data_end(ui: &mut Ui, p: &Project, owner: &str, f: &Flow, end: &mut DataEnd, source: bool) {
    ui.label(if source { "Producer" } else { "Consumer" });
    let before = end.step.clone();
    ComboBox::from_id_salt("occurrence")
        .selected_text(
            end.step
                .as_ref()
                .and_then(|s| f.step(s))
                .map(|s| s.name.as_str())
                .unwrap_or("Scope boundary"),
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut end.step, None, "Scope boundary");
            for s in &f.steps {
                ui.selectable_value(
                    &mut end.step,
                    Some(s.id.clone()),
                    format!("{} [{}]", s.name, s.id),
                );
            }
        });
    if before != end.step {
        end.port = None;
    }
    let node = if let Some(id) = &end.step {
        f.step(id).and_then(|s| s.target.as_deref())
    } else {
        Some(owner)
    };
    let ports: Vec<_> = node
        .and_then(|id| p.node(id))
        .map(|(_, n)| {
            n.ports
                .iter()
                .filter(|r| {
                    let direction = if end.step.is_none() {
                        r.direction.opposite()
                    } else {
                        r.direction
                    };
                    direction
                        == if source {
                            Direction::Out
                        } else {
                            Direction::In
                        }
                })
                .collect()
        })
        .unwrap_or_default();
    ComboBox::from_id_salt("binding")
        .selected_text(
            end.port
                .as_ref()
                .and_then(|id| ports.iter().find(|r| r.id == *id))
                .map(|r| r.name.as_str())
                .unwrap_or("No exact port binding"),
        )
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut end.port, None, "No exact port binding");
            for r in ports {
                ui.selectable_value(
                    &mut end.port,
                    Some(r.id.clone()),
                    format!("{} [{}]", r.name, r.id),
                );
            }
        });
}
fn impact(ui: &mut Ui, impact: &edit::Impact, chosen: Option<&ContractRef>) {
    ui.strong("Contract impact across both layers");
    ui.label(format!(
        "Assign {}",
        chosen
            .map(ToString::to_string)
            .unwrap_or_else(|| "Unassigned".into())
    ));
    if impact.ports.is_empty() && impact.edges.is_empty() && impact.data.is_empty() {
        ui.label("No existing contract bindings change.");
    }
    for r in &impact.ports {
        ui.label(format!(
            "Port {} · {} [{}]: {}",
            r.system,
            r.name,
            r.port,
            r.previous
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_else(|| "Unassigned".into())
        ));
    }
    for e in &impact.edges {
        ui.label(format!("Interface connection {} / {}", e.system, e.id));
    }
    for d in &impact.data {
        ui.label(format!("Control Flow {} / {} · {}", d.owner, d.id, d.name));
    }
    ui.small("Only these exact bindings are reconciled. Equal names or old types do not connect unrelated requirements.");
}
fn cancel(ui: &mut Ui) -> bool {
    ui.button("Cancel — discard draft").clicked()
}
impl Designer {
    pub(in crate::ui) fn flow_form(&mut self, ui: &mut Ui, dialog: &mut FlowDialog) -> bool {
        let p = self.store.snapshot();
        let owner = self.owner.clone();
        let f = p.behavior.get(&owner);
        let mut close = false;
        match dialog {
            FlowDialog::Start => {
                ui.heading("Start a flow");
                ui.label("Create an editable Begin -> Action -> Complete flow for this scope. Saving will use project version 2, which older readers may not support. This change is undoable.");
                if ui.button("Start flow").clicked() {
                    self.publish(
                        "Start flow (version 2)",
                        behavior::set(&p, &owner, Flow::starter()),
                    );
                    self.canvas.fit_requested = true;
                    close = self.error.is_none();
                }
            }
            FlowDialog::Step(s) => {
                ui.heading(format!("{} step", s.kind.label()));
                ui.monospace(&s.id);
                ui.label(if s.kind == StepKind::Decision {
                    "What question is answered?"
                } else {
                    "What happens?"
                });
                ui.text_edit_singleline(&mut s.name);
                ui.label("Local purpose / criteria");
                ui.add(
                    TextEdit::multiline(&mut s.purpose)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY),
                );
                if s.kind == StepKind::Call {
                    ComboBox::from_id_salt("call_target")
                        .selected_text(
                            s.target
                                .as_ref()
                                .map(|id| behavior::name(&p, id))
                                .unwrap_or("Choose component…"),
                        )
                        .show_ui(ui, |ui| {
                            for n in behavior::system(&p, &owner)
                                .into_iter()
                                .flat_map(|s| &s.nodes)
                            {
                                ui.selectable_value(
                                    &mut s.target,
                                    Some(n.id.clone()),
                                    format!("{} [{}]", n.name, n.id),
                                );
                            }
                        });
                    ui.small("This occurrence uses the existing shared component. Its action label stays independent of the component name.");
                }
                if matches!(s.kind, StepKind::Action | StepKind::Decision) {
                    ui.checkbox(
                        &mut s.information_reviewed,
                        "I reviewed this step's information needs",
                    );
                    ui.small("Changing meaning or incident information clears review. Save those edits, then review the resulting step.");
                }
                if ui.button("Save step").clicked() {
                    self.publish("Save flow step", behavior::save_step(&p, &owner, s.clone()));
                    close = self.error.is_none();
                    if close {
                        self.flow.selection.step(s.id.clone(), false);
                    }
                }
            }
            FlowDialog::Transition(t) => {
                ui.heading("Control transition");
                ui.monospace(&t.id);
                if let Some(f) = f {
                    let old_from = t.from.clone();
                    ui.label("From");
                    step_picker(ui, "from_step", &mut t.from, f, true);
                    if old_from != t.from {
                        t.outcome = None;
                    }
                    ui.label("To");
                    step_picker(ui, "to_step", &mut t.to, f, false);
                    ui.label("When does this branch apply?");
                    ui.text_edit_singleline(&mut t.condition);
                    if let Some(s) = f.step(&t.from).filter(|s| s.kind == StepKind::Call) {
                        ComboBox::from_id_salt("return_outcome")
                            .selected_text(t.outcome.as_deref().unwrap_or("Unresolved outcome"))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut t.outcome,
                                    None,
                                    "Unresolved outcome (draft)",
                                );
                                for out in s
                                    .target
                                    .as_ref()
                                    .and_then(|id| p.behavior.get(id))
                                    .into_iter()
                                    .flat_map(|f| &f.steps)
                                    .filter(|s| s.kind == StepKind::Outcome)
                                {
                                    ui.selectable_value(
                                        &mut t.outcome,
                                        Some(out.id.clone()),
                                        format!("{} [{}]", out.name, out.id),
                                    );
                                }
                            });
                    }
                    ui.small(
                        "Descriptive alternatives only. No guard evaluation or parallel execution.",
                    );
                    if ui.button("Save transition").clicked() {
                        self.publish(
                            "Save control transition",
                            behavior::save_transition(&p, &owner, t.clone()),
                        );
                        close = self.error.is_none();
                    }
                }
            }
            FlowDialog::Primitive(reason) => {
                ui.heading("Primitive / stopping criteria");
                ui.label("Explain why this scope has no useful further decomposition, or record the criteria for stopping.");
                ui.add(
                    TextEdit::multiline(reason)
                        .desired_rows(5)
                        .desired_width(f32::INFINITY),
                );
                if let Some(f) = f
                    && ui.button("Save explanation").clicked()
                {
                    let mut f = f.clone();
                    f.primitive = reason.clone();
                    self.publish("Record stopping criteria", behavior::set(&p, &owner, f));
                    close = self.error.is_none();
                }
            }
            FlowDialog::Extract(d) => return self.extraction_form(ui, d),
            FlowDialog::Information(d) => {
                ui.heading("Information requirement");
                ui.monospace(&d.link.id);
                let before = serde_json::to_string(&d.link).unwrap_or_default();
                ui.label("What information is needed?");
                ui.text_edit_singleline(&mut d.link.name);
                if let Some(f) = f {
                    ui.push_id("producer", |ui| {
                        data_end(ui, &p, &owner, f, &mut d.link.from, true)
                    });
                    ui.push_id("consumer", |ui| {
                        data_end(ui, &p, &owner, f, &mut d.link.to, false)
                    });
                }
                ui.label("Exact existing interface connection (optional)");
                ComboBox::from_id_salt("exchange_binding")
                    .selected_text(d.link.exchange.as_deref().unwrap_or("No association"))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut d.link.exchange, None, "No association");
                        for e in behavior::system(&p, &owner)
                            .into_iter()
                            .flat_map(|s| &s.edges)
                        {
                            ui.selectable_value(
                                &mut d.link.exchange,
                                Some(e.id.clone()),
                                format!(
                                    "{} [{}]",
                                    e.label.as_deref().unwrap_or("Connection"),
                                    e.id
                                ),
                            );
                        }
                    });
                ui.small("An association must match both exact port bindings. A control transition never creates a data channel.");
                contract_picker(
                    ui,
                    &mut d.link.contract,
                    &p,
                    &mut d.define_new,
                    &mut d.definition,
                );
                let mut link = d.link.clone();
                if d.define_new {
                    link.contract = Some(d.definition.reference());
                }
                let token = format!(
                    "{}:{}:{}:{}",
                    self.store.generation,
                    serde_json::to_string(&link).unwrap_or_default(),
                    d.define_new,
                    serde_json::to_string(&d.definition).unwrap_or_default()
                );
                if before != serde_json::to_string(&d.link).unwrap_or_default()
                    || d.reviewed.as_ref() != Some(&token)
                {
                    d.reviewed = None;
                }
                let candidate = behavior::information_candidate(
                    &p,
                    &owner,
                    link.clone(),
                    d.define_new.then(|| d.definition.clone()),
                );
                match &candidate {
                    Ok((_, i)) => impact(ui, i, link.contract.as_ref()),
                    Err(e) => {
                        ui.colored_label(egui::Color32::LIGHT_RED, e.to_string());
                    }
                }
                if ui
                    .add_enabled(candidate.is_ok(), egui::Button::new("Review complete edit"))
                    .clicked()
                {
                    d.reviewed = Some(token.clone());
                }
                if ui
                    .add_enabled(
                        d.reviewed.as_ref() == Some(&token),
                        egui::Button::new("Apply information requirement"),
                    )
                    .clicked()
                {
                    self.publish(
                        "Edit information requirement",
                        behavior::information_candidate(
                            self.store.project(),
                            &owner,
                            link,
                            d.define_new.then(|| d.definition.clone()),
                        )
                        .map(|(q, _)| q),
                    );
                    close = self.error.is_none();
                }
            }
            FlowDialog::Port(d) => {
                ui.heading("Refine public contract");
                ui.monospace(&d.id);
                contract_picker(ui, &mut d.chosen, &p, &mut d.define_new, &mut d.definition);
                let chosen = if d.define_new {
                    Some(d.definition.reference())
                } else {
                    d.chosen.clone()
                };
                let i = edit::binding_impact(
                    &p,
                    std::slice::from_ref(&d.id),
                    &[],
                    chosen.as_ref(),
                    None,
                );
                impact(ui, &i, chosen.as_ref());
                let token = format!(
                    "{}:{:?}:{}:{}",
                    self.store.generation,
                    chosen,
                    d.define_new,
                    serde_json::to_string(&d.definition).unwrap_or_default()
                );
                if d.reviewed.as_ref() != Some(&token) {
                    d.reviewed = None;
                }
                let candidate = edit::refine_port(
                    &p,
                    &d.id,
                    chosen.clone(),
                    d.define_new.then(|| d.definition.clone()),
                );
                if let Err(e) = &candidate {
                    ui.colored_label(egui::Color32::LIGHT_RED, e.to_string());
                }
                if ui
                    .add_enabled(candidate.is_ok(), egui::Button::new("Review complete edit"))
                    .clicked()
                {
                    d.reviewed = Some(token.clone());
                }
                if ui
                    .add_enabled(
                        d.reviewed.as_ref() == Some(&token),
                        egui::Button::new("Apply contract refinement"),
                    )
                    .clicked()
                {
                    self.publish(
                        "Refine public contract",
                        edit::refine_port(
                            self.store.project(),
                            &d.id,
                            chosen,
                            d.define_new.then(|| d.definition.clone()),
                        ),
                    );
                    close = self.error.is_none();
                }
            }
            FlowDialog::Delete(selection) => {
                ui.heading("Delete flow selection");
                if let Some(f) = f {
                    for id in &selection.steps {
                        ui.label(format!(
                            "Step {} [{}]",
                            f.step(id).map(|s| s.name.as_str()).unwrap_or("Missing"),
                            id
                        ));
                        for t in &f.transitions {
                            if t.from == *id || t.to == *id {
                                ui.label(format!("Remove transition {}", t.id));
                            }
                        }
                        for d in &f.data {
                            if d.from.step.as_ref() == Some(id) || d.to.step.as_ref() == Some(id) {
                                ui.label(format!("Remove information {} [{}]", d.name, d.id));
                            }
                        }
                        for caller in behavior::outcome_users(&p, &owner, id) {
                            ui.colored_label(
                                egui::Color32::LIGHT_RED,
                                format!("Blocked by caller {caller}"),
                            );
                        }
                    }
                    for id in [&selection.transition, &selection.data]
                        .into_iter()
                        .flatten()
                    {
                        ui.label(format!("Remove {id}"));
                    }
                    ui.label("Removing an occurrence preserves its shared component definition. This edit can be undone.");
                    if ui.button("Delete listed items").clicked() {
                        let result = (|| {
                            let mut q = (*p).clone();
                            for id in &selection.steps {
                                q = behavior::delete_step(&q, &owner, id)?;
                            }
                            if let Some(id) = &selection.transition {
                                q = behavior::delete_transition(&q, &owner, id)?;
                            }
                            if let Some(id) = &selection.data {
                                q = behavior::delete_data(&q, &owner, id)?;
                            }
                            Ok(q)
                        })();
                        self.publish("Delete flow selection", result);
                        close = self.error.is_none();
                    }
                }
            }
            FlowDialog::Uses(id) => {
                ui.heading(format!("Uses of {}", behavior::name(&p, id)));
                let mut count = 0;
                for (scope, f) in &p.behavior {
                    for s in &f.steps {
                        if s.target.as_ref() == Some(id) {
                            count += 1;
                            if ui
                                .button(format!(
                                    "{} / {} [{}]",
                                    behavior::name(&p, scope),
                                    s.name,
                                    s.id
                                ))
                                .clicked()
                            {
                                self.go_scope(scope.clone(), Layer::Flow);
                                self.flow.selection.step(s.id.clone(), false);
                                close = true;
                            }
                        }
                    }
                }
                if count == 0 {
                    ui.label("No call occurrences reference this component yet.");
                }
                ui.label(
                    "Choose the exact occurrence. Repeated calls share one component definition.",
                );
            }
        }
        ui.separator();
        close || cancel(ui)
    }
    fn extraction_form(&mut self, ui: &mut Ui, d: &mut ExtractDialog) -> bool {
        let p = self.store.snapshot();
        let owner = self.owner.clone();
        let Some(f) = p.behavior.get(&owner) else {
            return true;
        };
        ui.heading("Extract a responsibility");
        ui.small("Structural candidates need human purpose and information review. Extractability does not establish single responsibility.");
        let before = (d.members.clone(), d.name.clone(), d.purpose.clone());
        egui::CollapsingHeader::new("1 · Choose the work").default_open(true).show(ui,|ui| {
            if !d.suggestions.is_empty() {
                ui.label("Structural candidates · stable size/ID ordering · at most 24 suggestions / 80 steps");
                for (i,r) in d.suggestions.iter().enumerate() {
                    if ui.button(format!("Candidate {}: {}",i+1,r.members.iter().filter_map(|id|f.step(id)).map(|s|s.name.as_str()).collect::<Vec<_>>().join(", "))).clicked() {d.members = r.members.clone();}
                }
            }
            egui::ScrollArea::vertical().id_salt("region_members").max_height(180.0).show(ui,|ui| {
                for s in &f.steps {
                    let mut included = d.members.contains(&s.id);
                    if ui.checkbox(&mut included,format!("{} · {} [{}]",s.name,s.kind.label(),s.id)).changed() {
                        if included {d.members.insert(s.id.clone());} else {d.members.remove(&s.id);}
                    }
                }
            });
        });
        self.flow.selection = FlowSelection {
            steps: d.members.clone(),
            ..Default::default()
        };
        egui::CollapsingHeader::new("2 · Name the responsibility")
            .default_open(true)
            .show(ui, |ui| {
                ui.label("Canonical component name");
                ui.text_edit_singleline(&mut d.name);
                ui.label("What is this component responsible for?");
                ui.add(
                    TextEdit::multiline(&mut d.purpose)
                        .desired_rows(2)
                        .desired_width(f32::INFINITY),
                );
            });
        if before != (d.members.clone(), d.name.clone(), d.purpose.clone()) {
            d.plan = None;
            d.problem = None;
        }
        if ui.button("Preview boundary").clicked() {
            match behavior::preview(&p, &owner, &d.members, &d.name, &d.purpose) {
                Ok(plan) => {
                    d.plan = Some(plan);
                    d.problem = None;
                }
                Err(e) => {
                    d.plan = None;
                    d.problem = Some(e.to_string());
                }
            }
        }
        egui::CollapsingHeader::new("3 · Review the boundary").default_open(true).show(ui,|ui| {
            if let Some(error) = &d.problem {ui.colored_label(egui::Color32::LIGHT_RED,error);}
            if let Some(plan) = &d.plan {
                ui.label(format!("Parent call {} -> shared component {}",plan.call,plan.component));
                ui.label(format!("Child contains {} selected steps; entry {}",plan.region.members.len(),plan.region.entry));
                for t in &plan.region.exits {ui.label(format!("Distinct outcome: {} -> {} [{}]",if t.condition.is_empty(){"Continue"}else{&t.condition},f.step(&t.to).map(|s|s.name.as_str()).unwrap_or(&t.to),t.id));}
                if plan.requirements.is_empty() {ui.label("No information requirements declared yet");}
                for r in &plan.requirements {ui.label(format!("{} · {} · {}",r.direction.label(),r.name,r.contract.as_ref().map(ToString::to_string).unwrap_or_else(||"Unassigned".into())));}
                ui.label(format!("{} local information uses still need human review",plan.unreviewed));
                ui.small("Only declared requirements are projected. Side effects, aliasing and conditional availability are not inferred.");
            } else {ui.label("Preview to see entry, outgoing alternatives, declared requirements and blockers.");}
        });
        let mut close = false;
        if ui
            .add_enabled(d.plan.is_some(), egui::Button::new("Create component"))
            .clicked()
            && let Some(plan) = &d.plan
        {
            self.accept_extraction(plan);
            close = self.error.is_none();
        }
        ui.separator();
        close || cancel(ui)
    }
}
