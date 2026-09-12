use super::*;
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
    node: Option<String>,
    exported: String,
    incoming: String,
    validated: Option<String>,
    summary: String,
}
impl Designer {
    pub(super) fn open_handoff(&mut self) {
        let node = match &self.selected {
            Selection::Node(id) => Some(id.clone()),
            _ => None,
        };
        self.dialog = Some(Dialog::Handoff(HandoffDialog {
            tab: Tab::Initialize,
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
        }));
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
                ui.add(
                    TextEdit::multiline(&mut text)
                        .code_editor()
                        .desired_rows(18)
                        .desired_width(f32::INFINITY),
                );
            }
            Tab::Export => {
                ui.label("Overview, Focus and Lights do not narrow this export. Every real object in the chosen semantic scope is included.");
                let previous = d.scope;
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
                        ui.selectable_value(&mut d.scope, Scope::Subtree, Scope::Subtree.label());
                    });
                if previous != d.scope {
                    d.exported.clear();
                }
                ui.label(match d.scope{
                    Scope::Component=>"Only the selected node is editable. Siblings, wires, and deeper internals are preserved.",
                    Scope::Level=>"Immediate nodes and connections only. Hidden child systems and their ownership are preserved.",
                    Scope::Subtree=>"This level and every descendant. Ancestors and outside branches remain unchanged."
                });
                if d.exported.is_empty() {
                    match exchange::export(
                        self.store.project(),
                        &self.current,
                        d.scope,
                        d.node.as_deref(),
                    )
                    .and_then(|v| serde_json::to_string_pretty(&v).map_err(ModelError::one))
                    {
                        Ok(text) => {
                            d.exported = text;
                            self.error = None;
                        }
                        Err(e) => self.error = Some(e.to_string()),
                    }
                }
                ui.monospace(format!("{} · {} bytes", self.current, d.exported.len()));
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
                ui.add(
                    TextEdit::multiline(&mut text)
                        .code_editor()
                        .desired_rows(18)
                        .desired_width(f32::INFINITY),
                );
            }
            Tab::Import => {
                ui.label("Load the complete returned scope packet at the original level. Do not change its base, context, or public boundary. Full-project JSON is intentionally rejected here.");
                ui.monospace(format!("Target level: {}", self.current));
                if ui.button("Read scope file…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Scope JSON", &["json"])
                        .pick_file()
                    {
                        match std::fs::read_to_string(path) {
                            Ok(text) => {
                                d.incoming = text;
                                d.validated = None;
                            }
                            Err(e) => self.error = Some(e.to_string()),
                        }
                    }
                }
                if ui
                    .add(
                        TextEdit::multiline(&mut d.incoming)
                            .code_editor()
                            .desired_rows(15)
                            .desired_width(f32::INFINITY)
                            .hint_text("Paste the complete JSON object, without markdown fences"),
                    )
                    .changed()
                {
                    d.validated = None;
                    d.summary.clear();
                }
                ui.horizontal(|ui|{
                    if ui.button("Validate candidate").clicked(){
                        let result=serde_json::from_str::<serde_json::Value>(&d.incoming).map_err(ModelError::one).and_then(|packet|exchange::replace(self.store.project(),&packet,&self.current));
                        match result{
                            Ok(p)=>{
                                d.validated=Some(d.incoming.clone());
                                d.summary=format!("Valid merged project: {} systems, {} components. No changes applied yet.",p.systems.len(),p.systems.iter().map(|s|s.nodes.len()).sum::<usize>());
                                self.error=None;
                            },
                            Err(e)=>{
                                d.validated=None;
                                self.error=Some(e.to_string());
                            }
                        }
                    }
                    if ui.add_enabled(d.validated.as_ref()==Some(&d.incoming),egui::Button::new("Apply validated changes")).clicked(){
                        // Deliberately reconstruct and revalidate; never trust a cached candidate.
                        let result=serde_json::from_str::<serde_json::Value>(&d.incoming).map_err(ModelError::one).and_then(|packet|exchange::replace(self.store.project(),&packet,&self.current));
                        self.publish("Apply scoped replacement",result);
                        if self.error.is_none(){
                            d.summary="Applied as one undoable edit. Outside scope preserved.".into();
                            d.validated=None;
                            d.exported.clear();
                            self.canvas.fit_requested=true;
                        }
                    }
                });
                ui.label(&d.summary);
            }
        }
        ui.separator();
        ui.button("Close").clicked()
    }
}
