use super::*;
use crate::edit;
impl Designer {
    pub(super) fn inspector(&mut self, ui: &mut egui::Ui) {
        super::canvas::focus::details(self, ui);
        let p = self.store.snapshot();
        match self.selected.clone() {
            Selection::Node(id) => {
                if let Some((_, node)) = p.node(&id) {
                    ui.heading(&node.name);
                    ui.monospace(&node.id);
                    ui.label(node.kind.label());
                    ui.separator();
                    ui.label(&node.purpose);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Edit component").clicked() {
                            self.dialog = Some(Dialog::Node(node.clone()));
                        }
                        if let Some(child) = &node.child {
                            if ui.button("Enter >").clicked() {
                                self.navigate(child.clone());
                            }
                        } else if ui.button("Decompose").clicked() {
                            match edit::create_child(&p, &id) {
                                Ok((q, child)) => {
                                    self.publish("Create internal system", Ok(q));
                                    self.navigate(child);
                                }
                                Err(e) => self.error = Some(e.to_string()),
                            }
                        }
                    });
                    ui.separator();
                    ui.strong("Public interface");
                    for r in &node.ports {
                        ui.push_id(&r.id,|ui|{
                            ui.group(|ui|{
                                ui.label(format!("{} · {}",r.direction.label(),r.name));
                                ui.monospace(r.contract.as_ref().map(ToString::to_string).unwrap_or_else(||"Unassigned".into()));
                                ui.horizontal(|ui|{
                                    if let Some(reference)=&r.contract{
                                        if ui.small_button("Inspect type").clicked(){
                                            if let Some(c)=p.contract(reference){
                                                self.dialog=Some(Dialog::Contract(ContractDialog::edit(c)));
                                            }
                                        }
                                    }
                                    if ui.small_button("Remove port").clicked(){
                                        let refs=edit::port_edges(&p,&r.id);
                                        self.dialog=Some(Dialog::Confirm{
                                            message:format!("Remove {} and {} attached connection(s), including mirrored child-boundary wires?",r.name,refs.len()),
                                            action:ConfirmAction::Port(id.clone(),r.id.clone())
                                        });
                                    }
                                });
                            });
                        });
                    }
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("+ Input").clicked() {
                            self.publish("Add input", edit::add_port(&p, &id, Direction::In));
                        }
                        if ui.button("+ Output").clicked() {
                            self.publish("Add output", edit::add_port(&p, &id, Direction::Out));
                        }
                    });
                    if node.child.is_some() && ui.button("Remove internal system…").clicked() {
                        self.dialog=Some(Dialog::Confirm{
                            message:"Delete this component's entire internal subtree? Its public ports and parent connections remain.".into(),
                            action:ConfirmAction::Child(id.clone())
                        });
                    }
                    ui.separator();
                    if ui.button("Delete component…").clicked() {
                        self.ask_delete();
                    }
                }
            }
            Selection::Edge(id) => {
                if let Some(edge) = p
                    .system(&self.current)
                    .and_then(|s| s.edges.iter().find(|e| e.id == id))
                {
                    ui.heading("Connection");
                    ui.monospace(&edge.id);
                    for (title, ep) in [("From", &edge.from), ("To", &edge.to)] {
                        let name = ep
                            .node
                            .as_ref()
                            .and_then(|id| p.node(id))
                            .map(|(_, n)| n.name.as_str())
                            .unwrap_or("Boundary");
                        ui.label(format!(
                            "{title}: {name} · {}",
                            p.port(&self.current, ep)
                                .map(|r| r.name.as_str())
                                .unwrap_or("?")
                        ));
                    }
                    ui.separator();
                    ui.strong("Carried contract");
                    ui.monospace(
                        p.port(&self.current, &edge.from)
                            .and_then(|r| r.contract.as_ref())
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    );
                    ui.label(edge.label.as_deref().unwrap_or(""));
                    if ui.button("Change contract / endpoints").clicked() {
                        self.dialog = Some(Dialog::Connection(ConnectionDialog::new(
                            &p,
                            &self.current,
                            None,
                            Some(&id),
                        )));
                    }
                    if ui.button("Delete connection…").clicked() {
                        self.ask_delete();
                    }
                }
            }
            Selection::Boundary(pid) => {
                if let Some((parent, owner)) = p.owner(&self.current) {
                    ui.heading("Component boundary");
                    ui.label(format!("Owned by {}", owner.name));
                    if let Some(port) = owner.ports.iter().find(|r| r.id == pid) {
                        ui.strong(&port.name);
                        ui.label(port.direction.label());
                        ui.monospace(
                            port.contract
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_else(|| "Unassigned".into()),
                        );
                    }
                    ui.label("This is the owning component's actual port, not a separately editable copy.");
                    for external in p.external_connections(&self.current) {
                        if external["port"].as_str() == Some(&pid) {
                            for link in external["links"].as_array().into_iter().flatten() {
                                ui.label(format!(
                                    "Outside: {} · {}",
                                    link["name"].as_str().unwrap_or(""),
                                    link["portName"].as_str().unwrap_or("")
                                ));
                            }
                        }
                    }
                    if ui.button("Edit owner in parent").clicked() {
                        self.navigate(parent.id.clone());
                        self.selected = Selection::Node(owner.id.clone());
                        self.dialog = Some(Dialog::Node(owner.clone()));
                    }
                }
            }
            Selection::None => {
                let owner = p.owner(&self.current);
                ui.heading(owner.map(|(_, n)| n.name.as_str()).unwrap_or(&p.name));
                ui.monospace(&self.current);
                ui.label(owner.map(|(_, n)| n.purpose.as_str()).unwrap_or(&p.purpose));
                ui.separator();
                if let Some((parent, n)) = owner {
                    if ui.button("Inspect owning component").clicked() {
                        self.navigate(parent.id.clone());
                        self.selected = Selection::Node(n.id.clone());
                    }
                    ui.strong("Inherited context");
                    for a in p.ancestors(&self.current) {
                        ui.label(format!(
                            "{}: {}",
                            a["name"].as_str().unwrap_or(""),
                            a["purpose"].as_str().unwrap_or("")
                        ));
                    }
                } else if ui.button("Edit project purpose").clicked() {
                    self.dialog = Some(Dialog::Project {
                        name: p.name.clone(),
                        purpose: p.purpose.clone(),
                    });
                }
                ui.separator();
                ui.label("Drag between ports to define a connection. Double-click a wire to change its contract. Double-click a component to enter its internals.");
                ui.label("AI handoff includes a built-in initialization prompt and component, level, or subtree exchange.");
                ui.label("Around eight immediate components is a useful design budget, not a structural limit.");
            }
        }
    }
    pub(super) fn ask_delete(&mut self) {
        self.dialog=match &self.selected{
            Selection::Node(id)=>Some(Dialog::Confirm{
                message:"Delete this component, all its internal systems, and attached connections? Undo remains available in this session.".into(),
                action:ConfirmAction::Node(id.clone())
            }),
            Selection::Edge(id)=>Some(Dialog::Confirm{
                message:"Delete the selected connection? Port contract bindings will be preserved.".into(),
                action:ConfirmAction::Edge(id.clone())
            }),
            _=>None,
        };
    }
    pub(super) fn show_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        let mut keep = true;
        let mut next = None;
        egui::Modal::new(egui::Id::new("designer_modal")).show(ctx,|ui|{
            ui.set_width((ctx.available_rect().width()-80.0).clamp(440.0,800.0));
            egui::ScrollArea::vertical().max_height((ctx.available_rect().height()-110.0).max(260.0)).show(ui,|ui|{
                match &mut dialog{
                    Dialog::Project{
                        name,
                        purpose
                    }
                    =>{
                        ui.heading("Project purpose");
                        ui.label("Name");
                        ui.text_edit_singleline(name);
                        ui.label("Outcome, constraints, and design intent");
                        ui.add(egui::TextEdit::multiline(purpose).desired_rows(7).desired_width(f32::INFINITY));
                        ui.horizontal(|ui|{
                            if ui.button("Save project details").clicked(){
                                let name=name.clone();
                                let purpose=purpose.clone();
                                let q=edit::candidate(self.store.project(),|p|{
                                    p.name=name;
                                    p.purpose=purpose;
                                    Ok(())
                                });
                                self.publish("Edit project",q);
                                keep=self.error.is_some();
                            }
                            if ui.button("Cancel").clicked(){
                                keep=false;
                            }
                        });
                    }
                    Dialog::Node(node)=>{
                        ui.heading("Edit component");
                        ui.monospace(&node.id);
                        ui.label("Name");
                        ui.text_edit_singleline(&mut node.name);
                        ui.label("Purpose / design intent");
                        ui.add(egui::TextEdit::multiline(&mut node.purpose).desired_rows(5).desired_width(f32::INFINITY));
                        egui::ComboBox::from_id_salt("kind").selected_text(node.kind.label()).show_ui(ui,|ui|{
                            for k in Kind::ALL{
                                ui.selectable_value(&mut node.kind,k,k.label());
                            }
                        });
                        ui.separator();
                        ui.strong("Public ports");
                        let mut remove=None;
                        for(i,port)in node.ports.iter_mut().enumerate(){
                            ui.push_id(&port.id,|ui|{
                                ui.group(|ui|{
                                    ui.text_edit_singleline(&mut port.name);
                                    ui.small(&port.id);
                                    let connected=!edit::port_edges(self.store.project(),&port.id).is_empty();
                                    ui.add_enabled_ui(!connected,|ui|{
                                        ui.horizontal(|ui|{
                                            egui::ComboBox::from_id_salt("direction").selected_text(port.direction.label()).show_ui(ui,|ui|{
                                                ui.selectable_value(&mut port.direction,Direction::In,"Input");
                                                ui.selectable_value(&mut port.direction,Direction::Out,"Output");
                                            });
                                            egui::ComboBox::from_id_salt("type").selected_text(port.contract.as_ref().map(ToString::to_string).unwrap_or_else(||"Unassigned".into())).show_ui(ui,|ui|{
                                                ui.selectable_value(&mut port.contract,None,"Unassigned");
                                                for c in &self.store.project().contracts{
                                                    let r=c.reference();
                                                    ui.selectable_value(&mut port.contract,Some(r.clone()),r.to_string());
                                                }
                                            });
                                            if ui.small_button("Remove").clicked(){
                                                remove=Some(i);
                                            }
                                        });
                                    });
                                    if connected{
                                        ui.small("Connected: change type through its connection; remove this port in the inspector to remove all affected wires.");
                                    }
                                });
                            });
                        }
                        if let Some(i)=remove{
                            node.ports.remove(i);
                        }
                        ui.horizontal(|ui|{
                            for dir in[Direction::In,Direction::Out]{
                                if ui.button(format!("+ {}",dir.label())).clicked(){
                                    node.ports.push(Port{
                                        id:format!("port.{}",uuid::Uuid::new_v4()),
                                        name:dir.label().into(),
                                        direction:dir,
                                        contract:None
                                    });
                                }
                            }
                        });
                        ui.separator();
                        ui.horizontal(|ui|{
                            if ui.button("Save component").clicked(){
                                self.publish("Edit component",edit::save_node(self.store.project(),node.clone()));
                                keep=self.error.is_some();
                            }
                            if ui.button("Cancel").clicked(){
                                keep=false;
                            }
                        });
                    }
                    Dialog::Connection(d)=>{
                        keep=!self.connection_form(ui,d);
                    }
                    Dialog::Catalog=>{
                        ui.heading("Contract catalog");
                        ui.label("Definitions are shared by exact ID and version. New projects have no defaults.");
                        if ui.button("+ Define contract").clicked(){
                            next=Some(Dialog::Contract(ContractDialog::new()));
                            keep=false;
                        }
                        for c in self.store.project().contracts.clone(){
                            ui.group(|ui|{
                                ui.horizontal_wrapped(|ui|{
                                    ui.strong(c.reference().to_string());
                                    ui.label(&c.name);
                                    if ui.button("Edit").clicked(){
                                        next=Some(Dialog::Contract(ContractDialog::edit(&c)));
                                        keep=false;
                                    }
                                    if ui.button("New version").clicked(){
                                        let mut draft=c.clone();
                                        draft.version=self.store.project().contracts.iter().filter(|x|x.id==c.id).map(|x|x.version).max().unwrap_or(0)+1;
                                        next=Some(Dialog::Contract(ContractDialog{
                                            draft,
                                            editing:None,
                                            consent:false
                                        }));
                                        keep=false;
                                    }
                                    if ui.button("Delete").clicked(){
                                        next=Some(Dialog::Confirm{
                                            message:format!("Delete {}? Referenced types cannot be deleted.",c.reference()),
                                            action:ConfirmAction::Contract(c.reference())
                                        });
                                        keep=false;
                                    }
                                });
                                ui.label(&c.purpose);
                            });
                        }
                        if ui.button("Close").clicked(){
                            keep=false;
                        }
                    }
                    Dialog::Contract(d)=>{
                        keep=!self.contract_form(ui,d);
                    }
                    Dialog::Handoff(d)=>{
                        keep=!self.handoff_form(ui,d,ctx);
                    }
                    Dialog::Confirm{
                        message,
                        action
                    }
                    =>{
                        ui.heading("Confirm change");
                        ui.label(message.as_str());
                        ui.horizontal(|ui|{
                            if ui.button("Apply change").clicked(){
                                let p=self.store.project();
                                let result=match action.clone(){
                                    ConfirmAction::Node(id)=>edit::delete_node(p,&id),
                                    ConfirmAction::Child(id)=>edit::delete_child(p,&id),
                                    ConfirmAction::Port(nid,pid)=>edit::delete_port(p,&nid,&pid),
                                    ConfirmAction::Edge(id)=>edit::delete_edge(p,&self.current,&id),
                                    ConfirmAction::Contract(r)=>edit::delete_contract(p,&r)
                                };
                                self.publish("Delete selection",result);
                                keep=self.error.is_some();
                            }
                            if ui.button("Cancel").clicked(){
                                keep=false;
                            }
                        });
                    }
                    Dialog::Unsaved(action)=>{
                        ui.heading("Keep your changes?");
                        ui.label("This project has unsaved changes. Saving must succeed before continuing.");
                        ui.horizontal(|ui|{
                            if ui.button("Save and continue").clicked()&&self.save(false){
                                self.load(action.clone(),ctx);
                                keep=false;
                            }
                            if ui.button("Discard and continue").clicked(){
                                self.load(action.clone(),ctx);
                                keep=false;
                            }
                            if ui.button("Cancel").clicked(){
                                keep=false;
                            }
                        });
                    }
                    Dialog::Recovery=>{
                        ui.heading("Recovery copies");
                        ui.label("Restore opens an unsaved copy. Original project files and other recovery copies are not replaced or deleted. A copy may belong to another running instance.");
                        for path in self.recoveries.clone(){
                            ui.group(|ui|{
                                ui.small(path.display().to_string());
                                match storage::read_recovery(&path){
                                    Ok(r)=>{
                                        ui.label(&r.project.name);
                                        if let Some(original)=r.original_path{
                                            ui.small(format!("Original: {}",original.display()));
                                        }
                                        if ui.button("Restore as unsaved copy").clicked(){
                                            if self.dirty(){
                                                next=Some(Dialog::Unsaved(LoadAction::Restore(path.clone())));
                                            }else{
                                                self.load(LoadAction::Restore(path.clone()),ctx);
                                            }
                                            keep=false;
                                        }
                                    }
                                    Err(e)=>{
                                        ui.colored_label(egui::Color32::LIGHT_RED,format!("Cannot recover: {e}. The original bytes are preserved."));
                                    }
                                }
                            });
                        }
                        if self.recoveries.is_empty(){
                            ui.label("No recovery copies found.");
                        }
                        if ui.button("Close").clicked(){
                            keep=false;
                        }
                    }
                    Dialog::Help=>{
                        ui.heading("System Designer — native Rust");
                        ui.label("A local editor for recursive components, typed ports, and parent-owned connections. It does not execute workflows or call AI services.");
                        ui.separator();
                        ui.strong("Design");
                        ui.label("Add a component, name its purpose, and add input/output ports. Drag a wire in either direction. Choose or define its contract explicitly. Double-click a wire to edit it; double-click a component to enter its internals. Middle-drag or background-drag pans. Scroll zooms; Fit frames the current level.");
                        ui.strong("Keep work");
                        ui.label("Ctrl+S saves the actual project file. Save As creates a new location. A .bak file holds the prior valid saved version. Recovery snapshots are separate and are not a backup strategy. Close, New, and Open guard unsaved changes.");
                        ui.strong("AI collaboration");
                        ui.label("AI handoff → Initialize chat contains the entire prompt. Export the smallest scope, then load returned JSON at the same level. Validate candidate does not change the project. Apply revalidates before a single undoable publication.");
                        ui.strong("Explore this app");
                        ui.label("File → Open application design loads the application's own component tree from an embedded resource, as an unsaved copy. New projects remain blank.");
                        ui.strong("Keys");
                        ui.label("Ctrl/Cmd+S save · Ctrl/Cmd+O open · Ctrl/Cmd+Z undo · Ctrl/Cmd+Y or Ctrl/Cmd+Shift+Z redo · Delete removes selected component/wire with confirmation · Escape cancels a gesture/dialog.");
                        if ui.button("Copy initialization prompt").clicked(){
                            ctx.copy_text(crate::INITIALIZATION.into());
                        }
                        if ui.button("Close").clicked(){
                            keep=false;
                        }
                    }
                }
                if let Some(error)=&self.error{
                    ui.separator();
                    ui.colored_label(egui::Color32::LIGHT_RED,error);
                }
            });
        });
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            keep = false;
        }
        if let Some(next) = next {
            self.dialog = Some(next);
        } else if keep {
            self.dialog = Some(dialog);
        }
    }
}
