use super::*;
use crate::edit;
impl Designer {
    pub(super) fn inspector(&mut self, ui: &mut egui::Ui) {
        if canvas::details(self, ui) {
            return;
        }
        let p = self.store.snapshot();
        match self.selected.clone() {
            Selection::Node(id) => {
                if let Some((_, node)) = p.node(&id) {
                    ui.heading(&node.name);
                    ui.collapsing("Identity", |ui| {
                        ui.monospace(&node.id);
                    });
                    ui.label(node.kind.label());
                    ui.separator();
                    ui.label(&node.purpose);
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Move / position…").clicked() {
                            self.position_dialog();
                        }
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
                    if ui.button("Enter component behavior").clicked() {
                        self.go_scope(id.clone(), Layer::Flow);
                    }
                    if ui.button("Show uses in Control Flow").clicked() {
                        self.dialog = Some(Dialog::Flow(flow::FlowDialog::Uses(id.clone())));
                    }
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
                                    if ui.small_button("Refine contract…").clicked() { self.open_port_refinement(&r.id); }
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
                    for (owner, flow) in &p.behavior {
                        for d in flow
                            .data
                            .iter()
                            .filter(|d| d.exchange.as_ref() == Some(&edge.id))
                        {
                            if ui
                                .button(format!(
                                    "Information use: {} / {} [{}]",
                                    crate::behavior::name(&p, owner),
                                    d.name,
                                    d.id
                                ))
                                .clicked()
                            {
                                self.go_scope(owner.clone(), Layer::Flow);
                                self.flow.selection = flow::FlowSelection {
                                    data: Some(d.id.clone()),
                                    ..Default::default()
                                };
                            }
                        }
                    }
                    if ui.button("Change contract / endpoints").clicked() {
                        self.canvas_session.view = canvas::View::Detail;
                        self.canvas.cancel();
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
            Selection::Port(_) | Selection::Summary(..) => {}
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
                    self.project_settings();
                }
                ui.separator();
                ui.label("Drag between ports to define a connection. Double-click a wire to change its contract. Double-click a component to enter its internals.");
                ui.label("AI handoff includes a built-in initialization prompt and component, level, or subtree exchange.");
                ui.label("Around eight immediate components is a useful design budget, not a structural limit.");
            }
        }
    }
    pub(super) fn ask_delete(&mut self) {
        if matches!(self.selected, Selection::Summary(..)) {
            self.status="A summary is not an editable edge. Choose an exact member before deleting or changing a contract.".into();
            return;
        }
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
}
