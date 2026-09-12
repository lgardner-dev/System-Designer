#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! A read-only native worked candidate, not a new production file format/editor.
mod model;
#[cfg(test)]
mod tests;
mod view;

use eframe::egui::{self, Color32, RichText};
use model::{Document, Flow, Kind};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Tab {
    Control,
    Interfaces,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Selection {
    None,
    Step(String),
    Transition(String),
    Exchange(String),
    Contract(String),
}
#[derive(Clone)]
struct Location {
    scope: usize,
    original: bool,
    tab: Tab,
    selection: Selection,
}
struct App {
    document: Document,
    at: Location,
    history: Vec<Location>,
    cameras: BTreeMap<(usize, bool, Tab), view::Camera>,
    lights: bool,
    status: String,
    prompt: bool,
    icon: Option<egui::TextureHandle>,
    capture: Option<Capture>,
}
struct Capture {
    dir: PathBuf,
    stage: usize,
    frames: u32,
    requested: bool,
}
impl App {
    fn new(document: Document) -> Self {
        Self {
            document,
            at: Location {
                scope: 0,
                original: false,
                tab: Tab::Control,
                selection: Selection::None,
            },
            history: vec![],
            cameras: BTreeMap::new(),
            lights: false,
            status: "Select a step or arrow. Double-click the experiment subprocess to enter it."
                .into(),
            prompt: false,
            icon: None,
            capture: None,
        }
    }
    fn flow(&self) -> Flow {
        if self.at.original {
            self.document.original()
        } else {
            self.document.study.flows[self.at.scope].clone()
        }
    }
    fn enter(&mut self, scope: usize) {
        self.history.push(self.at.clone());
        self.at.scope = scope;
        self.at.original = false;
        self.at.selection = Selection::None;
    }
    fn back(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.at = previous;
        }
    }
    fn set_tab(&mut self, tab: Tab) {
        if tab == Tab::Control {
            if let Selection::Exchange(id) = &self.at.selection {
                if let Some(t) = self
                    .flow()
                    .transitions
                    .iter()
                    .find(|t| t.exchanges.contains(id))
                {
                    self.at.selection = Selection::Transition(t.id.clone());
                }
            }
        }
        if tab == Tab::Interfaces && self.at.original {
            // The reference graph is not a competing interface project. Locate
            // the same source item in the proposed component decomposition.
            self.at.scope = match &self.at.selection {
                Selection::Step(id)
                    if ["step.S2.hyp", "step.S2.plan", "step.S2.run"].contains(&id.as_str()) =>
                {
                    1
                }
                Selection::Transition(id) if ["S2.e5", "S2.e6"].contains(&id.as_str()) => 1,
                _ => 0,
            };
            self.at.original = false;
        }
        self.at.tab = tab;
    }
    fn export_interfaces(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("s2-inquiry.interfaces.project.json")
            .save_file()
        {
            self.status = match std::fs::write(&path, model::INTERFACES) {
                Ok(()) => format!(
                    "Exported version-1 interfaces to {}. Control flow is a separate study file.",
                    path.display()
                ),
                Err(e) => format!("Export failed: {e}"),
            };
        }
    }
    fn sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        ui.label(
            RichText::new("S2 WORKED CANDIDATE")
                .small()
                .color(view::ACCENT),
        );
        ui.add_space(12.0);
        if ui
            .selectable_label(self.at.scope == 0 && !self.at.original, "S2 · Inquiry")
            .clicked()
        {
            self.enter(0);
        }
        ui.indent("subtree", |ui| {
            if ui
                .selectable_label(self.at.scope == 1, "Produce evidence ›")
                .clicked()
            {
                self.enter(1);
            }
            if self.at.scope == 1 {
                for (id, name) in [
                    ("S2.hyp", "Compete hypotheses"),
                    ("S2.plan", "Seal experiment"),
                    ("S2.run", "Execute & observe"),
                ] {
                    if ui
                        .selectable_label(
                            self.at.selection == Selection::Step(format!("step.{id}")),
                            name,
                        )
                        .clicked()
                    {
                        self.at.selection = Selection::Step(format!("step.{id}"));
                    }
                }
            }
        });
        ui.add_space(14.0);
        ui.separator();
        ui.label(RichText::new("Compare").strong());
        if ui
            .selectable_label(self.at.original, "Original Atlas · 10 steps")
            .clicked()
        {
            self.history.push(self.at.clone());
            self.at = Location {
                scope: 0,
                original: true,
                tab: Tab::Control,
                selection: Selection::None,
            };
        }
        ui.add_space(12.0);
        ui.label(RichText::new("Simplicity without loss").strong());
        ui.small("The candidate puts three experimental-work steps behind one proposed boundary. Open it to see each step—not an opaque shortcut.");
        ui.add_space(10.0);
        ui.label(RichText::new("8 local components").color(view::ACCENT));
        ui.small("Original: 10 actions/decisions/outcomes. Candidate: 8 at S2, 3 inside the experiment subprocess.");
        ui.add_space(10.0);
        ui.small("Local cyclomatic complexity: 4 -> 4. The boundary reduces visible size, not the number of decisions.");
        ui.add_space(15.0);
        ui.separator();
        ui.label(RichText::new("Legend").strong());
        ui.small("Control Flow uses standard flowchart notation:\nRectangle · process/action\nDiamond · decision/assessment\nPredefined process · subprocess\nTerminator · entry/outcome\nInterfaces remain ordinary system cards");
        ui.add_space(12.0);
        ui.small("Colors supplement labels. These are design declarations—not execution, completion, or approval status.");
    }
    fn inspector(&mut self, ui: &mut egui::Ui) {
        let flow = self.flow();
        ui.add_space(8.0);
        match self.at.selection.clone() {
            Selection::None => {
                ui.heading("Design intent");
                ui.label("Control flow answers what happens next. Interfaces answer what information crosses a boundary.");
                ui.separator();
                ui.label(RichText::new("Entry").strong());
                ui.label(
                    self.document.study.source["entry"]
                        .as_str()
                        .unwrap_or("Unspecified"),
                );
                ui.label(RichText::new("Eligibility").strong());
                ui.label(
                    self.document.study.source["eligibility"]
                        .as_str()
                        .unwrap_or("Unspecified"),
                );
                ui.label(RichText::new("Authority").strong());
                ui.label(
                    self.document.study.source["authority"]
                        .as_str()
                        .unwrap_or("Unspecified"),
                );
                ui.separator();
                ui.collapsing("Source fidelity & limits", |ui| {
                    for limit in &self.document.study.limits {
                        ui.label(limit);
                        ui.add_space(8.0);
                    }
                });
                ui.collapsing("Exact source binding", |ui| {
                    for (k, v) in &self.document.study.provenance {
                        ui.label(k);
                        ui.small(v);
                    }
                });
                ui.add_space(10.0);
                ui.label(RichText::new("Try this").strong());
                ui.label("Select ‘sufficient’, then switch to Interfaces: one control transition corresponds to two exact typed exchanges.");
            }
            Selection::Step(id) => {
                let Some(step) = flow.step(&id) else {
                    self.at.selection = Selection::None;
                    return;
                };
                ui.heading(&step.name);
                ui.monospace(&step.id);
                ui.label(&step.purpose);
                if let Some(source_id) = step.source_steps.first() {
                    if let Some(source) = self
                        .document
                        .source_steps
                        .iter()
                        .find(|s| s.id == *source_id)
                    {
                        ui.label(format!("Source role: {}", source.kind));
                    }
                }
                if step.kind == Kind::Choice {
                    ui.colored_label(view::WARN, "A recorded semantic assessment / disposition. Not an evaluated Boolean expression.");
                }
                if step.kind == Kind::Call {
                    ui.colored_label(
                        view::WARN,
                        "Proposed responsibility boundary—not adopted Method semantics.",
                    );
                    if ui.button("Enter subprocess ›").clicked() {
                        self.enter(1);
                    }
                }
                ui.separator();
                let mut contract_ids = vec![];
                if let Some(component) = step.component.as_deref() {
                    if let Some((sid, node)) = self.document.project.node(component) {
                        ui.label(RichText::new("Declared interfaces").strong());
                        ui.small(format!(
                            "Component: {} · containing system: {}",
                            node.id, sid.id
                        ));
                        for port in &node.ports {
                            if let Some(c) = &port.contract {
                                ui.label(format!("{} · {}", port.direction.label(), port.name));
                                if ui.small_button(c.to_string()).clicked() {
                                    contract_ids.push(c.id.clone());
                                }
                            }
                        }
                    }
                }
                ui.separator();
                ui.label(RichText::new("Control alternatives").strong());
                for edge in &flow.transitions {
                    if edge.from == id {
                        let condition = if edge.condition.is_empty() {
                            "Continue"
                        } else {
                            &edge.condition
                        };
                        let target = flow.step(&edge.to).map(|s| s.name.as_str()).unwrap_or("?");
                        if ui.button(format!("{condition} -> {target}")).clicked() {
                            self.at.selection = Selection::Transition(edge.id.clone());
                        }
                    }
                }
                for id in contract_ids {
                    self.at.selection = Selection::Contract(id);
                }
            }
            Selection::Transition(id) => {
                let Some(edge) = flow.transition(&id) else {
                    self.at.selection = Selection::None;
                    return;
                };
                ui.heading(if edge.condition.is_empty() {
                    "Continue"
                } else {
                    &edge.condition
                });
                ui.monospace(&edge.id);
                ui.label(format!(
                    "{} -> {}",
                    flow.step(&edge.from)
                        .map(|s| s.name.as_str())
                        .unwrap_or("?"),
                    flow.step(&edge.to).map(|s| s.name.as_str()).unwrap_or("?")
                ));
                ui.small("Condition text is preserved from the source. This candidate does not evaluate it or grant permission to run work.");
                ui.separator();
                if self.at.tab == Tab::Control {
                    if ui.button("Show linked interface exchanges").clicked() {
                        self.set_tab(Tab::Interfaces);
                    }
                } else if ui.button("Show control transition").clicked() {
                    self.set_tab(Tab::Control);
                }
                ui.label(RichText::new("Information carried").strong());
                if let Some(original) = self.document.source_edge(&edge.source_flow) {
                    ui.label(&original.label);
                    let ids = original.contracts.clone();
                    for id in ids {
                        let c = self.document.project.contracts.iter().find(|c| c.id == id);
                        let title = c
                            .map(|c| format!("{}@{} · {}", c.id, c.version, c.name))
                            .unwrap_or(id.clone());
                        if ui.button(title).clicked() {
                            self.at.selection = Selection::Contract(id);
                        }
                    }
                }
                if !self.at.original {
                    ui.separator();
                    ui.label(format!("{} exact local exchange(s)", edge.exchanges.len()));
                    for id in &edge.exchanges {
                        if let Some(system) = self.document.project.system(&flow.system) {
                            if let Some(wire) = system.edges.iter().find(|e| e.id == *id) {
                                ui.collapsing(id, |ui| {
                                    ui.monospace(format!("{}\n  -> {}", wire.from.port, wire.to.port));
                                    ui.small("Exact endpoint identities, not an aggregated replacement edge.");
                                });
                            }
                        }
                    }
                }
            }
            Selection::Exchange(id) => {
                ui.heading("Exact interface exchange");
                ui.monospace(&id);
                if let Some(system) = self.document.project.system(&flow.system) {
                    if let Some(edge) = system.edges.iter().find(|e| e.id == id) {
                        ui.label(edge.label.as_deref().unwrap_or(""));
                        ui.label(RichText::new("Source port").strong());
                        ui.monospace(&edge.from.port);
                        ui.label(RichText::new("Destination port").strong());
                        ui.monospace(&edge.to.port);
                        let contract = self
                            .document
                            .project
                            .port(&flow.system, &edge.from)
                            .and_then(|p| p.contract.as_ref())
                            .cloned();
                        if let Some(c) = contract {
                            if ui.button(format!("Inspect {}", c)).clicked() {
                                self.at.selection = Selection::Contract(c.id);
                            }
                        }
                    }
                }
                if let Some(t) = flow.transitions.iter().find(|t| t.exchanges.contains(&id)) {
                    ui.separator();
                    ui.label(format!(
                        "Part of {} · {} exchange(s)",
                        t.id,
                        t.exchanges.len()
                    ));
                    if ui.button("Show control transition").clicked() {
                        self.set_tab(Tab::Control);
                    }
                }
            }
            Selection::Contract(id) => {
                let Some(c) = self.document.project.contracts.iter().find(|c| c.id == id) else {
                    self.at.selection = Selection::None;
                    return;
                };
                ui.heading(&c.name);
                ui.monospace(c.reference().to_string());
                ui.colored_label(
                    view::WARN,
                    "Unchanged migrated contract definition · draft schema",
                );
                ui.collapsing("Original purpose & migration notes", |ui| {
                    ui.label(&c.purpose);
                });
                ui.separator();
                ui.label(RichText::new("Exact structured definition").strong());
                let text = serde_json::to_string_pretty(&c.definition).unwrap_or_default();
                egui::ScrollArea::both()
                    .id_salt("contract-schema")
                    .max_height(450.0)
                    .show(ui, |ui| {
                        ui.monospace(text);
                    });
                ui.separator();
                ui.label("Used by these control transitions:");
                for edge in &flow.transitions {
                    if self
                        .document
                        .source_edge(&edge.source_flow)
                        .is_some_and(|s| s.contracts.contains(&id))
                    {
                        if ui.button(format!("{} · {}", edge.id, edge.label)).clicked() {
                            self.at.selection = Selection::Transition(edge.id.clone());
                        }
                    }
                }
            }
        }
    }
    fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(icon) = &self.icon {
                    ui.image((icon.id(), egui::vec2(28.0, 28.0)));
                }
                ui.heading("System Designer");
                ui.label(
                    RichText::new("CONTROL-FLOW STUDY")
                        .small()
                        .color(view::ACCENT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("AI instructions").clicked() {
                        self.prompt = true;
                    }
                    if ui.button("Export interfaces…").clicked() {
                        self.export_interfaces();
                    }
                    if ui.button("Copy study JSON").clicked() {
                        ctx.copy_text(model::STUDY.into());
                        self.status =
                            "Copied experimental study JSON—not a version-1 replacement packet."
                                .into();
                    }
                });
            });
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(!self.history.is_empty(), egui::Button::new("‹ Back"))
                    .clicked()
                {
                    self.back();
                }
                ui.label("S2 · Investigate uncertainty");
                if self.at.scope == 1 {
                    ui.label("/ Produce discriminating evidence");
                }
                if self.at.original {
                    ui.label("/ Original reference");
                }
                ui.separator();
                ui.label(
                    RichText::new("Read-only worked candidate · no Method adoption")
                        .small()
                        .color(view::WARN),
                );
            });
        });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.small(&self.status);
        });
        egui::SidePanel::left("navigation")
            .default_width(215.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.sidebar(ui));
            });
        egui::SidePanel::right("inspector")
            .default_width(345.0)
            .min_width(280.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.inspector(ui));
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.at.tab == Tab::Control, "Control Flow").clicked() { self.set_tab(Tab::Control); }
                if ui.selectable_label(self.at.tab == Tab::Interfaces, "Interfaces").clicked() { self.set_tab(Tab::Interfaces); }
                ui.separator();
                let key = (self.at.scope, self.at.original, self.at.tab);
                if ui.button("Fit").clicked() { self.cameras.remove(&key); }
                ui.checkbox(&mut self.lights, "Selected direction light").on_hover_text("Direction preview only; not live execution. Static arrows remain when off.");
            });
            let flow = self.flow();
            let m = flow.metric().map(|n| n.to_string()).unwrap_or_else(|_| "unavailable".into());
            let wires = self.document.project.system(&flow.system).map(|s| s.edges.len()).unwrap_or(0);
            ui.horizontal(|ui| {
                ui.strong(&flow.name);
                ui.small(format!("{} local steps · {} control transitions · M={m}", flow.local_steps(), flow.transitions.len()));
            });
            if self.at.tab == Tab::Interfaces {
                ui.small(format!("{wires} exact typed exchanges · select a channel to trace its control transition · export opens in the current designer"));
            } else {
                ui.small("One arrow per control transition; contract details stay in the inspector. Wheel zooms, drag background to pan.");
            }
            view::canvas(self, ui, &flow);
        });
        if self.prompt {
            let mut open = true;
            egui::Window::new("Embedded AI initialization instructions")
                .open(&mut open)
                .default_size([820.0, 650.0])
                .show(ctx, |ui| {
                    if ui.button("Copy instructions").clicked() {
                        ctx.copy_text(model::PROMPT.into());
                    }
                    ui.separator();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(model::PROMPT);
                    });
                });
            self.prompt = open;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.at.selection = Selection::None;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Backspace)) {
            self.back();
        }
    }
    fn capture(&mut self, ctx: &egui::Context) {
        let Some(capture) = &mut self.capture else {
            return;
        };
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            let name = [
                "01-control",
                "02-linked-interfaces",
                "03-subprocess",
                "04-original",
                "05-child-interfaces",
            ][capture.stage];
            let path = capture.dir.join(format!("{name}.ppm"));
            let mut bytes = format!("P6\n{} {}\n255\n", image.size[0], image.size[1]).into_bytes();
            for pixel in &image.pixels {
                bytes.extend_from_slice(&pixel.to_array()[..3]);
            }
            if let Err(e) = std::fs::write(path, bytes) {
                eprintln!("Screenshot failed: {e}");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                self.capture = None;
                return;
            }
            capture.stage += 1;
            capture.frames = 0;
            capture.requested = false;
            if capture.stage == 5 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                self.capture = None;
                return;
            }
            self.at = match capture.stage {
                1 => Location {
                    scope: 0,
                    original: false,
                    tab: Tab::Interfaces,
                    selection: Selection::Transition("S2.e3".into()),
                },
                2 => Location {
                    scope: 1,
                    original: false,
                    tab: Tab::Control,
                    selection: Selection::Step("step.S2.plan".into()),
                },
                3 => Location {
                    scope: 0,
                    original: true,
                    tab: Tab::Control,
                    selection: Selection::None,
                },
                _ => Location {
                    scope: 1,
                    original: false,
                    tab: Tab::Interfaces,
                    selection: Selection::Transition("S2.e6".into()),
                },
            };
        }
        if let Some(capture) = &mut self.capture {
            capture.frames += 1;
            if capture.frames > 8 && !capture.requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                capture.requested = true;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.show(ctx);
        self.capture(ctx);
    }
}
fn main() -> eframe::Result {
    let document = match Document::load() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Invalid candidate: {e}");
            std::process::exit(2);
        }
    };
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|s| s == "--check") {
        println!("Source fidelity and production interface validation: PASS");
        for f in &document.study.flows {
            println!(
                "{}: {} local steps, {} transitions, M={:?}",
                f.id,
                f.local_steps(),
                f.transitions.len(),
                f.metric()
            );
        }
        return Ok(());
    }
    let capture_dir = args
        .windows(2)
        .find(|a| a[0] == "--capture")
        .map(|a| PathBuf::from(&a[1]));
    if let Some(dir) = &capture_dir {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("Capture directory: {e}");
            std::process::exit(2);
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("System Designer — S2 control-flow study")
            .with_app_id("system-designer-flow-study")
            .with_icon(system_designer::ui::window_icon())
            .with_inner_size([1800.0, 1080.0])
            .with_min_inner_size([1150.0, 750.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "System Designer — S2 control-flow study",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            let mut style = (*cc.egui_ctx.style()).clone();
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.visuals.panel_fill = Color32::from_rgb(20, 25, 33);
            cc.egui_ctx.set_style(style);
            let icon = system_designer::ui::window_icon();
            let texture = cc.egui_ctx.load_texture(
                "approved-mark",
                egui::ColorImage::from_rgba_unmultiplied(
                    [icon.width as usize, icon.height as usize],
                    &icon.rgba,
                ),
                Default::default(),
            );
            let mut app = App::new(document);
            app.icon = Some(texture);
            app.capture = capture_dir.map(|dir| Capture {
                dir,
                stage: 0,
                frames: 0,
                requested: false,
            });
            Ok(Box::new(app))
        }),
    )
}
