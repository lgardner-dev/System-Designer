use crate::model::*;
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug, PartialEq)]
pub struct ChangedPort {
    pub node: String,
    pub port: String,
    pub name: String,
    pub system: String,
    pub previous: Option<ContractRef>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ChangedEdge {
    pub id: String,
    pub system: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Impact {
    pub ports: Vec<ChangedPort>,
    pub edges: Vec<ChangedEdge>,
    pub requires_consent: bool,
}
#[derive(Clone, Debug)]
pub struct Connection {
    pub id: Option<String>,
    pub from: Endpoint,
    pub to: Endpoint,
    pub label: String,
    pub contract: ContractRef,
    pub new_contract: Option<Contract>,
    pub consent: bool,
}
pub fn connection_impact(
    p: &Project,
    sid: &str,
    from: &Endpoint,
    to: &Endpoint,
    chosen: &ContractRef,
    editing: Option<&str>,
) -> Result<Impact> {
    let s = p
        .system(sid)
        .ok_or_else(|| ModelError::one("missing system"))?;
    if let Some(id) = editing {
        if !s.edges.iter().any(|e| e.id == id) {
            return Err(ModelError::one("missing connection to edit"));
        }
    }
    if p.effective_direction(sid, from) != Some(Direction::Out)
        || p.effective_direction(sid, to) != Some(Direction::In)
    {
        return Err(ModelError::one(
            "choose a local output and input (or corresponding boundary ports)",
        ));
    }
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for s in &p.systems {
        for n in &s.nodes {
            for r in &n.ports {
                adjacency.entry(&r.id).or_default();
            }
        }
        for e in &s.edges {
            if Some(e.id.as_str()) != editing {
                adjacency.entry(&e.from.port).or_default().push(&e.to.port);
                adjacency.entry(&e.to.port).or_default().push(&e.from.port);
            }
        }
    }
    let mut seen = HashSet::new();
    let mut pending = vec![from.port.as_str(), to.port.as_str()];
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        if let Some(neighbors) = adjacency.get(id) {
            pending.extend(neighbors.iter().copied());
        }
    }
    let mut ports = vec![];
    for s in &p.systems {
        for n in &s.nodes {
            for r in &n.ports {
                if seen.contains(r.id.as_str()) && r.contract.as_ref() != Some(chosen) {
                    ports.push(ChangedPort {
                        node: n.id.clone(),
                        port: r.id.clone(),
                        name: format!("{} · {}", n.name, r.name),
                        system: s.id.clone(),
                        previous: r.contract.clone(),
                    });
                }
            }
        }
    }
    let changed: HashSet<&str> = ports.iter().map(|r| r.port.as_str()).collect();
    let edges: Vec<_> = p
        .systems
        .iter()
        .flat_map(|s| {
            s.edges
                .iter()
                .filter(|e| {
                    Some(e.id.as_str()) != editing
                        && (changed.contains(e.from.port.as_str())
                            || changed.contains(e.to.port.as_str()))
                })
                .map(move |e| ChangedEdge {
                    id: e.id.clone(),
                    system: s.id.clone(),
                })
        })
        .collect();
    let requires_consent = !edges.is_empty() || ports.iter().any(|r| r.previous.is_some());
    Ok(Impact {
        ports,
        edges,
        requires_consent,
    })
}
pub fn connect(p: &Project, sid: &str, request: &Connection) -> Result<Project> {
    validate(p)?;
    let mut q = p.clone();
    if let Some(c) = &request.new_contract {
        if c.reference() != request.contract {
            return Err(ModelError::one("new contract does not match selection"));
        }
        if q.contract(&c.reference()).is_some() {
            return Err(ModelError::one(
                "contract already exists; select it or create a new version",
            ));
        }
        q.contracts.push(c.clone());
    }
    if q.contract(&request.contract).is_none() {
        return Err(ModelError::one("select or define a contract first"));
    }
    let impact = connection_impact(
        p,
        sid,
        &request.from,
        &request.to,
        &request.contract,
        request.id.as_deref(),
    )?;
    if impact.requires_consent && !request.consent {
        return Err(ModelError::one(
            "confirm the displayed shared port and connection changes",
        ));
    }
    for r in &impact.ports {
        let n = q
            .node_mut(&r.node)
            .ok_or_else(|| ModelError::one("missing affected component"))?;
        let port = n
            .ports
            .iter_mut()
            .find(|p| p.id == r.port)
            .ok_or_else(|| ModelError::one("missing affected port"))?;
        port.contract = Some(request.contract.clone());
    }
    let id = request.id.clone().unwrap_or_else(|| q.fresh("edge"));
    let edge = Edge {
        id,
        from: request.from.clone(),
        to: request.to.clone(),
        label: Some(request.label.clone()),
    };
    let s = q
        .system_mut(sid)
        .ok_or_else(|| ModelError::one("missing system"))?;
    if let Some(id) = &request.id {
        let target = s
            .edges
            .iter_mut()
            .find(|e| e.id == *id)
            .ok_or_else(|| ModelError::one("missing edge"))?;
        *target = edge;
    } else {
        s.edges.push(edge);
    }
    validate(&q)?;
    Ok(q)
}
