use super::{canonical_json, digest};
use crate::model::*;
use serde_json::{Value, json};
use std::collections::HashSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Component,
    Level,
    Subtree,
}
impl Scope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Component => "Selected component only",
            Self::Level => "Current level only",
            Self::Subtree => "Current level + descendants",
        }
    }
    pub fn tag(self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::Level => "level",
            Self::Subtree => "subtree",
        }
    }
}
fn project_context(p: &Project) -> Value {
    json!({
        "name":p.name,
        "purpose":p.purpose
    })
}
fn contracts<'a>(p: &'a Project, used: &HashSet<ContractRef>) -> Vec<&'a Contract> {
    p.contracts
        .iter()
        .filter(|c| used.contains(&c.reference()))
        .collect()
}
pub fn export(p: &Project, sid: &str, scope: Scope, nid: Option<&str>) -> Result<Value> {
    validate(p)?;
    let s = p
        .system(sid)
        .ok_or_else(|| ModelError::one("missing selected system"))?;
    let content = if scope == Scope::Component {
        let nid = nid.ok_or_else(|| ModelError::one("select a component first"))?;
        let n = s
            .nodes
            .iter()
            .find(|n| n.id == nid)
            .ok_or_else(|| ModelError::one("component is not in selected level"))?;
        let links: Vec<_> = s
            .edges
            .iter()
            .filter(|e| e.from.node.as_deref() == Some(nid) || e.to.node.as_deref() == Some(nid))
            .collect();
        let mut used: HashSet<_> = n.ports.iter().filter_map(|r| r.contract.clone()).collect();
        let mut neighbors = vec![];
        let mut included = HashSet::new();
        for e in &links {
            for ep in [&e.from, &e.to] {
                if ep.node.as_deref() == Some(nid) || !included.insert(ep.port.clone()) {
                    continue;
                }
                let port = p
                    .port(sid, ep)
                    .ok_or_else(|| ModelError::one("missing neighbor port"))?;
                if let Some(c) = &port.contract {
                    used.insert(c.clone());
                }
                let name = ep
                    .node
                    .as_ref()
                    .and_then(|id| p.node(id))
                    .map(|(_, n)| n.name.as_str())
                    .unwrap_or("Parent boundary");
                neighbors.push(json!({
                    "node":ep.node,
                    "name":name,
                    "port":port
                }));
            }
        }
        json!({
            "component":n,
            "contracts":contracts(p,&used),
            "context":{
                "project":project_context(p),
                "ancestors":p.ancestors(sid),
                "connections":links,
                "neighbors":neighbors,
                "preservedChild":n.child
            }
        })
    } else {
        let selected = if scope == Scope::Level {
            HashSet::from([sid.to_owned()])
        } else {
            p.descendants(sid)
        };
        let systems: Vec<_> = p
            .systems
            .iter()
            .filter(|s| selected.contains(&s.id))
            .collect();
        let mut used: HashSet<_> = systems
            .iter()
            .flat_map(|s| s.nodes.iter())
            .flat_map(|n| n.ports.iter())
            .filter_map(|r| r.contract.clone())
            .collect();
        for r in p.boundary(sid) {
            if let Some(c) = &r.contract {
                used.insert(c.clone());
            }
        }
        let boundary = p.owner(sid).map(|(_, n)| {
            json!({
                "ownerId":n.id,
                "name":n.name,
                "purpose":n.purpose,
                "ports":n.ports
            })
        });
        let preserved: Vec<Value> = if scope == Scope::Level {
            s.nodes
                .iter()
                .filter_map(|n| {
                    n.child.as_ref().map(|id| {
                        json!({
                            "ownerId":n.id,
                            "systemId":id
                        })
                    })
                })
                .collect()
        } else {
            vec![]
        };
        json!({
            "boundary":boundary,
            "systems":systems,
            "contracts":contracts(p,&used),
            "context":{
                "project":project_context(p),
                "ancestors":p.ancestors(sid),
                "externalConnections":p.external_connections(sid),
                "preservedChildren":preserved
            }
        })
    };
    let base = digest(canonical_json(&content).as_bytes());
    let mut packet = content;
    let m = packet
        .as_object_mut()
        .ok_or_else(|| ModelError::one("invalid internal packet"))?;
    m.insert("format".into(), json!(SCOPE_FORMAT));
    m.insert("version".into(), json!(1));
    m.insert("scope".into(), json!(scope.tag()));
    m.insert("projectId".into(), json!(p.id));
    m.insert("systemId".into(), json!(sid));
    m.insert("base".into(), json!(base));
    if scope == Scope::Component {
        m.insert("nodeId".into(), json!(nid));
    }
    Ok(packet)
}
