use super::*;
use crate::edit;
use egui::{Key, Modifiers};
impl Designer {
    pub(super) fn shortcuts(&mut self, ctx: &egui::Context) {
        if self.dialog.is_some() {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S)) {
            self.save(false);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::O)) {
            if let Some(p) = rfd::FileDialog::new()
                .add_filter("System Designer project", &["json"])
                .pick_file()
            {
                self.request_load(LoadAction::Open(p), ctx);
            }
        }
        if !ctx.wants_keyboard_input() {
            if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z)) {
                self.store.undo();
                self.normalize_selection();
            }
            if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Y))
                || ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z))
            {
                self.store.redo();
                self.normalize_selection();
            }
            if ctx.input(|i| i.key_pressed(Key::Delete)) {
                if self.layer == Layer::Flow {
                    self.ask_delete_flow();
                } else {
                    self.ask_delete();
                }
            }
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                if self.layer == Layer::Flow {
                    if self.flow.has_gesture() {
                        self.flow.cancel();
                    } else {
                        self.flow.selection = flow::FlowSelection::default();
                    }
                } else if self.canvas.has_gesture() {
                    self.canvas.cancel();
                } else if self.canvas_session.focus != canvas::Focus::Off {
                    self.canvas_session.focus = canvas::Focus::Off;
                } else {
                    self.selected = Selection::None;
                }
            }
        }
    }
    pub(super) fn workspace(&mut self, ctx: &egui::Context) {
        let enabled = self.dialog.is_none();
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                ui.horizontal_wrapped(|ui| {
                    super::branding::show(ui, 24.0);
                    ui.heading("System Designer");
                    ui.separator();
                    ui.menu_button("File", |ui| {
                        if ui.button("New project").clicked() {
                            ui.close();
                            self.request_load(LoadAction::New, ctx);
                        }
                        if ui.button("Open project…").clicked() {
                            ui.close();
                            if let Some(p) = rfd::FileDialog::new()
                                .add_filter("Project JSON", &["json"])
                                .pick_file()
                            {
                                self.request_load(LoadAction::Open(p), ctx);
                            }
                        }
                        if ui.button("Save  Ctrl+S").clicked() {
                            ui.close();
                            self.save(false);
                        }
                        if ui.button("Save As…").clicked() {
                            ui.close();
                            self.save(true);
                        }
                        ui.separator();
                        if ui.button("Open application design").clicked() {
                            ui.close();
                            self.request_load(LoadAction::Builtin, ctx);
                        }
                        if ui.button("Recovery copies…").clicked() {
                            ui.close();
                            match storage::recovery_files() {
                                Ok(paths) => self.recoveries = paths,
                                Err(e) => self.error = Some(format!("Recovery lookup failed: {e}")),
                            }
                            self.dialog = Some(Dialog::Recovery);
                        }
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            ui.close();
                            self.request_load(LoadAction::Close, ctx);
                        }
                    });
                    let undo = self
                        .store
                        .undo_label()
                        .map(|s| format!("Undo {s}"))
                        .unwrap_or_else(|| "Undo".into());
                    if ui
                        .add_enabled(self.store.undo_label().is_some(), egui::Button::new("Undo"))
                        .on_hover_text(undo)
                        .clicked()
                    {
                        self.store.undo();
                    }
                    if ui
                        .add_enabled(self.store.redo_label().is_some(), egui::Button::new("Redo"))
                        .clicked()
                    {
                        self.store.redo();
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            self.interface_system().is_some(),
                            egui::Button::new("+ Component"),
                        )
                        .clicked()
                    {
                        self.new_component();
                    }
                    if ui
                        .add_enabled(
                            self.interface_system().is_some(),
                            egui::Button::new("Connect ports"),
                        )
                        .clicked()
                    {
                        self.canvas_session.view = canvas::View::Detail;
                        self.canvas.cancel();
                        self.dialog = Some(Dialog::Connection(ConnectionDialog::new(
                            self.store.project(),
                            &self.current,
                            None,
                            None,
                        )));
                    }
                    if ui.button("Contract types").clicked() {
                        self.dialog = Some(Dialog::Catalog);
                    }
                    if ui.button("AI handoff").clicked() {
                        self.open_handoff();
                    }
                    if ui.button("Help").clicked() {
                        self.dialog = Some(Dialog::Help);
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(self.can_go_back(), egui::Button::new("<- Back"))
                        .clicked()
                    {
                        self.back_scope();
                    }
                    if ui
                        .selectable_label(
                            self.owner == crate::behavior::ROOT,
                            &self.store.project().name,
                        )
                        .clicked()
                    {
                        self.go_scope(crate::behavior::ROOT.into(), self.layer);
                    }
                    let p = self.store.snapshot();
                    let mut owners = vec![];
                    let mut owner = self.owner.clone();
                    while owner != crate::behavior::ROOT {
                        owners.push(owner.clone());
                        let Some((system, _)) = p.node(&owner) else {
                            break;
                        };
                        owner = p
                            .owner(&system.id)
                            .map(|(_, n)| n.id.clone())
                            .unwrap_or_else(|| crate::behavior::ROOT.into());
                    }
                    for id in owners.iter().rev() {
                        ui.label("/");
                        if ui.button(crate::behavior::name(&p, id)).clicked() {
                            self.go_scope(id.clone(), self.layer);
                        }
                    }
                    ui.separator();
                    for (layer, label) in [
                        (Layer::Flow, "Control Flow"),
                        (Layer::Interfaces, "Interfaces"),
                    ] {
                        if ui.selectable_label(self.layer == layer, label).clicked() {
                            self.go_scope(self.owner.clone(), layer);
                        }
                    }
                    if self.dirty() {
                        ui.label(egui::RichText::new("• Unsaved").color(egui::Color32::YELLOW));
                    }
                });
            });
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::LIGHT_RED, error);
                } else {
                    ui.label(&self.status);
                }
                if self.error.is_some() && ui.small_button("Dismiss").clicked() {
                    self.error = None;
                }
            });
            if !self.recoveries.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} recovery copies available",
                        self.recoveries.len()
                    ));
                    if ui
                        .add_enabled(enabled, egui::Button::new("Inspect"))
                        .clicked()
                    {
                        self.dialog = Some(Dialog::Recovery);
                    }
                });
            }
        });
        egui::SidePanel::left("tree")
            .default_width(235.0)
            .min_width(180.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    ui.heading("Design tree");
                    ui.label("Select to inspect · > to enter");
                    ui.separator();
                    self.tree(ui);
                });
            });
        egui::SidePanel::right("inspector")
            .default_width(315.0)
            .min_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_enabled_ui(enabled, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if self.layer == Layer::Flow {
                            self.flow_inspector(ui);
                        } else if self.interface_system().is_none() {
                            self.leaf_inspector(ui);
                        } else {
                            self.inspector(ui);
                        }
                    });
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                if self.layer == Layer::Flow {
                    self.flow_view(ui);
                } else if self.interface_system().is_none() {
                    self.leaf_interfaces(ui);
                } else {
                    self.canvas_view(ui);
                }
            });
        });
    }
    fn tree(&mut self, ui: &mut egui::Ui) {
        let p = self.store.snapshot();
        egui::ScrollArea::both().show(ui, |ui| {
            if ui
                .selectable_label(
                    self.owner == crate::behavior::ROOT && self.selected == Selection::None,
                    format!("{}  [root]", p.name),
                )
                .clicked()
            {
                self.go_scope(crate::behavior::ROOT.into(), self.layer);
            }
            let mut pending: Vec<(&System, &Node, usize)> = p
                .system(&p.root)
                .map(|s| s.nodes.iter().rev().map(|n| (s, n, 0)).collect())
                .unwrap_or_default();
            while let Some((system, node, depth)) = pending.pop() {
                ui.push_id(&node.id, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(depth as f32 * 14.0);
                        if node.child.is_some() {
                            let collapsed = self.collapsed.contains(&node.id);
                            if ui.small_button(if collapsed { "+" } else { "−" }).clicked() {
                                if collapsed {
                                    self.collapsed.remove(&node.id);
                                } else {
                                    self.collapsed.insert(node.id.clone());
                                }
                            }
                        } else {
                            ui.add_space(22.0);
                        }
                        let selected = matches!(&self.selected,Selection::Port(ep) if ep.node.as_ref()==Some(&node.id))
                            || self.selected == Selection::Node(node.id.clone())
                            || self.owner == node.id;
                        let r = ui.selectable_label(selected, &node.name);
                        if r.clicked() {
                            if self.current != system.id || self.layer != Layer::Interfaces {
                                self.navigate(system.id.clone());
                            }
                            self.selected = Selection::Node(node.id.clone());
                        }
                        if r.double_clicked() || ui.small_button(">").clicked() {
                            self.go_scope(node.id.clone(), self.layer);
                        }
                    });
                });
                if !self.collapsed.contains(&node.id) {
                    if let Some(child) = node.child.as_ref().and_then(|id| p.system(id)) {
                        for n in child.nodes.iter().rev() {
                            pending.push((child, n, depth + 1));
                        }
                    }
                }
            }
        });
    }
    pub(super) fn new_component(&mut self) {
        self.go_scope(self.owner.clone(), Layer::Interfaces);
        match edit::add_node(self.store.project(), &self.current) {
            Ok((p, id)) => {
                self.publish("Add component", Ok(p));
                self.selected = Selection::Node(id.clone());
                if let Some((_, n)) = self.store.project().node(&id) {
                    self.dialog = Some(Dialog::Node(n.clone()));
                }
                self.canvas.fit_requested = true;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
}
