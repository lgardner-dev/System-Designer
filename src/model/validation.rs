use super::*;
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub path: String,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ModelError(pub Vec<Problem>);
pub type Result<T> = std::result::Result<T, ModelError>;
impl ModelError {
    pub fn one(message: impl ToString) -> Self {
        Self(vec![Problem {
            path: "project".into(),
            message: message.to_string(),
        }])
    }
}
impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, p) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}: {}", p.path, p.message)?;
        }
        Ok(())
    }
}
impl std::error::Error for ModelError {}
pub fn parse(text: &str) -> Result<Project> {
    let p: Project = serde_json::from_str(text).map_err(ModelError::one)?;
    validate(&p)?;
    Ok(p)
}
fn issue(e: &mut Vec<Problem>, path: impl ToString, message: impl ToString) {
    e.push(Problem {
        path: path.to_string(),
        message: message.to_string(),
    });
}
fn text(e: &mut Vec<Problem>, path: &str, value: &str) {
    if value.trim().is_empty() {
        issue(e, path, "must be nonempty");
    }
}
fn identity(e: &mut Vec<Problem>, used: &mut HashSet<String>, path: &str, id: &str) {
    text(e, path, id);
    if !used.insert(id.into()) {
        issue(e, path, format!("duplicate structural ID {id}"));
    }
}
fn version(e: &mut Vec<Problem>, path: &str, v: u64) {
    if !(1..=MAX_SAFE_INTEGER).contains(&v) {
        issue(e, path, "version must be a positive JSON-safe integer");
    }
}
/// Graph cycles are legal. Ownership cycles, unreachable systems and cross-level links are not.
/// Component containment is walked iteratively; there is no depth or eight-node validity limit.
pub fn validate(p: &Project) -> Result<()> {
    let mut errors = vec![];
    let mut ids = HashSet::new();
    if p.format != FORMAT {
        issue(&mut errors, "format", format!("expected {FORMAT}"));
    }
    if p.version != 1 {
        issue(&mut errors, "version", "unsupported project version");
    }
    identity(&mut errors, &mut ids, "id", &p.id);
    text(&mut errors, "name", &p.name);
    text(&mut errors, "root", &p.root);
    let mut catalog = HashSet::new();
    for (i, c) in p.contracts.iter().enumerate() {
        let at = format!("contracts[{i}]");
        text(&mut errors, &format!("{at}.id"), &c.id);
        text(&mut errors, &format!("{at}.name"), &c.name);
        version(&mut errors, &at, c.version);
        if !catalog.insert(c.reference()) {
            issue(&mut errors, &at, "duplicate contract ID/version");
        }
        let mut pending = vec![(&c.definition, format!("{at}.definition"))];
        while let Some((shape, path)) = pending.pop() {
            match shape {
                Shape::Enum { values } => {
                    if values.is_empty() {
                        issue(&mut errors, &path, "enum needs at least one value");
                    }
                    let mut names = HashSet::new();
                    for v in values {
                        text(&mut errors, &path, v);
                        if !names.insert(v) {
                            issue(&mut errors, &path, "duplicate enum value");
                        }
                    }
                }
                Shape::Object { fields } => {
                    let mut names = HashSet::new();
                    for (j, f) in fields.iter().enumerate() {
                        let fp = format!("{path}.fields[{j}]");
                        text(&mut errors, &fp, &f.name);
                        if !names.insert(&f.name) {
                            issue(&mut errors, &fp, "duplicate field name");
                        }
                        pending.push((&f.schema, format!("{fp}.schema")));
                    }
                }
                Shape::Array { items } => pending.push((items.as_ref(), format!("{path}.items"))),
                _ => {}
            }
        }
    }
    let mut systems = HashMap::new();
    let mut nodes = HashMap::new();
    let mut owners = HashMap::new();
    for s in &p.systems {
        identity(&mut errors, &mut ids, "system.id", &s.id);
        systems.insert(s.id.as_str(), s);
        for n in &s.nodes {
            identity(&mut errors, &mut ids, "node.id", &n.id);
            text(&mut errors, &format!("{}.name", n.id), &n.name);
            nodes.insert(n.id.as_str(), (s.id.as_str(), n));
            if let Some(child) = &n.child {
                text(&mut errors, &format!("{}.child", n.id), child);
                if owners.insert(child.as_str(), n).is_some() {
                    issue(&mut errors, child, "child has more than one owner");
                }
            }
            for port in &n.ports {
                identity(&mut errors, &mut ids, "port.id", &port.id);
                text(&mut errors, &format!("{}.name", port.id), &port.name);
                if let Some(r) = &port.contract {
                    text(&mut errors, &port.id, &r.id);
                    version(&mut errors, &port.id, r.version);
                    if !catalog.contains(r) {
                        issue(&mut errors, &port.id, format!("unknown contract {r}"));
                    }
                }
            }
        }
        for edge in &s.edges {
            identity(&mut errors, &mut ids, "edge.id", &edge.id);
        }
    }
    if !systems.contains_key(p.root.as_str()) {
        issue(&mut errors, "root", "missing root system");
    }
    if owners.contains_key(p.root.as_str()) {
        issue(&mut errors, "root", "root may not have an owner");
    }
    for s in &p.systems {
        if s.id != p.root && !owners.contains_key(s.id.as_str()) {
            issue(&mut errors, &s.id, "non-root system has no owner");
        }
    }
    for child in owners.keys() {
        if !systems.contains_key(child) {
            issue(&mut errors, child, "missing child system");
        }
    }
    let mut visited = HashSet::new();
    let mut pending = vec![p.root.as_str()];
    while let Some(sid) = pending.pop() {
        if !visited.insert(sid) {
            continue;
        }
        if let Some(s) = systems.get(sid) {
            for n in &s.nodes {
                if let Some(c) = &n.child {
                    pending.push(c.as_str());
                }
            }
        }
    }
    for s in &p.systems {
        if !visited.contains(s.id.as_str()) {
            issue(&mut errors, &s.id, "unreachable or cyclic containment");
        }
    }
    for s in &p.systems {
        for edge in &s.edges {
            let mut resolve = |ep: &Endpoint, role: Direction| -> Option<&Port> {
                let n = if let Some(nid) = &ep.node {
                    match nodes.get(nid.as_str()) {
                        Some((sid, n)) if *sid == s.id => *n,
                        Some(_) => {
                            issue(&mut errors, &edge.id, "cross-level connection forbidden");
                            return None;
                        }
                        None => {
                            issue(&mut errors, &edge.id, "missing endpoint component");
                            return None;
                        }
                    }
                } else {
                    match owners.get(s.id.as_str()) {
                        Some(n) => *n,
                        None => {
                            issue(&mut errors, &edge.id, "root has no boundary");
                            return None;
                        }
                    }
                };
                let Some(port) = n.ports.iter().find(|port| port.id == ep.port) else {
                    issue(&mut errors, &edge.id, "missing endpoint port");
                    return None;
                };
                let direction = if ep.node.is_none() {
                    port.direction.opposite()
                } else {
                    port.direction
                };
                if direction != role {
                    issue(
                        &mut errors,
                        &edge.id,
                        "source must produce and destination must consume",
                    );
                }
                Some(port)
            };
            let a = resolve(&edge.from, Direction::Out);
            let b = resolve(&edge.to, Direction::In);
            if let (Some(a), Some(b)) = (a, b) {
                if a.contract.is_none() || a.contract != b.contract {
                    issue(
                        &mut errors,
                        &edge.id,
                        "connected ports need the same assigned contract ID/version",
                    );
                }
            }
        }
    }
    for (sid, positions) in &p.layout {
        if !systems.contains_key(sid.as_str()) {
            issue(&mut errors, "layout", "unknown system");
        }
        for (nid, pos) in positions {
            if nodes.get(nid.as_str()).map(|(s, _)| *s) != Some(sid.as_str()) {
                issue(&mut errors, "layout", "unknown or nonlocal component");
            }
            if !pos.x.is_finite() || !pos.y.is_finite() || pos.x < 0.0 || pos.y < 0.0 {
                issue(
                    &mut errors,
                    "layout",
                    "coordinates must be finite and nonnegative",
                );
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ModelError(errors))
    }
}
