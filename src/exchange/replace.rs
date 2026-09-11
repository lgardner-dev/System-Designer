use super::{Scope, export};
use crate::model::*;
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value> {
    v.get(key)
        .ok_or_else(|| ModelError::one(format!("missing packet field {key}")))
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    field(v, key)?
        .as_str()
        .ok_or_else(|| ModelError::one(format!("{key} must be a string")))
}
fn children(s: &System) -> BTreeSet<(String, String)> {
    s.nodes
        .iter()
        .filter_map(|n| n.child.as_ref().map(|c| (n.id.clone(), c.clone())))
        .collect()
}
fn require_types<'a>(
    ports: impl Iterator<Item = &'a Port>,
    seen: &HashSet<ContractRef>,
) -> Result<()> {
    for p in ports {
        if let Some(r) = &p.contract {
            if !seen.contains(r) {
                return Err(ModelError::one(format!(
                    "scope is missing definition for {r}"
                )));
            }
        }
    }
    Ok(())
}
/// Builds a complete validated replacement, preserving everything outside the scope.
/// This function performs no I/O and cannot publish, authorize, or execute a proposal.
pub fn replace(p: &Project, packet: &Value, sid: &str) -> Result<Project> {
    validate(p)?;
    if string(packet, "format")? != SCOPE_FORMAT || field(packet, "version")?.as_u64() != Some(1) {
        return Err(ModelError::one(
            "expected a version 1 System Designer scope packet",
        ));
    }
    let scope = match string(packet, "scope")? {
        "component" => Scope::Component,
        "level" => Scope::Level,
        "subtree" => Scope::Subtree,
        _ => return Err(ModelError::one("unknown replacement scope")),
    };
    let allowed = if scope == Scope::Component {
        vec![
            "format",
            "version",
            "scope",
            "projectId",
            "systemId",
            "nodeId",
            "base",
            "component",
            "contracts",
            "context",
        ]
    } else {
        vec![
            "format",
            "version",
            "scope",
            "projectId",
            "systemId",
            "base",
            "boundary",
            "systems",
            "contracts",
            "context",
        ]
    };
    let map = packet
        .as_object()
        .ok_or_else(|| ModelError::one("scope must be an object"))?;
    for key in map.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(ModelError::one(format!("unknown packet field {key}")));
        }
    }
    for key in &allowed {
        field(packet, key)?;
    }
    if string(packet, "projectId")? != p.id || string(packet, "systemId")? != sid {
        return Err(ModelError::one(
            "wrong project or level; open the original level",
        ));
    }
    let nid = if scope == Scope::Component {
        Some(string(packet, "nodeId")?)
    } else {
        None
    };
    let current = export(p, sid, scope, nid)?;
    if field(packet, "base")? != field(&current, "base")? {
        return Err(ModelError::one(
            "stale export: export again and reconcile the proposed change",
        ));
    }
    if field(packet, "context")? != field(&current, "context")? {
        return Err(ModelError::one("read-only context changed"));
    }
    if scope != Scope::Component && field(packet, "boundary")? != field(&current, "boundary")? {
        return Err(ModelError::one(
            "read-only boundary changed; edit its owner at the parent level",
        ));
    }
    let supplied: Vec<Contract> =
        serde_json::from_value(field(packet, "contracts")?.clone()).map_err(ModelError::one)?;
    let mut seen = HashSet::new();
    let mut q = p.clone();
    for c in supplied {
        let r = c.reference();
        if !seen.insert(r.clone()) {
            return Err(ModelError::one("duplicate contract in scope"));
        }
        if let Some(existing) = p.contract(&r) {
            if *existing != c {
                return Err(ModelError::one(format!(
                    "shared definition {r} changed; add a new version or edit the catalog explicitly"
                )));
            }
        } else {
            q.contracts.push(c);
        }
    }
    if let Some(nid) = nid {
        let node: Node =
            serde_json::from_value(field(packet, "component")?.clone()).map_err(ModelError::one)?;
        let old = p
            .node(nid)
            .ok_or_else(|| ModelError::one("missing target component"))?
            .1;
        if node.id != nid || node.child != old.child {
            return Err(ModelError::one(
                "component identity and child ownership must be preserved",
            ));
        }
        require_types(node.ports.iter(), &seen)?;
        // Neighbor definitions are part of read-only context and must still accompany it.
        for n in current["context"]["neighbors"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let port: Port = serde_json::from_value(n["port"].clone()).map_err(ModelError::one)?;
            require_types(std::iter::once(&port), &seen)?;
        }
        *q.node_mut(nid)
            .ok_or_else(|| ModelError::one("missing target component"))? = node;
    } else {
        let incoming: Vec<System> =
            serde_json::from_value(field(packet, "systems")?.clone()).map_err(ModelError::one)?;
        if scope == Scope::Level {
            if incoming.len() != 1 || incoming[0].id != sid {
                return Err(ModelError::one(
                    "level scope must contain exactly its selected system",
                ));
            }
            if children(&incoming[0])
                != children(
                    p.system(sid)
                        .ok_or_else(|| ModelError::one("missing selected system"))?,
                )
            {
                return Err(ModelError::one(
                    "level replacement must preserve hidden child ownership; use subtree scope to change it",
                ));
            }
        }
        let removed = if scope == Scope::Level {
            HashSet::from([sid.to_owned()])
        } else {
            p.descendants(sid)
        };
        q.systems.retain(|s| !removed.contains(&s.id));
        let outside: HashSet<_> = q.systems.iter().map(|s| s.id.clone()).collect();
        let included: HashSet<_> = incoming.iter().map(|s| s.id.clone()).collect();
        for s in &incoming {
            if outside.contains(&s.id) {
                return Err(ModelError::one(
                    "replacement collides with an outside system",
                ));
            }
            require_types(s.nodes.iter().flat_map(|n| n.ports.iter()), &seen)?;
        }
        require_types(p.boundary(sid).iter(), &seen)?;
        q.systems.extend(incoming);
        q.prune_layout();
        validate(&q)?;
        if scope == Scope::Subtree && q.descendants(sid) != included {
            return Err(ModelError::one(
                "subtree must contain exactly the selected system and all its descendants",
            ));
        }
    }
    validate(&q)?;
    Ok(q)
}
