use super::*;
use egui::Ui;
impl Designer {
    pub(in crate::ui) fn flow_inspector(&mut self, ui: &mut Ui) {
        let p = self.store.snapshot();
        let owner = self.owner.clone();
        ui.heading(behavior::name(&p, &owner));
        ui.small("Describe behavior -> Extract responsibilities -> Define information");
        ui.small("Revisit any step. Purpose and information review stay human decisions.");
        ui.separator();
        let Some(f) = p.behavior.get(&owner) else {
            ui.label("Control flow not specified");
            if ui.button("Start a flow").clicked() {
                self.dialog = Some(Dialog::Flow(FlowDialog::Start));
            }
            return;
        };
        for id in self.flow.selection.steps.clone() {
            if let Some(s) = f.step(&id) {
                ui.heading(&s.name);
                ui.collapsing("Identity", |ui| {
                    ui.monospace(&s.id);
                });
                if let Some(target) = &s.target {
                    ui.label(format!("Shared component: {}", behavior::name(&p, target)));
                    if ui.button("Show in Interfaces").clicked() {
                        self.show_component_interfaces(target);
                    }
                    if ui.button("Enter component").clicked() {
                        self.go_scope(target.clone(), Layer::Flow);
                    }
                    if ui.button("All call sites…").clicked() {
                        self.dialog = Some(Dialog::Flow(FlowDialog::Uses(target.clone())));
                    }
                } else {
                    ui.label(format!("Local to {}", behavior::name(&p, &owner)));
                }
                ui.label(&s.purpose);
                if matches!(s.kind, StepKind::Action | StepKind::Decision) {
                    ui.label(if s.information_reviewed {
                        "Information use reviewed by human"
                    } else {
                        "Review needed: What information is needed?"
                    });
                }
                if ui.button("Move / position…").clicked() {
                    self.position_dialog();
                }
                if ui.button("Edit step…").clicked() {
                    self.dialog = Some(Dialog::Flow(FlowDialog::Step(s.clone())));
                }
                for t in f
                    .transitions
                    .iter()
                    .filter(|t| t.from == s.id || t.to == s.id)
                {
                    if ui
                        .small_button(format!(
                            "{} -> {} [{}]",
                            f.step(&t.from).map(|s| s.name.as_str()).unwrap_or(&t.from),
                            f.step(&t.to).map(|s| s.name.as_str()).unwrap_or(&t.to),
                            t.id
                        ))
                        .clicked()
                    {
                        self.dialog = Some(Dialog::Flow(FlowDialog::Transition(t.clone())));
                    }
                }
            }
        }
        if let Some(t) = self
            .flow
            .selection
            .transition
            .as_ref()
            .and_then(|id| f.transitions.iter().find(|t| t.id == *id))
        {
            ui.heading("Transition");
            ui.monospace(&t.id);
            ui.label(&t.condition);
            if ui.button("Edit transition…").clicked() {
                self.dialog = Some(Dialog::Flow(FlowDialog::Transition(t.clone())));
            }
        }
        if self.flow.selection != FlowSelection::default()
            && ui.button("Delete selection…").clicked()
        {
            self.ask_delete_flow();
        }
        ui.separator();
        ui.strong("Information requirements");
        if f.data.is_empty() {
            ui.label("No information requirements declared yet");
        }
        for d in &f.data {
            ui.push_id(&d.id, |ui| {
                ui.group(|ui| {
                    if ui
                        .selectable_label(self.flow.selection.data.as_ref() == Some(&d.id), &d.name)
                        .clicked()
                    {
                        self.flow.selection = FlowSelection {
                            data: Some(d.id.clone()),
                            ..Default::default()
                        };
                    }
                    let end = |e: &DataEnd| {
                        e.step
                            .as_ref()
                            .and_then(|id| f.step(id))
                            .map(|s| s.name.as_str())
                            .unwrap_or("Scope boundary")
                    };
                    ui.small(format!("{} -> {}", end(&d.from), end(&d.to)));
                    ui.monospace(
                        d.contract
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "Unassigned".into()),
                    );
                    if ui.small_button("Edit requirement…").clicked() {
                        self.information_dialog(Some(d));
                    }
                    if let Some(edge) = &d.exchange {
                        if ui.small_button(format!("Show exact wire {edge}")).clicked() {
                            self.go_scope(owner.clone(), Layer::Interfaces);
                            self.selected = Selection::Edge(edge.clone());
                            self.canvas_session.focus = canvas::Focus::Selection;
                        }
                    } else {
                        ui.small("No exact interface-wire association");
                    }
                });
            });
        }
        if ui.button("+ Information requirement").clicked() {
            self.information_dialog(None);
        }
        ui.separator();
        ui.label(if f.primitive.is_empty() {
            "No stopping explanation recorded"
        } else {
            &f.primitive
        });
        if ui.button("Primitive / stopping criteria…").clicked() {
            self.dialog = Some(Dialog::Flow(FlowDialog::Primitive(f.primitive.clone())));
        }
        let issues = behavior::issues(&p, &owner);
        egui::CollapsingHeader::new(format!("{} draft issues", issues.len()))
            .default_open(true)
            .show(ui, |ui| {
                for issue in issues {
                    ui.label(issue);
                }
            });
        ui.small(format!(
            "Normalized graph complexity: {}",
            behavior::complexity(f)
                .map(|n| n.to_string())
                .unwrap_or_else(|| "incomplete graph".into())
        ));
        ui.small("A graph metric is not a feasible execution count or correctness proof.");
    }
    pub(in crate::ui) fn leaf_inspector(&mut self, ui: &mut Ui) {
        let p = self.store.snapshot();
        if let Some((parent, n)) = p.node(&self.owner) {
            ui.heading(&n.name);
            ui.monospace(&n.id);
            ui.label(&n.purpose);
            if ui.button("Edit component").clicked() {
                self.dialog = Some(Dialog::Node(n.clone()));
            }
            if ui.button("Show uses in Control Flow").clicked() {
                self.dialog = Some(Dialog::Flow(FlowDialog::Uses(n.id.clone())));
            }
            if ui.button("Show in parent Interfaces").clicked() {
                self.navigate(parent.id.clone());
                self.selected = Selection::Node(n.id.clone());
            }
            for r in &n.ports {
                ui.group(|ui| {
                    ui.label(format!("{} · {}", r.direction.label(), r.name));
                    ui.monospace(&r.id);
                    ui.label(
                        r.contract
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "Unassigned".into()),
                    );
                    if ui.button("Refine contract…").clicked() {
                        self.open_port_refinement(&r.id);
                    }
                });
            }
            ui.horizontal(|ui| {
                for direction in [Direction::In, Direction::Out] {
                    if ui.button(format!("+ {}", direction.label())).clicked() {
                        self.publish(
                            "Add public requirement",
                            edit::add_port(&p, &n.id, direction),
                        );
                    }
                }
            });
        }
    }
    pub(in crate::ui) fn leaf_interfaces(&mut self, ui: &mut Ui) {
        let p = self.store.snapshot();
        if let Some((_, n)) = p.node(&self.owner) {
            ui.heading(format!("{} · Public boundary", n.name));
            ui.label("This leaf has no internal interface graph. Its public ports are the same ports used by its callers.");
            ui.add_space(20.0);
            ui.group(|ui| {
                ui.set_min_width(400.0);
                ui.heading(&n.name);
                ui.label(&n.purpose);
                for r in &n.ports {
                    ui.label(format!(
                        "{}  {}  · {}",
                        if r.direction == Direction::In {
                            "->"
                        } else {
                            "<-"
                        },
                        r.name,
                        r.contract
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "Unassigned".into())
                    ));
                }
                if n.ports.is_empty() {
                    ui.label("No public information requirements declared yet");
                }
            });
            if ui.button("Open Control Flow").clicked() {
                self.go_scope(n.id.clone(), Layer::Flow);
            }
        }
    }
}
