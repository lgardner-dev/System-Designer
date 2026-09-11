use super::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};
#[derive(Clone, Debug)]
pub struct AvailablePort {
    pub endpoint: Endpoint,
    pub name: String,
    pub contract: Option<ContractRef>,
}
impl Project {
    pub fn system(&self, id: &str) -> Option<&System> {
        self.systems.iter().find(|s| s.id == id)
    }
    pub fn system_mut(&mut self, id: &str) -> Option<&mut System> {
        self.systems.iter_mut().find(|s| s.id == id)
    }
    pub fn node(&self, id: &str) -> Option<(&System, &Node)> {
        self.systems
            .iter()
            .find_map(|s| s.nodes.iter().find(|n| n.id == id).map(|n| (s, n)))
    }
    pub fn node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.systems
            .iter_mut()
            .find_map(|s| s.nodes.iter_mut().find(|n| n.id == id))
    }
    pub fn owner(&self, sid: &str) -> Option<(&System, &Node)> {
        self.systems.iter().find_map(|s| {
            s.nodes
                .iter()
                .find(|n| n.child.as_deref() == Some(sid))
                .map(|n| (s, n))
        })
    }
    pub fn contract(&self, r: &ContractRef) -> Option<&Contract> {
        self.contracts
            .iter()
            .find(|c| c.id == r.id && c.version == r.version)
    }
    pub fn boundary(&self, sid: &str) -> &[Port] {
        self.owner(sid)
            .map(|(_, n)| n.ports.as_slice())
            .unwrap_or(&[])
    }
    pub fn port(&self, sid: &str, ep: &Endpoint) -> Option<&Port> {
        let n = match &ep.node {
            Some(id) => {
                let (s, n) = self.node(id)?;
                if s.id != sid {
                    return None;
                }
                n
            }
            None => self.owner(sid)?.1,
        };
        n.ports.iter().find(|r| r.id == ep.port)
    }
    pub fn effective_direction(&self, sid: &str, ep: &Endpoint) -> Option<Direction> {
        self.port(sid, ep).map(|p| {
            if ep.node.is_none() {
                p.direction.opposite()
            } else {
                p.direction
            }
        })
    }
    pub fn endpoints(&self, sid: &str, dir: Direction) -> Vec<AvailablePort> {
        let mut out = vec![];
        for p in self.boundary(sid) {
            if p.direction.opposite() == dir {
                out.push(AvailablePort {
                    endpoint: Endpoint {
                        node: None,
                        port: p.id.clone(),
                    },
                    name: format!("Boundary · {}", p.name),
                    contract: p.contract.clone(),
                });
            }
        }
        if let Some(s) = self.system(sid) {
            for n in &s.nodes {
                for p in &n.ports {
                    if p.direction == dir {
                        out.push(AvailablePort {
                            endpoint: Endpoint {
                                node: Some(n.id.clone()),
                                port: p.id.clone(),
                            },
                            name: format!("{} · {}", n.name, p.name),
                            contract: p.contract.clone(),
                        });
                    }
                }
            }
        }
        out
    }
    pub fn descendants(&self, sid: &str) -> HashSet<String> {
        let index: BTreeMap<&str, &System> =
            self.systems.iter().map(|s| (s.id.as_str(), s)).collect();
        let mut seen = HashSet::new();
        let mut stack = vec![sid.to_owned()];
        while let Some(id) = stack.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            if let Some(s) = index.get(id.as_str()) {
                for n in &s.nodes {
                    if let Some(c) = &n.child {
                        stack.push(c.clone());
                    }
                }
            }
        }
        seen
    }
    pub fn ancestors(&self, sid: &str) -> Vec<Value> {
        let owners: BTreeMap<&str, (&System, &Node)> = self
            .systems
            .iter()
            .flat_map(|s| {
                s.nodes
                    .iter()
                    .filter_map(move |n| n.child.as_deref().map(|c| (c, (s, n))))
            })
            .collect();
        let mut out = vec![];
        let mut current = sid;
        let mut seen = HashSet::new();
        while let Some((s, n)) = owners.get(current) {
            if !seen.insert(current) {
                break;
            }
            out.push(json!({
                "system":s.id,
                "node":n.id,
                "name":n.name,
                "purpose":n.purpose
            }));
            current = &s.id;
        }
        out.reverse();
        out
    }
    pub fn external_connections(&self, sid: &str) -> Vec<Value> {
        let Some((parent, owner)) = self.owner(sid) else {
            return vec![];
        };
        owner
            .ports
            .iter()
            .map(|port| {
                let mut links = vec![];
                for e in &parent.edges {
                    let (own, remote) = if port.direction == Direction::In {
                        (&e.to, &e.from)
                    } else {
                        (&e.from, &e.to)
                    };
                    if own.node.as_deref() != Some(owner.id.as_str()) || own.port != port.id {
                        continue;
                    }
                    let name = remote
                        .node
                        .as_ref()
                        .and_then(|id| self.node(id))
                        .map(|(_, n)| n.name.as_str())
                        .unwrap_or("Parent boundary");
                    let port_name = self
                        .port(&parent.id, remote)
                        .map(|p| p.name.as_str())
                        .unwrap_or("");
                    links.push(json!({
                        "edge":e.id,
                        "node":remote.node,
                        "name":name,
                        "port":remote.port,
                        "portName":port_name,
                        "label":e.label.as_deref().unwrap_or(""),
                        "contract":port.contract
                    }));
                }
                json!({
                    "port":port.id,
                    "direction":port.direction,
                    "links":links
                })
            })
            .collect()
    }
    pub fn fresh(&self, prefix: &str) -> String {
        let mut used = HashSet::from([self.id.as_str()]);
        for s in &self.systems {
            used.insert(s.id.as_str());
            for n in &s.nodes {
                used.insert(n.id.as_str());
                for p in &n.ports {
                    used.insert(p.id.as_str());
                }
            }
            for e in &s.edges {
                used.insert(e.id.as_str());
            }
        }
        for i in 1u64.. {
            let id = format!("{prefix}.{i}");
            if !used.contains(id.as_str()) {
                return id;
            }
        }
        unreachable!("finite memory cannot contain all u64 identifiers")
    }
    pub fn prune_layout(&mut self) {
        let locations: BTreeMap<String, HashSet<String>> = self
            .systems
            .iter()
            .map(|s| (s.id.clone(), s.nodes.iter().map(|n| n.id.clone()).collect()))
            .collect();
        self.layout.retain(|sid, positions| {
            if let Some(ids) = locations.get(sid) {
                positions.retain(|id, _| ids.contains(id));
                true
            } else {
                false
            }
        });
    }
}
pub fn node_height(n: &Node) -> f64 {
    let left = n
        .ports
        .iter()
        .filter(|p| p.direction == Direction::In)
        .count();
    let right = n.ports.len() - left;
    88.0 + left.max(right).max(1) as f64 * 38.0
}
pub fn auto_layout(p: &Project, sid: &str) -> BTreeMap<String, Position> {
    let Some(s) = p.system(sid) else {
        return BTreeMap::new();
    };
    let columns = ((s.nodes.len() as f64 / 1.25).sqrt().ceil() as usize).max(1);
    let mut heights = vec![0.0f64; s.nodes.len().div_ceil(columns)];
    for (i, n) in s.nodes.iter().enumerate() {
        heights[i / columns] = heights[i / columns].max(node_height(n));
    }
    let mut y = 130.0;
    let mut ys = vec![];
    for h in heights {
        ys.push(y);
        y += h + 100.0;
    }
    s.nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            (
                n.id.clone(),
                Position {
                    x: if sid == p.root { 100.0 } else { 280.0 } + (i % columns) as f64 * 420.0,
                    y: ys[i / columns],
                },
            )
        })
        .collect()
}
