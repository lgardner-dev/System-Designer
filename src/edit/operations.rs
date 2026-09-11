use crate::model::*;
use std::collections::HashSet;
pub fn candidate(p: &Project, f: impl FnOnce(&mut Project) -> Result<()>) -> Result<Project> {
    let mut q = p.clone();
    f(&mut q)?;
    validate(&q)?;
    Ok(q)
}
pub fn add_node(p: &Project, sid: &str) -> Result<(Project, String)> {
    let id = p.fresh("component");
    let q = candidate(p, |q| {
        q.system_mut(sid)
            .ok_or_else(|| ModelError::one("missing system"))?
            .nodes
            .push(Node {
                id: id.clone(),
                name: "New component".into(),
                purpose: String::new(),
                kind: Kind::Component,
                ports: vec![],
                child: None,
            });
        Ok(())
    })?;
    Ok((q, id))
}
pub fn create_child(p: &Project, nid: &str) -> Result<(Project, String)> {
    let id = p.fresh("system");
    let q = candidate(p, |q| {
        let n = q
            .node_mut(nid)
            .ok_or_else(|| ModelError::one("missing component"))?;
        if n.child.is_some() {
            return Err(ModelError::one("component already owns a child system"));
        }
        n.child = Some(id.clone());
        q.systems.push(System {
            id: id.clone(),
            nodes: vec![],
            edges: vec![],
        });
        Ok(())
    })?;
    Ok((q, id))
}
pub fn delete_child(p: &Project, nid: &str) -> Result<Project> {
    let n = p
        .node(nid)
        .ok_or_else(|| ModelError::one("missing component"))?
        .1;
    let removed = n
        .child
        .as_ref()
        .map(|c| p.descendants(c))
        .unwrap_or_default();
    candidate(p, |q| {
        q.systems.retain(|s| !removed.contains(&s.id));
        if let Some(n) = q.node_mut(nid) {
            n.child = None;
        }
        q.prune_layout();
        Ok(())
    })
}
pub fn delete_node(p: &Project, nid: &str) -> Result<Project> {
    let n = p
        .node(nid)
        .ok_or_else(|| ModelError::one("missing component"))?
        .1;
    let removed = n
        .child
        .as_ref()
        .map(|c| p.descendants(c))
        .unwrap_or_default();
    candidate(p, |q| {
        q.systems.retain(|s| !removed.contains(&s.id));
        for s in &mut q.systems {
            s.edges.retain(|e| {
                e.from.node.as_deref() != Some(nid) && e.to.node.as_deref() != Some(nid)
            });
            s.nodes.retain(|n| n.id != nid);
        }
        q.prune_layout();
        Ok(())
    })
}
pub fn port_edges(p: &Project, pid: &str) -> Vec<(String, String)> {
    p.systems
        .iter()
        .flat_map(|s| {
            s.edges
                .iter()
                .filter(|e| e.from.port == pid || e.to.port == pid)
                .map(move |e| (s.id.clone(), e.id.clone()))
        })
        .collect()
}
pub fn delete_port(p: &Project, nid: &str, pid: &str) -> Result<Project> {
    candidate(p, |q| {
        let n = q
            .node_mut(nid)
            .ok_or_else(|| ModelError::one("missing component"))?;
        if !n.ports.iter().any(|r| r.id == pid) {
            return Err(ModelError::one("missing port"));
        }
        n.ports.retain(|r| r.id != pid);
        for s in &mut q.systems {
            s.edges.retain(|e| e.from.port != pid && e.to.port != pid);
        }
        Ok(())
    })
}
pub fn delete_edge(p: &Project, sid: &str, eid: &str) -> Result<Project> {
    candidate(p, |q| {
        let s = q
            .system_mut(sid)
            .ok_or_else(|| ModelError::one("missing system"))?;
        s.edges.retain(|e| e.id != eid);
        Ok(())
    })
}
pub fn add_port(p: &Project, nid: &str, direction: Direction) -> Result<Project> {
    let id = p.fresh("port");
    candidate(p, |q| {
        q.node_mut(nid)
            .ok_or_else(|| ModelError::one("missing component"))?
            .ports
            .push(Port {
                id,
                name: direction.label().into(),
                direction,
                contract: None,
            });
        Ok(())
    })
}
pub fn save_node(p: &Project, node: Node) -> Result<Project> {
    candidate(p, |q| {
        let old = q
            .node_mut(&node.id)
            .ok_or_else(|| ModelError::one("missing component"))?;
        *old = node;
        Ok(())
    })
}
pub fn contract_users(p: &Project, r: &ContractRef) -> Vec<String> {
    p.systems
        .iter()
        .flat_map(|s| s.nodes.iter())
        .flat_map(|n| {
            n.ports
                .iter()
                .filter(|p| p.contract.as_ref() == Some(r))
                .map(move |p| format!("{} · {}", n.name, p.name))
        })
        .collect()
}
pub fn save_contract(
    p: &Project,
    contract: Contract,
    editing: Option<&ContractRef>,
    consent: bool,
) -> Result<Project> {
    if let Some(old) = editing {
        if *old != contract.reference() {
            return Err(ModelError::one(
                "existing contract identity is immutable; use New version",
            ));
        }
        if !contract_users(p, old).is_empty() && !consent {
            return Err(ModelError::one(
                "confirm updating all users of this shared definition",
            ));
        }
    }
    candidate(p, |q| {
        if let Some(old) = editing {
            let c = q
                .contracts
                .iter_mut()
                .find(|c| c.reference() == *old)
                .ok_or_else(|| ModelError::one("missing contract"))?;
            *c = contract;
        } else {
            if q.contract(&contract.reference()).is_some() {
                return Err(ModelError::one("contract ID/version already exists"));
            }
            q.contracts.push(contract);
        }
        Ok(())
    })
}
pub fn delete_contract(p: &Project, r: &ContractRef) -> Result<Project> {
    if !contract_users(p, r).is_empty() {
        return Err(ModelError::one("cannot delete a referenced contract"));
    }
    candidate(p, |q| {
        q.contracts.retain(|c| c.reference() != *r);
        Ok(())
    })
}
pub fn contract_ids(p: &Project) -> HashSet<ContractRef> {
    p.contracts.iter().map(Contract::reference).collect()
}
