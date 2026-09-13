use super::*;
use crate::model::{ModelError, Project, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    format: String,
    version: u32,
    project_id: String,
    owner: String,
    base: String,
    context: Value,
    flow: Option<Flow>,
}
fn context(p: &Project, owner: &str) -> Value {
    let node = p.node(owner).map(|(_, n)| n);
    // Only direct call signatures are needed here, not all child interiors.
    let children=system(p,owner).map(|s|s.nodes.iter().map(|n|json!({"component":n,"outcomes":p.behavior.get(&n.id).map(|f|f.steps.iter().filter(|s|s.kind==StepKind::Outcome).collect::<Vec<_>>())})).collect::<Vec<_>>()).unwrap_or_default();
    let callers:Vec<_>=p.behavior.iter().flat_map(|(scope,f)|f.steps.iter().filter(move|s|s.target.as_deref()==Some(owner)).map(move|s|json!({"owner":scope,"step":s,"returns":f.transitions.iter().filter(|t|t.from==s.id).collect::<Vec<_>>(),"data":f.data.iter().filter(|d|d.from.step.as_ref()==Some(&s.id)||d.to.step.as_ref()==Some(&s.id)).collect::<Vec<_>>()}))).collect();
    json!({"project":{"id":p.id,"name":p.name,"purpose":p.purpose},"component":node,"children":children,"interfaces":system(p,owner),"contracts":p.contracts,"callers":callers})
}
pub fn export(p: &Project, owner: &str) -> Result<Value> {
    crate::model::validate(p)?;
    if !exists(p, owner) {
        return Err(ModelError::one("Missing behavior owner"));
    }
    let context = context(p, owner);
    let flow = p.behavior.get(owner).cloned();
    let base = crate::exchange::digest(
        crate::exchange::canonical_json(&json!({"context":context,"flow":flow})).as_bytes(),
    );
    serde_json::to_value(Packet {
        format: "system-designer-behavior-scope".into(),
        version: 1,
        project_id: p.id.clone(),
        owner: owner.into(),
        base,
        context,
        flow,
    })
    .map_err(ModelError::one)
}
pub fn replace(p: &Project, value: &Value, owner: &str) -> Result<Project> {
    if value.get("flow").is_none() {
        return Err(ModelError::one(
            "Missing flow payload; explicit flow: null requests clearing this scope",
        ));
    }
    let packet: Packet = serde_json::from_value(value.clone()).map_err(ModelError::one)?;
    if packet.format != "system-designer-behavior-scope"
        || packet.version != 1
        || packet.owner != owner
        || packet.project_id != p.id
    {
        return Err(ModelError::one("Wrong behavior format, project or scope"));
    }
    let current = export(p, owner)?;
    if current["base"] != packet.base || current["context"] != packet.context {
        return Err(ModelError::one(
            "Stale behavior scope or changed read-only context; export again",
        ));
    }
    crate::edit::candidate(p, |q| {
        if let Some(flow) = packet.flow {
            q.version = 2;
            q.behavior.insert(owner.into(), flow);
        } else {
            q.behavior.remove(owner);
        }
        prune_layout(q);
        Ok(())
    })
}
