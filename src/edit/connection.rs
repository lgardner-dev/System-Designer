use crate::model::*;
use std::collections::BTreeSet;
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
pub struct ChangedData {
    pub owner: String,
    pub id: String,
    pub name: String,
    pub previous: Option<ContractRef>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Impact {
    pub ports: Vec<ChangedPort>,
    pub edges: Vec<ChangedEdge>,
    pub data: Vec<ChangedData>,
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
    Ok(binding_impact(
        p,
        &[from.port.clone(), to.port.clone()],
        &[],
        Some(chosen),
        editing,
    ))
}
/// The closure follows exact shared port identities, interface edges and explicit
/// DataEnd bindings. Names and previous contract equality never create adjacency.
pub fn binding_impact(
    p: &Project,
    seed_ports: &[String],
    seed_data: &[(String, String)],
    chosen: Option<&ContractRef>,
    editing_edge: Option<&str>,
) -> Impact {
    let mut ports: BTreeSet<String> = seed_ports.iter().cloned().collect();
    let mut links: BTreeSet<(String, String)> = seed_data.iter().cloned().collect();
    loop {
        let before = (ports.len(), links.len());
        for s in &p.systems {
            for e in &s.edges {
                if Some(e.id.as_str()) != editing_edge
                    && (ports.contains(&e.from.port) || ports.contains(&e.to.port))
                {
                    ports.insert(e.from.port.clone());
                    ports.insert(e.to.port.clone());
                }
            }
        }
        for (owner, f) in &p.behavior {
            for d in &f.data {
                let key = (owner.clone(), d.id.clone());
                let bound: Vec<_> = [&d.from.port, &d.to.port].into_iter().flatten().collect();
                if links.contains(&key) || bound.iter().any(|id| ports.contains(*id)) {
                    links.insert(key);
                    ports.extend(bound.into_iter().cloned());
                }
            }
        }
        if before == (ports.len(), links.len()) {
            break;
        }
    }
    let changed_ports: Vec<_> = p
        .systems
        .iter()
        .flat_map(|s| {
            s.nodes
                .iter()
                .flat_map(move |n| n.ports.iter().map(move |r| (s, n, r)))
        })
        .filter(|(_, _, r)| ports.contains(&r.id) && r.contract.as_ref() != chosen)
        .map(|(s, n, r)| ChangedPort {
            node: n.id.clone(),
            port: r.id.clone(),
            name: format!("{} · {}", n.name, r.name),
            system: s.id.clone(),
            previous: r.contract.clone(),
        })
        .collect();
    let changed: BTreeSet<_> = changed_ports.iter().map(|r| &r.port).collect();
    let edges: Vec<_> = p
        .systems
        .iter()
        .flat_map(|s| s.edges.iter().map(move |e| (s, e)))
        .filter(|(_, e)| {
            Some(e.id.as_str()) != editing_edge
                && (changed.contains(&e.from.port) || changed.contains(&e.to.port))
        })
        .map(|(s, e)| ChangedEdge {
            id: e.id.clone(),
            system: s.id.clone(),
        })
        .collect();
    let data: Vec<_> = p
        .behavior
        .iter()
        .flat_map(|(o, f)| f.data.iter().map(move |d| (o, d)))
        .filter(|(o, d)| {
            links.contains(&(o.to_string(), d.id.clone())) && d.contract.as_ref() != chosen
        })
        .map(|(o, d)| ChangedData {
            owner: o.clone(),
            id: d.id.clone(),
            name: d.name.clone(),
            previous: d.contract.clone(),
        })
        .collect();
    let requires_consent =
        !edges.is_empty() || !data.is_empty() || changed_ports.iter().any(|r| r.previous.is_some());
    Impact {
        ports: changed_ports,
        edges,
        data,
        requires_consent,
    }
}
/// Candidate-internal helper; the caller must validate the complete transaction.
pub(crate) fn reconcile(q: &mut Project, impact: &Impact, chosen: Option<&ContractRef>) {
    for r in &impact.ports {
        if let Some(n) = q.node_mut(&r.node)
            && let Some(port) = n.ports.iter_mut().find(|p| p.id == r.port)
        {
            port.contract = chosen.cloned();
        }
    }
    for d in &impact.data {
        if let Some(f) = q.behavior.get_mut(&d.owner)
            && let Some(link) = f.data.iter_mut().find(|l| l.id == d.id)
        {
            link.contract = chosen.cloned();
        }
    }
}
pub fn refine_port(
    p: &Project,
    pid: &str,
    contract: Option<ContractRef>,
    new_contract: Option<Contract>,
) -> Result<Project> {
    if !p
        .systems
        .iter()
        .flat_map(|s| &s.nodes)
        .flat_map(|n| &n.ports)
        .any(|r| r.id == pid)
    {
        return Err(ModelError::one("Missing public port"));
    }
    let impact = binding_impact(p, &[pid.into()], &[], contract.as_ref(), None);
    super::candidate(p, |q| {
        add_definition(q, contract.as_ref(), new_contract)?;
        reconcile(q, &impact, contract.as_ref());
        Ok(())
    })
}
pub(crate) fn add_definition(
    q: &mut Project,
    chosen: Option<&ContractRef>,
    definition: Option<Contract>,
) -> Result<()> {
    if let Some(c) = definition {
        if Some(&c.reference()) != chosen || q.contract(&c.reference()).is_some() {
            return Err(ModelError::one(
                "New contract must match the assignment and have an unused identity/version",
            ));
        }
        q.contracts.push(c);
    }
    Ok(())
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
    reconcile(&mut q, &impact, Some(&request.contract));
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
    crate::behavior::invalidate_reviews(p, &mut q);
    validate(&q)?;
    Ok(q)
}
