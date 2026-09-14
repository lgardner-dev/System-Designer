use super::Action;
use crate::edit;
use crate::ui::*;
impl Designer {
    pub(in crate::ui) fn show_dialog(&mut self, ctx: &egui::Context) {
        let Some(mut dialog) = self.dialog.take() else {
            return;
        };
        let mut keep = true;
        let mut next = None;
        let popup_was_open = egui::Popup::is_any_open(ctx);
        egui::Modal::new(egui::Id::new("designer_modal")).show(ctx,|ui|{
            // A reused Modal area remembers its previous small size. Give the
            // scrolling form a viewport based on the whole window, independently
            // of the last dialog and the workspace panels drawn before it.
            let size=ctx.content_rect().size();
            ui.set_width((size.x-40.0).clamp(180.0,800.0));
            ui.spacing_mut().interact_size.y=ui.spacing().interact_size.y.max(24.0);
            let height=(size.y-40.0).clamp(120.0,900.0);
            ui.set_height(height);
            let status=match &dialog {
                Dialog::Handoff(d)=>d.status(),
                Dialog::Flow(flow::FlowDialog::Extract(d))=>d.problem.clone().unwrap_or_else(||if d.plan.is_some(){"Boundary preview ready; review before creating.".into()}else{"Preview required before creating a component.".into()}),
                Dialog::Flow(flow::FlowDialog::Information(_) | flow::FlowDialog::Port(_))=>"Review the complete edit before Apply; changing it requires a new review.".into(),
                Dialog::Project{..}=>"Uncommitted document details. Save details is one undoable edit.".into(),
                _=>"Draft changes remain uncommitted until the named action succeeds.".into(),
            };
            let top=ui.cursor().top();
            let mut actions=super::header(ui,dialog.header(),&status,self.error.as_deref());
            if let Dialog::Handoff(d)=&mut dialog {d.confirmation(ui);}
            let body_height=(height-(ui.cursor().top()-top)).max(40.0);
            egui::ScrollArea::vertical().id_salt("modal_body").max_height(body_height).auto_shrink([false,false]).show(ui,|ui|{
                match &mut dialog{
                    Dialog::Position(d)=>{keep=!self.position_form(ui,d,&mut actions);}
                    Dialog::Flow(d) => { keep = !self.flow_form(ui,d,&mut actions); }
                    Dialog::Project{
                        name,
                        purpose
                    }
                    =>{

                        ui.label("Name");
                        ui.add(egui::TextEdit::singleline(name).id(egui::Id::new("project_name")));
                        ui.label("Outcome, constraints, and design intent");
                        ui.add(egui::TextEdit::multiline(purpose).id(egui::Id::new("project_purpose")).desired_rows(7).desired_width(f32::INFINITY));
                        ui.horizontal(|ui|{
                            if actions.take(Action::Primary,true){
                                let name=name.clone();
                                let purpose=purpose.clone();
                                let q=edit::candidate(self.store.project(),|p|{
                                    p.name=name;
                                    p.purpose=purpose;
                                    Ok(())
                                });
                                self.publish("Edit project",q);
                                keep=self.error.is_some();
                                if keep {ui.memory_mut(|m|m.request_focus(egui::Id::new("project_name")));}
                            }
                            if actions.cancel{
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
                                    let connected=!edit::port_edges(self.store.project(),&port.id).is_empty() || !crate::behavior::references(self.store.project(),&port.id).is_empty();
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
                                        ui.small("Bound: use Refine contract in the inspector to review both layers. Remove ports from the inspector.");
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
                        {
                            if actions.take(Action::Primary,true){
                                self.publish("Edit component",edit::save_node(self.store.project(),node.clone()));
                                keep=self.error.is_some();
                            }
                            if actions.cancel{
                                keep=false;
                            }
                        }
                    }
                    Dialog::Connection(d)=>{
                        keep=!self.connection_form(ui,d,&mut actions);
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
                        if actions.cancel{
                            keep=false;
                        }
                    }
                    Dialog::Contract(d)=>{
                        keep=!self.contract_form(ui,d,&mut actions);
                    }
                    Dialog::Handoff(d)=>{
                        keep=!self.handoff_form(ui,d,ctx,&mut actions);
                    }
                    Dialog::Confirm{
                        message,
                        action
                    }
                    =>{
                        ui.heading("Confirm change");
                        ui.label(message.as_str());
                        let deleting_node = matches!(action,ConfirmAction::Node(_));
                        let ids: Vec<String> = match action {
                            ConfirmAction::Node(id) | ConfirmAction::Child(id) => {
                                let p = self.store.project();
                                let mut ids = if deleting_node {vec![id.clone()]} else {vec![]};
                                if let Some((_,n)) = p.node(id) {
                                    if let Some(child) = &n.child {
                                        let systems = p.descendants(child);
                                        ids.extend(p.systems.iter().filter(|s|systems.contains(&s.id)).flat_map(|s|s.nodes.iter().map(|n|n.id.clone())));
                                    }
                                }
                                ids
                            },
                            ConfirmAction::Port(_,id) | ConfirmAction::Edge(id) => vec![id.clone()],
                            ConfirmAction::Contract(_) => vec![],
                        };
                        for id in ids { for reference in crate::behavior::references(self.store.project(),&id) {
                            ui.colored_label(egui::Color32::LIGHT_RED,format!("Reconcile first: {reference}"));
                        }}
                        {
                            if actions.take(Action::Primary,true){
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
                            if actions.cancel{
                                keep=false;
                            }
                        }
                    }
                    Dialog::Unsaved(action)=>{
                        ui.heading("Keep your changes?");
                        ui.label("This project has unsaved changes. Saving must succeed before continuing.");
                        {
                            if actions.take(Action::Primary,true)&&self.save(false){
                                self.load(action.clone(),ctx);
                                keep=false;
                            }
                            if actions.take(Action::Discard,true){
                                self.load(action.clone(),ctx);
                                keep=false;
                            }
                            if actions.cancel{
                                keep=false;
                            }
                        }
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
                        if actions.cancel{
                            keep=false;
                        }
                    }
                    Dialog::Help=>{
                        ui.horizontal(|ui| {
                            crate::ui::branding::show(ui, 64.0);
                            ui.vertical(|ui| {
                                ui.heading("System Designer");
                                ui.label(format!("Native Rust · {}", env!("CARGO_PKG_VERSION")));
                            });
                        });
                        ui.label("A local editor for recursive components, typed ports, and parent-owned connections. It does not execute workflows or call AI services.");
                        ui.separator();
                        ui.strong("Control Flow workflow");
                        ui.label("Start a flow explicitly, describe local work and alternatives, then review a responsibility boundary before extraction. Structural extractability does not establish a single purpose. Define information through exact bindings; unassigned requirements remain honest drafts. Calls and Interfaces share one component identity.");
                        ui.hyperlink_to("Control Flow workflow and limitations", "https://github.com/lgardner-dev/System-Designer/blob/work/add-logic-diagramming/docs/CONTROL-FLOW-WORKFLOW.md");
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
                        if actions.cancel{
                            keep=false;
                        }
                    }
                }
                if let Some(error)=&self.error{
                    ui.separator();
                    ui.colored_label(egui::Color32::LIGHT_RED,error);
                }
            });
            if actions.cancel { keep=false; }
            if actions.blocked { self.error=Some("Complete the fields and required preview/review or confirmation before applying. Your draft is retained.".into()); }
        });
        if !popup_was_open
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            keep = false;
        }
        if let Some(next) = next {
            self.dialog = Some(next);
        } else if keep {
            self.dialog = Some(dialog);
        }
    }
}
