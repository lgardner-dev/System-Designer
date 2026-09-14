//! Native immediate-mode desktop UI. Only this module depends on eframe/rfd.
mod branding;
pub use branding::window_icon;
mod canvas;
#[cfg(test)]
mod coherence_tests;
mod contracts;
mod diagram;
mod dialogs;
mod flow;
mod handoff;
mod inspector;
mod scope;
mod workspace;
use crate::{edit::Store, model::*, storage};
use canvas::CanvasState;
use contracts::{ConnectionDialog, ContractDialog};
use eframe::egui;
use handoff::HandoffDialog;
use scope::Layer;
use std::{
    collections::HashSet,
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Clone, Debug, PartialEq)]
enum Selection {
    None,
    Node(String),
    Edge(String),
    Boundary(String),
    Port(Endpoint),
    Summary(String, String),
}
#[derive(Clone)]
enum LoadAction {
    New,
    Open(PathBuf),
    Builtin,
    Restore(PathBuf),
    Close,
}
#[derive(Clone)]
enum ConfirmAction {
    Node(String),
    Child(String),
    Port(String, String),
    Edge(String),
    Contract(ContractRef),
}
enum Dialog {
    Position(dialogs::PositionForm),
    Flow(flow::FlowDialog),
    Project {
        name: String,
        purpose: String,
    },
    Node(Node),
    Connection(ConnectionDialog),
    Catalog,
    Contract(ContractDialog),
    Handoff(HandoffDialog),
    Unsaved(LoadAction),
    Confirm {
        message: String,
        action: ConfirmAction,
    },
    Recovery,
    Help,
}
pub struct Designer {
    store: Store,
    current: String,
    owner: String,
    layer: Layer,
    navigation: scope::Navigation,
    flow: flow::FlowState,
    selected: Selection,
    canvas: CanvasState,
    canvas_session: canvas::Session,
    dialog: Option<Dialog>,
    collapsed: HashSet<String>,
    path: Option<PathBuf>,
    file_stamp: Option<String>,
    saved: Option<Project>,
    recovery: Option<PathBuf>,
    last_recovery: Instant,
    recovered_generation: u64,
    allow_close: bool,
    status: String,
    error: Option<String>,
    recoveries: Vec<PathBuf>,
}
impl Designer {
    pub fn new(cc: &eframe::CreationContext<'_>, initial: Option<PathBuf>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 7.0);
        style.spacing.interact_size.y = 26.0;
        style.visuals.panel_fill = egui::Color32::from_rgb(21, 25, 32);
        cc.egui_ctx.set_style(style);
        let mut app = Self::blank();
        if let Some(path) = initial {
            app.load(LoadAction::Open(path), &cc.egui_ctx);
        }
        app
    }
    /// Also used by headless egui interaction tests; no window or GPU is needed.
    pub fn blank() -> Self {
        let p = Project::blank();
        let root = p.root.clone();
        let (recovery, mut error) = match storage::new_recovery_path() {
            Ok(path) => (Some(path), None),
            Err(e) => (None, Some(format!("Recovery unavailable: {e}"))),
        };
        let recoveries = match storage::recovery_files() {
            Ok(paths) => paths,
            Err(e) => {
                error = Some(format!("Recovery lookup failed: {e}"));
                Vec::new()
            }
        };
        Self {
            store: Store::new(p.clone()).expect("blank project is valid"),
            current: root,
            owner: crate::behavior::ROOT.into(),
            layer: Layer::Flow,
            navigation: scope::Navigation::default(),
            flow: flow::FlowState::default(),
            selected: Selection::None,
            canvas: CanvasState::default(),
            canvas_session: canvas::Session::default(),
            dialog: None,
            collapsed: HashSet::new(),
            path: None,
            file_stamp: None,
            saved: Some(p),
            recovery,
            last_recovery: Instant::now(),
            recovered_generation: 0,
            allow_close: false,
            status: "New local project. Start a flow or explore Interfaces.".into(),
            error,
            recoveries,
        }
    }
    fn dirty(&self) -> bool {
        self.saved.as_ref() != Some(self.store.project())
    }
    fn publish(&mut self, label: &str, result: Result<Project>) {
        let promoted = result
            .as_ref()
            .is_ok_and(|p| p.version == 3 && self.store.project().version != 3);
        match result.and_then(|p| self.store.publish(label, p)) {
            Ok(()) => {
                self.error = None;
                self.status = if promoted {
                    format!(
                        "{label}. Signed layout uses project version 3; older readers cannot open it. Undo restores the previous version."
                    )
                } else {
                    label.into()
                };
                self.normalize_selection();
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
    fn navigate(&mut self, sid: String) {
        let p = self.store.project();
        let owner = if sid == p.root {
            crate::behavior::ROOT.into()
        } else if let Some((_, n)) = p.owner(&sid) {
            n.id.clone()
        } else {
            return;
        };
        self.go_scope(owner, Layer::Interfaces);
    }
    fn normalize_selection(&mut self) {
        self.normalize_scope();
        if self
            .canvas
            .generation
            .is_some_and(|g| g != self.store.generation)
        {
            if self.canvas.has_gesture() {
                self.status = "Canvas gesture cancelled because the project changed.".into();
            }
            self.canvas.cancel();
            self.flow.cancel();
        }
        self.canvas.generation = Some(self.store.generation);
        if self.selected != Selection::None
            && !canvas::selection_valid(self.store.project(), &self.current, &self.selected)
        {
            self.selected = Selection::None;
            self.status =
                "The selected item was changed or removed; selection and focus cleared.".into();
        }
        self.canvas_session.normalize(&self.selected);
    }
    fn clear_recovery(&mut self) {
        if let Some(path) = &self.recovery
            && let Err(e) = std::fs::remove_file(path)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            self.error = Some(format!("Could not remove recovery copy: {e}"));
        }
    }
    fn request_load(&mut self, action: LoadAction, ctx: &egui::Context) {
        if self.dirty() {
            self.dialog = Some(Dialog::Unsaved(action));
        } else {
            self.load(action, ctx);
        }
    }
    fn load(&mut self, action: LoadAction, ctx: &egui::Context) {
        if matches!(action, LoadAction::Close) {
            self.clear_recovery();
            self.allow_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let is_new = matches!(action, LoadAction::New);
        let result: Result<(Project, Option<PathBuf>, Option<String>, bool)> = match action {
            LoadAction::New => Ok((Project::blank(), None, None, false)),
            LoadAction::Open(path) => {
                storage::read_project(&path).map(|(p, s)| (p, Some(path), Some(s), false))
            }
            LoadAction::Builtin => parse(crate::APPLICATION_DESIGN).map(|p| (p, None, None, true)),
            LoadAction::Restore(path) => {
                storage::read_recovery(&path).map(|r| (r.project, None, None, true))
            }
            LoadAction::Close => unreachable!(),
        };
        match result {
            Ok((p, path, stamp, unsaved)) => {
                self.error = None;
                self.clear_recovery();
                self.current = p.root.clone();
                self.owner = crate::behavior::ROOT.into();
                self.layer = if is_new || p.behavior.contains_key(crate::behavior::ROOT) {
                    Layer::Flow
                } else {
                    Layer::Interfaces
                };
                self.navigation = scope::Navigation::default();
                self.flow = flow::FlowState::default();
                self.collapsed = p
                    .systems
                    .iter()
                    .flat_map(|s| s.nodes.iter())
                    .filter(|n| n.child.is_some())
                    .map(|n| n.id.clone())
                    .collect();
                self.saved = if unsaved { None } else { Some(p.clone()) };
                self.path = path;
                self.file_stamp = stamp;
                self.store = Store::new(p).expect("load result was validated");
                self.selected = Selection::None;
                self.canvas = CanvasState::default();
                self.canvas_session = canvas::Session::default();
                self.dialog = None;
                self.status = if unsaved {
                    "Opened as an unsaved copy; Save As to keep it."
                } else {
                    "Project opened."
                }
                .into();
                self.recovered_generation = u64::MAX;
            }
            Err(e) => self.error = Some(e.to_string()),
        }
    }
    fn save(&mut self, save_as: bool) -> bool {
        let chosen = if save_as || self.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("System Designer project", &["json"])
                .set_file_name("design.project.json")
                .save_file()
        } else {
            self.path.clone()
        };
        let Some(path) = chosen else {
            return false;
        };
        let expected = if self.path.as_ref() == Some(&path) {
            Ok(self.file_stamp.clone())
        } else {
            storage::stamp(&path)
        };
        match expected
            .and_then(|stamp| storage::save_project(&path, self.store.project(), stamp.as_deref()))
        {
            Ok(stamp) => {
                self.path = Some(path);
                self.file_stamp = Some(stamp);
                self.saved = Some(self.store.project().clone());
                self.error = None;
                self.clear_recovery();
                self.status =
                    "Saved project; a previous valid file is backed up when present.".into();
                true
            }
            Err(e) => {
                self.error = Some(e.to_string());
                false
            }
        }
    }
    fn recover_periodically(&mut self, ctx: &egui::Context) {
        if self.dirty()
            && self.recovered_generation != self.store.generation
            && self.last_recovery.elapsed() >= Duration::from_secs(2)
        {
            self.last_recovery = Instant::now();
            if let Some(path) = &self.recovery {
                match storage::write_recovery(path, self.store.project(), self.path.as_deref()) {
                    Ok(()) => {
                        self.recovered_generation = self.store.generation;
                    }
                    Err(e) => self.error = Some(format!("Recovery save failed. Use Save As: {e}")),
                }
            }
        }
        if self.dirty() {
            ctx.request_repaint_after(Duration::from_secs(2));
        }
    }
    pub fn draw(&mut self, ctx: &egui::Context) {
        self.normalize_selection();
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close {
            if self.dirty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.dialog = Some(Dialog::Unsaved(LoadAction::Close));
            } else {
                self.clear_recovery();
            }
        }
        self.shortcuts(ctx);
        if matches!(self.dialog, Some(Dialog::Connection(_))) {
            self.canvas_session.view = canvas::View::Detail;
            self.canvas.cancel();
        }
        self.workspace(ctx);
        if matches!(self.dialog, Some(Dialog::Connection(_))) {
            self.canvas_session.view = canvas::View::Detail;
            self.canvas.cancel();
        }
        self.show_dialog(ctx);
        self.recover_periodically(ctx);
    }
}
impl eframe::App for Designer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(ctx);
    }
}
