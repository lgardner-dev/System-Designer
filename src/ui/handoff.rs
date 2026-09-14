use super::dialogs::{Action, Actions, Header};
use super::*;
use crate::behavior::BehaviorChanges;
use crate::exchange::{self, Scope};
use egui::{TextEdit, Ui};
#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Initialize,
    Export,
    Import,
}
pub(super) struct HandoffDialog {
    tab: Tab,
    scope: Scope,
    behavior: bool,
    interface_system: String,
    clear: bool,
    clear_confirmed: bool,
    node: Option<String>,
    exported: String,
    incoming: String,
    validated: Option<String>,
    summary: String,
    changes: Option<BehaviorChanges>,
}
impl HandoffDialog {
    pub(super) fn header(&self) -> Header<'_> {
        Header {
            title: "AI handoff",
            primary: (self.tab == Tab::Import).then_some("Apply validated changes"),
            preview: (self.tab == Tab::Import).then_some("Validate candidate"),
        }
    }
    pub(super) fn status(&self) -> String {
        format!(
            "{}{}",
            if self.clear {
                "Destructive removals require confirmation. "
            } else {
                ""
            },
            self.summary
        )
    }

    pub(super) fn confirmation(&mut self, ui: &mut Ui) {
        if self.clear {
            ui.checkbox(
                &mut self.clear_confirmed,
                "Confirm the control flow removals listed above",
            );
        }
    }
    fn review_details(&self, ui: &mut Ui) {
        ui.label(&self.summary);
        if let Some(changes) = &self.changes {
            for clear in &changes.clears {
                ui.colored_label(egui::Color32::YELLOW, clear);
            }
            if !changes.details.is_empty() {
                ui.collapsing(
                    format!("Exact behavior changes ({})", changes.details.len()),
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("behavior_changes")
                            .max_height(180.0)
                            .show(ui, |ui| {
                                for detail in &changes.details {
                                    ui.label(detail);
                                }
                            });
                    },
                );
            }
            if !changes.new_issues.is_empty() {
                egui::CollapsingHeader::new(format!("New draft issues ({})", changes.new_issues.len())).default_open(true).show(ui, |ui| {
                            ui.small("These are saveable incomplete drafts, not validation failures or execution results.");
                            egui::ScrollArea::vertical().id_salt("new_behavior_issues").max_height(110.0).show(ui, |ui| {
                                for issue in &changes.new_issues { ui.label(issue); }
                            });
                        });
            }
        }
    }
    fn reset_preview(&mut self) {
        self.validated = None;
        self.summary.clear();
        self.changes = None;
        self.clear = false;
        self.clear_confirmed = false;
    }
}
impl Designer {
    pub(super) fn open_handoff(&mut self) {
        let node = if self.interface_system().is_none() {
            Some(self.owner.clone())
        } else {
            match &self.selected {
                Selection::Node(id) => Some(id.clone()),
                _ => None,
            }
        };
        self.dialog = Some(Dialog::Handoff(HandoffDialog {
            tab: Tab::Initialize,
            behavior: self.layer == Layer::Flow,
            interface_system: self
                .interface_system()
                .map(|s| s.id.clone())
                .or_else(|| {
                    self.store
                        .project()
                        .node(&self.owner)
                        .map(|(s, _)| s.id.clone())
                })
                .unwrap_or_default(),
            clear: false,
            clear_confirmed: false,
            scope: if node.is_some() {
                Scope::Component
            } else {
                Scope::Level
            },
            node,
            exported: String::new(),
            incoming: String::new(),
            validated: None,
            summary: String::new(),
            changes: None,
        }));
    }
    fn validate_handoff(&mut self, d: &mut HandoffDialog) {
        d.reset_preview();
        let result = serde_json::from_str::<serde_json::Value>(&d.incoming)
            .map_err(ModelError::one)
            .and_then(|packet| {
                exchange::replace_packet(
                    self.store.project(),
                    &packet,
                    &d.interface_system,
                    &self.owner,
                )
            });
        match result {
            Ok(p) => {
                let changes = BehaviorChanges::between(self.store.project(), &p);
                d.clear = !changes.clears.is_empty();
                d.summary = format!(
                    "{}\nValid merged project: {} systems, {} components, {} behavior scopes. No changes applied yet.",
                    changes.summary(),
                    p.systems.len(),
                    p.systems.iter().map(|s| s.nodes.len()).sum::<usize>(),
                    p.behavior.len()
                );
                d.changes = Some(changes);
                d.validated = Some(d.incoming.clone());
                self.error = None;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
    fn apply_handoff(&mut self, d: &mut HandoffDialog) {
        // Reconstruct and revalidate, then verify the reviewed semantic changes.
        let result = serde_json::from_str::<serde_json::Value>(&d.incoming)
            .map_err(ModelError::one)
            .and_then(|packet| {
                exchange::replace_packet(
                    self.store.project(),
                    &packet,
                    &d.interface_system,
                    &self.owner,
                )
            })
            .and_then(|p| {
                let changes = BehaviorChanges::between(self.store.project(), &p);
                if d.validated.as_ref() != Some(&d.incoming) || d.changes.as_ref() != Some(&changes)
                {
                    return Err(ModelError::one(
                        "Scope changes differ from the preview; validate again.",
                    ));
                }
                if !changes.clears.is_empty() && !d.clear_confirmed {
                    return Err(ModelError::one(
                        "Confirm the control flow removals before applying.",
                    ));
                }
                Ok(p)
            });
        self.publish("Apply scoped replacement", result);
        if self.error.is_none() {
            d.reset_preview();
            d.summary = "Applied as one undoable edit. Outside scope preserved.".into();
            d.exported.clear();
            self.canvas.fit_requested = true;
        }
    }
    fn export_text(&mut self, text: &str, name: &str) {
        let Some(path) = rfd::FileDialog::new().set_file_name(name).save_file() else {
            return;
        };
        let same_file = self.path.as_ref().is_some_and(|active| {
            active == &path
                || std::fs::canonicalize(active)
                    .ok()
                    .zip(std::fs::canonicalize(&path).ok())
                    .is_some_and(|(a, b)| a == b)
        });
        if same_file {
            self.error = Some(
                "Export cannot overwrite the active project file. Choose a different filename."
                    .into(),
            );
            return;
        }
        match storage::atomic_write(&path, text.as_bytes()) {
            Ok(()) => {
                self.status = format!("Exported {}", path.display());
                self.error = None;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
    pub(super) fn handoff_form(
        &mut self,
        ui: &mut Ui,
        d: &mut HandoffDialog,
        ctx: &egui::Context,
        actions: &mut Actions,
    ) -> bool {
        ui.heading("AI handoff");
        ui.label("Manual exchange only. This app does not send data to any AI service.");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut d.tab, Tab::Initialize, "1 · Initialize chat");
            ui.selectable_value(&mut d.tab, Tab::Export, "2 · Export scope");
            ui.selectable_value(&mut d.tab, Tab::Import, "3 · Load changes");
        });
        ui.separator();
        match d.tab {
            Tab::Initialize => {
                ui.label("Copy this into a fresh chat, then supply a scope export and your design request. The prompt contains the design method and exact format, but no project data.");
                ui.horizontal(|ui| {
                    if ui.button("Copy initialization prompt").clicked() {
                        ctx.copy_text(crate::INITIALIZATION.into());
                        self.status = "Initialization prompt copied.".into();
                    }
                    if ui.button("Save prompt…").clicked() {
                        self.export_text(
                            crate::INITIALIZATION,
                            "system-designer-initialization.txt",
                        );
                    }
                });
                let mut text = crate::INITIALIZATION;
                egui::ScrollArea::vertical()
                    .id_salt("initialization_text")
                    .max_height(450.0)
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut text)
                                .code_editor()
                                .desired_rows(18)
                                .desired_width(f32::INFINITY),
                        );
                    });
            }
            Tab::Export => {
                ui.label("Overview, Focus and Lights do not narrow this export. Every real object in the chosen semantic scope is included.");
                let previous_layer = d.behavior;
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut d.behavior, false, "Interfaces");
                    ui.selectable_value(&mut d.behavior, true, "Control Flow — current component");
                });
                if previous_layer != d.behavior {
                    d.exported.clear();
                }
                if d.behavior {
                    ui.label("Writable: this component's flow only. Preserved: every interface, other flows and both saved layouts. Read-only: owner boundary, child signatures/outcomes, exact local exchanges, callers and contract catalog.");
                    ui.small("Context policy v1 includes the complete contract catalog and some repeated child signatures. Unrelated catalog edits can stale this packet; this is not minimal-context delivery.");
                }
                let previous = d.scope;
                ui.add_enabled_ui(!d.behavior, |ui| {
                    egui::ComboBox::from_id_salt("export_scope")
                        .selected_text(d.scope.label())
                        .show_ui(ui, |ui| {
                            if d.node.is_some() {
                                ui.selectable_value(
                                    &mut d.scope,
                                    Scope::Component,
                                    Scope::Component.label(),
                                );
                            }
                            ui.selectable_value(&mut d.scope, Scope::Level, Scope::Level.label());
                            ui.selectable_value(
                                &mut d.scope,
                                Scope::Subtree,
                                Scope::Subtree.label(),
                            );
                        });
                });
                if previous != d.scope {
                    d.exported.clear();
                }
                if !d.behavior {
                    if self.interface_system().is_none() {
                        ui.label("Leaf boundary: Interfaces exports target this component in its owning parent level.");
                    }
                    ui.label("Other behavior and layouts are preserved. Context is read-only; version-2 interface context may include sibling behavior interiors.");
                    ui.label(match d.scope{
                    Scope::Component=>"Only the selected node is editable. Siblings, wires, and deeper internals are preserved.",
                    Scope::Level=>"Immediate nodes and connections only. Hidden child systems and their ownership are preserved.",
                    Scope::Subtree=>"This level and every descendant. Ancestors and outside branches remain unchanged."
                });
                }
                if d.exported.is_empty() {
                    match if d.behavior {
                        crate::behavior::export(self.store.project(), &self.owner)
                    } else {
                        exchange::export(
                            self.store.project(),
                            &d.interface_system,
                            d.scope,
                            d.node.as_deref(),
                        )
                    }
                    .and_then(|v| serde_json::to_string_pretty(&v).map_err(ModelError::one))
                    {
                        Ok(text) => {
                            d.exported = text;
                            self.error = None;
                        }
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                ui.monospace(format!(
                    "{} · {} bytes",
                    if d.behavior {
                        &self.owner
                    } else {
                        &d.interface_system
                    },
                    d.exported.len()
                ));
                ui.horizontal(|ui| {
                    if ui.button("Copy scope JSON").clicked() {
                        ctx.copy_text(d.exported.clone());
                        self.status = "Scope copied.".into();
                    }
                    if ui.button("Export scope file…").clicked() {
                        self.export_text(&d.exported, "design.scope.json");
                    }
                });
                let mut text = d.exported.as_str();
                egui::ScrollArea::vertical()
                    .id_salt("exported_text")
                    .max_height(360.0)
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut text)
                                .code_editor()
                                .desired_rows(18)
                                .desired_width(f32::INFINITY),
                        );
                    });
            }
            Tab::Import => {
                d.review_details(ui);
                ui.label("Load the complete returned scope packet at the original level. Do not change its base, context, or public boundary. Full-project JSON is intentionally rejected here.");
                ui.monospace(format!(
                    "Interfaces target: {} · Control Flow owner: {}",
                    d.interface_system, self.owner
                ));
                if ui.button("Read scope file…").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Scope JSON", &["json"])
                        .pick_file()
                {
                    match std::fs::read_to_string(path) {
                        Ok(text) => {
                            d.incoming = text;
                            d.reset_preview();
                        }
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                let response = egui::ScrollArea::vertical()
                    .id_salt("incoming_text")
                    .max_height(300.0)
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut d.incoming)
                                .code_editor()
                                .desired_rows(15)
                                .desired_width(f32::INFINITY)
                                .hint_text(
                                    "Paste the complete JSON object, without markdown fences",
                                ),
                        )
                    });
                if response.inner.changed() {
                    d.reset_preview();
                }
                {
                    if actions.take(Action::Preview, true) {
                        self.validate_handoff(d);
                    }
                    if actions.take(
                        Action::Primary,
                        d.validated.as_ref() == Some(&d.incoming)
                            && (!d.clear || d.clear_confirmed),
                    ) {
                        self.apply_handoff(d);
                    }
                }
            }
        }
        ui.separator();
        actions.cancel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::behavior::{self, Flow, ROOT};
    fn frame(a: &mut Designer, ctx: &egui::Context, events: Vec<egui::Event>) -> egui::FullOutput {
        ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500.0, 1000.0),
                )),
                events,
                focused: true,
                ..Default::default()
            },
            |ctx| a.draw(ctx),
        )
    }
    fn locate(s: &egui::Shape, label: &str) -> Option<egui::Pos2> {
        match s {
            egui::Shape::Text(t) if t.galley.text() == label => Some(t.pos + t.galley.size() * 0.5),
            egui::Shape::Vec(v) => v.iter().find_map(|s| locate(s, label)),
            _ => None,
        }
    }
    fn click(a: &mut Designer, ctx: &egui::Context, label: &str) {
        frame(a, ctx, vec![]);
        let out = frame(a, ctx, vec![]);
        let pos = out
            .shapes
            .iter()
            .find_map(|s| locate(&s.shape, label))
            .unwrap_or_else(|| panic!("Missing visible control {label}"));
        for pressed in [true, false] {
            frame(
                a,
                ctx,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
    }
    fn open(a: &mut Designer, packet: &serde_json::Value) {
        a.open_handoff();
        if let Some(Dialog::Handoff(d)) = &mut a.dialog {
            d.tab = Tab::Import;
            d.incoming = serde_json::to_string_pretty(packet).expect("packet");
        }
    }
    #[test]
    fn f3_review_removed_transition_is_disclosed_before_apply() {
        let ctx = egui::Context::default();
        let mut a = Designer::blank();
        let p = parse(include_str!(
            "../../tests/fixtures/control-flow-review/after-extraction.project.json"
        ))
        .expect("review fixture");
        a.current = p.root.clone();
        a.store = Store::new(p.clone()).expect("store");
        let packet = serde_json::from_str(include_str!(
            "../../tests/fixtures/control-flow-review/returned-delete-transition.behavior.json"
        ))
        .expect("returned fixture");
        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        assert!(a.error.is_none(), "{:?}", a.error);
        assert_eq!(a.store.project(), &p);
        assert_eq!(a.store.generation, 0);
        assert!(
            matches!(&a.dialog, Some(Dialog::Handoff(d)) if d.summary.contains("1 removed")),
            "removed return must be disclosed"
        );
        let detail = match &a.dialog {
            Some(Dialog::Handoff(d)) => d.changes.as_ref().expect("changes").details[0].clone(),
            _ => panic!("handoff"),
        };
        assert!(detail.contains("Removed transition t3: call.1 → done"));
        click(&mut a, &ctx, "Exact behavior changes (1)");
        let out = frame(&mut a, &ctx, vec![]);
        assert!(
            out.shapes
                .iter()
                .any(|s| locate(&s.shape, &detail).is_some())
        );
        assert!(
            out.shapes
                .iter()
                .any(|s| locate(&s.shape, "New draft issues (3)").is_some())
        );
        click(&mut a, &ctx, "Apply validated changes");
        assert!(a.error.is_none(), "{:?}", a.error);
        assert!(
            !a.store.project().behavior[ROOT]
                .transitions
                .iter()
                .any(|t| t.id == "t3")
        );
        assert_eq!(a.store.generation, 1);
        a.store.undo();
        assert_eq!(a.store.project(), &p);
    }
    #[test]
    fn f3_empty_contents_need_confirmation_and_empty_creation_does_not() {
        let ctx = egui::Context::default();
        let mut a = Designer::blank();
        let p = behavior::set(a.store.project(), ROOT, Flow::starter()).expect("flow");
        a.store = Store::new(p.clone()).expect("store");
        let mut packet = behavior::export(&p, ROOT).expect("export");
        packet["flow"] = serde_json::to_value(Flow::default()).expect("empty");
        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        assert!(
            matches!(&a.dialog, Some(Dialog::Handoff(d)) if d.clear && d.changes.as_ref().expect("changes").clears[0].contains("empty draft flow object"))
        );
        click(&mut a, &ctx, "Apply validated changes");
        assert_eq!(a.store.project(), &p);
        assert_eq!(a.store.generation, 0);
        click(
            &mut a,
            &ctx,
            "Confirm the control flow removals listed above",
        );
        click(&mut a, &ctx, "Apply validated changes");
        assert!(a.error.is_none(), "{:?}", a.error);
        assert_eq!(a.store.project().behavior[ROOT], Flow::default());
        assert_eq!(a.store.generation, 1);
        a.store.undo();
        assert_eq!(a.store.project(), &p);

        let mut a = Designer::blank();
        let mut packet = behavior::export(a.store.project(), ROOT).expect("absent");
        packet["flow"] = serde_json::to_value(Flow::default()).expect("empty");
        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        assert!(matches!(&a.dialog, Some(Dialog::Handoff(d)) if !d.clear));
        click(&mut a, &ctx, "Apply validated changes");
        assert!(a.error.is_none(), "{:?}", a.error);
        assert_eq!(a.store.project().behavior[ROOT], Flow::default());
    }
    #[test]
    fn f3_cancel_invalid_and_stale_reviews_preserve_history_and_generation() {
        let ctx = egui::Context::default();
        let mut a = Designer::blank();
        let p = behavior::set(a.store.project(), ROOT, Flow::starter()).expect("flow");
        a.store = Store::new(p.clone()).expect("store");
        let mut changed = p.clone();
        changed.behavior.get_mut(ROOT).expect("flow").primitive = "Temporary edit".into();
        a.store
            .publish("Temporary", changed.clone())
            .expect("publish");
        a.store.undo();
        let generation = a.store.generation;
        let mut packet = behavior::export(&p, ROOT).expect("export");
        packet["flow"]["transitions"] = serde_json::json!([]);
        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        click(&mut a, &ctx, "Cancel");
        assert!(a.dialog.is_none());
        assert_eq!(a.store.project(), &p);
        assert_eq!(a.store.generation, generation);
        assert_eq!(a.store.undo_label(), None);
        assert_eq!(a.store.redo_label(), Some("Temporary"));

        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        if let Some(Dialog::Handoff(d)) = &mut a.dialog {
            d.incoming = "{}".into();
        }
        click(&mut a, &ctx, "Validate candidate");
        assert!(a.error.is_some());
        assert!(
            matches!(&a.dialog, Some(Dialog::Handoff(d)) if d.changes.is_none() && d.summary.is_empty() && d.validated.is_none() && !d.clear_confirmed)
        );
        assert_eq!(a.store.project(), &p);
        assert_eq!(a.store.generation, generation);
        assert_eq!(a.store.redo_label(), Some("Temporary"));

        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        a.store.redo();
        let generation = a.store.generation;
        click(&mut a, &ctx, "Apply validated changes");
        assert!(a.error.as_ref().is_some_and(|s| s.contains("Stale")));
        assert_eq!(a.store.project(), &changed);
        assert_eq!(a.store.generation, generation);
        assert_eq!(a.store.undo_label(), Some("Temporary"));
        assert_eq!(a.store.redo_label(), None);
    }
    #[test]
    fn full_workspace_handoff_requires_explicit_clear_and_rechecks_stale_input() {
        let ctx = egui::Context::default();
        let mut a = Designer::blank();
        a.publish(
            "Start",
            behavior::set(a.store.project(), ROOT, Flow::starter()),
        );
        let p = a.store.project().clone();
        let generation = a.store.generation;
        let mut packet = behavior::export(&p, ROOT).expect("export");
        packet["flow"] = serde_json::Value::Null;
        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        assert!(matches!(&a.dialog,Some(Dialog::Handoff(d)) if d.clear && !d.clear_confirmed));
        click(&mut a, &ctx, "Apply validated changes");
        assert_eq!(a.store.project(), &p);
        assert_eq!(a.store.generation, generation);
        click(
            &mut a,
            &ctx,
            "Confirm the control flow removals listed above",
        );
        click(&mut a, &ctx, "Apply validated changes");
        assert!(a.store.project().behavior.is_empty());
        assert_eq!(a.store.generation, generation + 1);
        a.store.undo();
        a.normalize_selection();
        assert_eq!(a.store.project(), &p);
        let packet = behavior::export(&p, ROOT).expect("export");
        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        let mut step = p.behavior[ROOT].steps[1].clone();
        step.name = "Concurrent edit".into();
        a.publish("Edit", behavior::save_step(&p, ROOT, step));
        let changed = a.store.project().clone();
        let generation = a.store.generation;
        click(&mut a, &ctx, "Apply validated changes");
        assert!(a.error.as_ref().is_some_and(|s| s.contains("Stale")));
        assert_eq!(a.store.project(), &changed);
        assert_eq!(a.store.generation, generation);
    }
    #[test]
    fn full_workspace_long_handoff_keeps_validation_visible() {
        let ctx = egui::Context::default();
        let mut a = Designer::blank();
        let p = parse(crate::APPLICATION_DESIGN).expect("self design");
        a.current = p.root.clone();
        a.store = Store::new(p).expect("store");
        let packet = behavior::export(a.store.project(), ROOT).expect("export");
        open(&mut a, &packet);
        click(&mut a, &ctx, "Validate candidate");
        assert!(a.error.is_none(), "{:?}", a.error);
        assert!(matches!(&a.dialog,Some(Dialog::Handoff(d)) if d.validated.is_some()));
    }
}
