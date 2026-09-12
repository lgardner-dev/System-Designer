#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
//! Native self-inspection; read-only design review, not a CFG execution engine.
#[path = "../../src/ui/canvas/geometry.rs"]
#[allow(dead_code)]
mod geometry;
mod model;
#[cfg(test)]
mod tests;
mod view;
use eframe::egui::{self, Color32, RichText};
use model::{Atlas, Behavior, Kind};
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Tab {
    Control,
    Interfaces,
}
impl Tab {
    fn tag(self) -> &'static str {
        if self == Self::Control {
            "control"
        } else {
            "interfaces"
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Selection {
    None,
    Step(String),
    Component(String),
    Transition(String),
    Edge(String),
    Port(String),
}
#[derive(Clone)]
struct Location {
    owner: String,
    tab: Tab,
    selection: Selection,
}
struct Capture {
    dir: PathBuf,
    index: usize,
    frames: usize,
    requested: bool,
    manifest: Vec<serde_json::Value>,
}
struct App {
    atlas: Atlas,
    owner: String,
    tab: Tab,
    selection: Selection,
    history: Vec<Location>,
    cameras: BTreeMap<(String, Tab), view::Camera>,
    lights: u8,
    prompt: bool,
    filter: String,
    capture: Option<Capture>,
    status: String,
    icon: Option<egui::TextureHandle>,
    tree_scroll: bool,
}
impl App {
    fn new(atlas: Atlas) -> Self {
        Self{atlas,owner:"@root".into(),tab:Tab::Control,selection:Selection::None,history:vec![],cameras:BTreeMap::new(),lights:0,prompt:false,filter:String::new(),capture:None,status:"Double-click a component to enter; select a primitive step to inspect its exact rule.".into(),icon:None,tree_scroll:true}
    }
    fn navigate(&mut self, owner: &str) {
        if self.atlas.scope(owner).is_some() {
            self.history.push(Location {
                owner: self.owner.clone(),
                tab: self.tab,
                selection: self.selection.clone(),
            });
            self.owner = owner.into();
            self.selection = Selection::None;
            self.tree_scroll = true;
        }
    }
    fn back(&mut self) {
        if let Some(p) = self.history.pop() {
            self.owner = p.owner;
            self.tab = p.tab;
            self.selection = p.selection;
            self.tree_scroll = true;
        }
    }
    fn tree(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("APPLICATION DESIGN")
                .color(view::ACCENT)
                .small(),
        );
        ui.add_space(10.0);
        ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Find component"));
        ui.add_space(8.0);
        let mut stack = vec![("@root".to_owned(), 0usize)];
        let mut chosen = None;
        while let Some((owner, depth)) = stack.pop() {
            if let Some(system) = self.atlas.system(&owner) {
                for n in system.nodes.iter().rev() {
                    stack.push((n.id.clone(), depth + 1));
                }
            }
            let name = self.atlas.name(&owner);
            if !self.filter.is_empty()
                && !name.to_lowercase().contains(&self.filter.to_lowercase())
                && !owner.to_lowercase().contains(&self.filter.to_lowercase())
            {
                continue;
            }
            ui.horizontal(|ui| {
                ui.add_space(depth as f32 * 10.0);
                let compound = self.atlas.system(&owner).is_some();
                let label = format!("{} {}", if compound { "+" } else { "·" }, name);
                let r = ui
                    .selectable_label(owner == self.owner, label)
                    .on_hover_text(&owner);
                if owner == self.owner && self.tree_scroll {
                    r.scroll_to_me(Some(egui::Align::Center));
                    self.tree_scroll = false;
                }
                if r.clicked() {
                    chosen = Some(owner.clone());
                }
            });
        }
        if let Some(owner) = chosen {
            self.navigate(&owner);
        }
    }
    fn inspector(&mut self, ui: &mut egui::Ui, b: &Behavior) {
        ui.label(
            RichText::new("SCOPE / PRIMITIVE INSPECTOR")
                .small()
                .color(view::ACCENT),
        );
        ui.add_space(10.0);
        match &self.selection.clone() {
            Selection::Step(id) => {
                if let Some(s) = b.step(id) {
                    ui.heading(&s.label);
                    ui.monospace(&s.id);
                    ui.label(if s.kind == Kind::Call {
                        "Subprocess reference"
                    } else if s.kind == Kind::Decision {
                        "Explicit question / predicate"
                    } else {
                        "Local operation"
                    });
                    ui.separator();
                    ui.label(RichText::new("Rule").strong());
                    ui.label(&s.rule);
                    if !s.inputs.is_empty() {
                        ui.label(RichText::new("Inputs").strong());
                        ui.label(&s.inputs);
                    }
                    if !s.outputs.is_empty() {
                        ui.label(RichText::new("Outputs").strong());
                        ui.label(&s.outputs);
                    }
                    if let Some(target) = &s.target {
                        ui.monospace(target);
                        if ui.button("Enter this component").clicked() {
                            self.navigate(target);
                        }
                        if ui.button("Locate in Interfaces").clicked() {
                            self.tab = Tab::Interfaces;
                            self.selection = Selection::Component(target.clone());
                        }
                    }
                    if let Some(source) = &s.source {
                        ui.separator();
                        ui.monospace(&source.path);
                        ui.label(&source.symbol);
                    }
                }
            }
            Selection::Component(id) => {
                let owner = if id == "@body" {
                    self.owner.clone()
                } else {
                    id.clone()
                };
                if let Some((_, n)) = self.atlas.project.node(&owner) {
                    ui.heading(&n.name);
                    ui.monospace(&n.id);
                    ui.label(&n.purpose);
                }
                if owner != self.owner {
                    if ui.button("Enter component scope").clicked() {
                        self.navigate(&owner);
                    }
                }
                if let Some(flow_step) = b.steps.iter().find(|s| s.target.as_ref() == Some(&owner))
                {
                    if ui.button("Locate in Control Flow").clicked() {
                        self.tab = Tab::Control;
                        self.selection = Selection::Step(flow_step.id.clone());
                    }
                }
            }
            Selection::Transition(id) => {
                if let Some(t) = b.transitions.iter().find(|x| x.id == *id) {
                    ui.heading(if t.label.is_empty() {
                        "Continue"
                    } else {
                        &t.label
                    });
                    ui.monospace(id);
                    ui.label(format!("{}  >  {}", t.from, t.to));
                    ui.separator();
                    ui.label(format!(
                        "{} explicitly linked interface exchange(s)",
                        t.exchanges.len()
                    ));
                    if t.exchanges.is_empty() {
                        ui.small("No direct exchange has been asserted on this control transition. Sequence alone does not create a data channel.");
                    }
                    for id in &t.exchanges {
                        if ui.button(id).clicked() {
                            self.tab = Tab::Interfaces;
                            self.selection = Selection::Edge(id.clone());
                        }
                    }
                }
            }
            Selection::Edge(id) => {
                if let Some(sys) = self.atlas.system(&self.owner) {
                    if let Some(e) = sys.edges.iter().find(|e| e.id == *id) {
                        ui.heading("Exact interface connection");
                        ui.monospace(id);
                        ui.label(e.label.as_deref().unwrap_or(""));
                        ui.label(RichText::new("From").strong());
                        ui.monospace(&e.from.port);
                        ui.label(RichText::new("To").strong());
                        ui.monospace(&e.to.port);
                        if let Some(r) = self
                            .atlas
                            .project
                            .port(&sys.id, &e.from)
                            .and_then(|p| p.contract.as_ref())
                        {
                            if let Some(c) = self.atlas.project.contract(r) {
                                ui.separator();
                                ui.heading(c.reference().to_string());
                                ui.label(&c.purpose);
                                ui.collapsing("Exact schema", |ui| {
                                    ui.monospace(
                                        serde_json::to_string_pretty(&c.definition)
                                            .unwrap_or_default(),
                                    );
                                });
                            }
                        }
                        for t in b.transitions.iter().filter(|t| t.exchanges.contains(id)) {
                            if ui
                                .button(format!("Show control transition: {}", t.label))
                                .clicked()
                            {
                                self.tab = Tab::Control;
                                self.selection = Selection::Transition(t.id.clone());
                            }
                        }
                    }
                }
            }
            Selection::Port(id) => {
                ui.heading("Authoritative physical port");
                ui.monospace(id);
                if let Some((n, p)) = self
                    .atlas
                    .project
                    .systems
                    .iter()
                    .flat_map(|s| &s.nodes)
                    .find_map(|n| n.ports.iter().find(|p| p.id == *id).map(|p| (n, p)))
                {
                    ui.label(&n.name);
                    ui.label(&p.name);
                    ui.label(p.direction.label());
                    if let Some(c) = &p.contract {
                        ui.heading(c.to_string());
                        if let Some(def) = self.atlas.project.contract(c) {
                            ui.label(&def.purpose);
                        }
                    }
                }
                ui.small("A child/primitive boundary borrows this same identity. It is not a second editable interface.");
            }
            Selection::None => {
                ui.heading(self.atlas.name(&b.owner));
                ui.monospace(&b.owner);
                ui.add_space(8.0);
                ui.label(&b.notes);
                ui.separator();
                ui.label(RichText::new("Local control structure").strong());
                ui.label(format!(
                    "{} actions / questions\n{} transitions\nCyclomatic M = {}",
                    b.work_steps(),
                    b.transitions.len(),
                    b.metric()
                ));
                ui.small("Explicit entry/return markers are not architecture components. M counts independent graph paths, not feasible executions.");
                if let Some(stop) = &b.primitive_stop {
                    ui.separator();
                    ui.label(
                        RichText::new("Why this is a leaf")
                            .strong()
                            .color(view::ACCENT),
                    );
                    ui.label(stop);
                } else {
                    ui.separator();
                    ui.label(
                        "Decomposed responsibility. Enter a child from either layer or the tree.",
                    );
                }
            }
        }
        ui.add_space(12.0);
        ui.separator();
        ui.label(RichText::new("Source mapping").strong());
        for s in &b.sources {
            ui.small(&s.path);
            ui.monospace(&s.symbol);
        }
        ui.small(format!("Inspected baseline: {}", &self.atlas.baseline[..8]));
        if !b.planned.is_empty() {
            ui.separator();
            ui.label(
                RichText::new("Planned — not implemented")
                    .color(view::WARN)
                    .strong(),
            );
            for x in &b.planned {
                ui.label(x);
                ui.add_space(5.0);
            }
        }
    }
    fn show(&mut self, ctx: &egui::Context) {
        let b = self.atlas.scope(&self.owner).expect("valid scope").clone();
        egui::TopBottomPanel::top("title").show(ctx,|ui|{
   ui.horizontal(|ui|{if let Some(t)=&self.icon{ui.image((t.id(),egui::vec2(26.0,26.0)));}ui.heading("System Designer");ui.label("Self-design atlas");ui.separator();ui.colored_label(view::WARN,"Read-only design review");
    if ui.add_enabled(!self.history.is_empty(),egui::Button::new("Back")).clicked(){self.back();}
    if ui.button("Initialization prompt").clicked(){self.prompt=true;}
    if ui.button("Copy complete atlas").clicked(){ctx.copy_text(model::ATLAS.into());self.status="Copied the complete paired design, not a view-filtered replacement.".into();}
    if ui.button("Export interfaces").clicked(){if let Some(path)=rfd::FileDialog::new().set_file_name("system-designer.project.json").save_file(){self.status=match std::fs::write(&path,system_designer::APPLICATION_DESIGN){Ok(())=>"Exported version-1 interface projection only; behavior remains in the atlas.".into(),Err(e)=>format!("Export failed: {e}")};}}
   });
   ui.horizontal_wrapped(|ui|{let path=self.atlas.lineage(&self.owner);for(i,id)in path.iter().enumerate(){if i>0{ui.small(">");}if ui.link(self.atlas.name(id)).clicked(){self.navigate(id);}}});
  });
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.small(&self.status);
        });
        egui::SidePanel::left("tree")
            .default_width(255.0)
            .min_width(210.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.tree(ui));
            });
        egui::SidePanel::right("inspect")
            .default_width(330.0)
            .min_width(260.0)
            .resizable(true)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.inspector(ui, &b));
            });
        egui::CentralPanel::default().show(ctx,|ui|{
   ui.horizontal(|ui|{ui.selectable_value(&mut self.tab,Tab::Control,"Control Flow");ui.selectable_value(&mut self.tab,Tab::Interfaces,"Interfaces");ui.separator();if ui.button("Fit").clicked(){self.cameras.remove(&(self.owner.clone(),self.tab));}if ui.button("Clear selection").clicked(){self.selection=Selection::None;}
    egui::ComboBox::from_id_salt("lights").selected_text(match self.lights{0=>"Lights: Off",1=>"Lights: Selection",_=>"Lights: All"}).show_ui(ui,|ui|{ui.selectable_value(&mut self.lights,0,"Off");ui.selectable_value(&mut self.lights,1,"Selection");ui.selectable_value(&mut self.lights,2,"All");});ui.small("Direction preview, not execution");
   });
   ui.add_space(7.0);ui.label(RichText::new(if self.tab==Tab::Control{b.title.clone()}else{format!("{} — interfaces at this scope",self.atlas.name(&b.owner))}).strong());
   ui.small(if self.tab==Tab::Control{"Rectangle: operation · diamond: question · double-sided rectangle: child call · pill: entry/outcome"}else if self.atlas.system(&b.owner).is_none(){"Leaf interface boundary: exact owner ports, no invented child components"}else{"Exact typed connections · enter any component to inspect its internal scope"});ui.add_space(5.0);view::canvas(self,ui,&b);
  });
        if self.prompt {
            egui::Window::new("Embedded design initialization")
                .default_width(800.0)
                .default_height(600.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Copy prompt").clicked() {
                            ctx.copy_text(model::PROMPT.into());
                        }
                        if ui.button("Close").clicked() {
                            self.prompt = false;
                        }
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(model::PROMPT);
                    });
                });
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selection = Selection::None;
            self.prompt = false;
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Backspace)) {
            self.back();
        }
    }
    fn capture(&mut self, ctx: &egui::Context) {
        let Some(c) = &mut self.capture else {
            return;
        };
        let image = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            let owner = self.owner.replace('@', "");
            let name = format!("{:03}-{}-{}.ppm", c.index + 1, owner, self.tab.tag());
            let path = c.dir.join(&name);
            let mut bytes = format!("P6\n{} {}\n255\n", image.size[0], image.size[1]).into_bytes();
            for px in &image.pixels {
                bytes.extend_from_slice(&px.to_array()[..3]);
            }
            if let Err(e) = std::fs::write(path, bytes) {
                eprintln!("Capture failed: {e}");
                std::process::exit(3);
            }
            c.manifest.push(serde_json::json!({"file":name,"owner":self.owner,"layer":self.tab.tag(),"name":self.atlas.name(&self.owner),"lineage":self.atlas.lineage(&self.owner)}));
            c.index += 1;
            c.frames = 0;
            c.requested = false;
            if c.index == self.atlas.scopes.len() * 2 {
                let manifest = serde_json::to_vec_pretty(&c.manifest).expect("manifest");
                if let Err(e) = std::fs::write(c.dir.join("manifest.json"), manifest) {
                    eprintln!("Manifest failed: {e}");
                    std::process::exit(3);
                }
                self.capture = None;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
            self.owner = self.atlas.scopes[c.index / 2].owner.clone();
            self.tab = if c.index % 2 == 0 {
                Tab::Control
            } else {
                Tab::Interfaces
            };
            self.selection = Selection::None;
            self.tree_scroll = true;
        }
        if let Some(c) = &mut self.capture {
            c.frames += 1;
            if c.frames >= 4 && !c.requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
                c.requested = true;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(80));
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
    let a = Atlas::load().unwrap_or_else(|e| {
        eprintln!("Invalid self-design: {e}");
        std::process::exit(2)
    });
    let args: Vec<_> = std::env::args().collect();
    if args.iter().any(|x| x == "--check") {
        println!(
            "Self-design atlas: {} scopes; {} components; {} systems; all invariant/reference checks passed",
            a.scopes.len(),
            a.project
                .systems
                .iter()
                .map(|s| s.nodes.len())
                .sum::<usize>(),
            a.project.systems.len()
        );
        return Ok(());
    }
    let capture = args
        .windows(2)
        .find(|x| x[0] == "--capture")
        .map(|x| PathBuf::from(&x[1]));
    if let Some(dir) = &capture {
        std::fs::create_dir_all(dir).unwrap_or_else(|e| {
            eprintln!("Capture directory: {e}");
            std::process::exit(2)
        });
    }
    let owner = args
        .windows(2)
        .find(|x| x[0] == "--scope")
        .map(|x| x[1].clone());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("System Designer — self-design atlas")
            .with_app_id("system-designer-self-design-atlas")
            .with_icon(system_designer::ui::window_icon())
            .with_inner_size([1920.0, 1160.0])
            .with_min_inner_size([1150.0, 750.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "System Designer — self-design atlas",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            let mut st = (*cc.egui_ctx.style()).clone();
            st.spacing.item_spacing = egui::vec2(7.0, 7.0);
            st.visuals.panel_fill = Color32::from_rgb(20, 26, 35);
            cc.egui_ctx.set_style(st);
            let mut app = App::new(a);
            if let Some(id) = owner {
                if app.atlas.scope(&id).is_some() {
                    app.owner = id;
                }
            }
            if let Some(dir) = capture {
                app.owner = "@root".into();
                app.capture = Some(Capture {
                    dir,
                    index: 0,
                    frames: 0,
                    requested: false,
                    manifest: vec![],
                });
            }
            let icon = system_designer::ui::window_icon();
            app.icon = Some(cc.egui_ctx.load_texture(
                "atom",
                egui::ColorImage::from_rgba_unmultiplied(
                    [icon.width as usize, icon.height as usize],
                    &icon.rgba,
                ),
                Default::default(),
            ));
            Ok(Box::new(app))
        }),
    )
}
