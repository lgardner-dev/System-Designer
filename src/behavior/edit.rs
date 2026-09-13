//! Validated behavior authoring operations, shared by forms and packet editing.
use super::*;
use crate::{edit, model::*};

pub fn save_step(p: &Project, owner: &str, step: Step) -> Result<Project> {
    let mut f = flow(p, owner)?;
    if let Some(old) = f.steps.iter_mut().find(|s| s.id == step.id) {
        *old = step;
    } else {
        f.steps.push(step);
    }
    set(p, owner, f)
}
pub fn save_transition(p: &Project, owner: &str, transition: Transition) -> Result<Project> {
    let mut f = flow(p, owner)?;
    if let Some(old) = f.transitions.iter_mut().find(|t| t.id == transition.id) {
        *old = transition;
    } else {
        f.transitions.push(transition);
    }
    set(p, owner, f)
}
fn flow(p: &Project, owner: &str) -> Result<Flow> {
    p.behavior
        .get(owner)
        .cloned()
        .ok_or_else(|| ModelError::one("Start a flow first"))
}
pub fn delete_step(p: &Project, owner: &str, id: &str) -> Result<Project> {
    let callers = outcome_users(p, owner, id);
    if !callers.is_empty() {
        return Err(ModelError::one(format!(
            "Outcome is referenced by: {}. Reconcile these returns first.",
            callers.join(", ")
        )));
    }
    let mut f = flow(p, owner)?;
    f.steps.retain(|s| s.id != id);
    f.transitions.retain(|t| t.from != id && t.to != id);
    f.data
        .retain(|d| d.from.step.as_deref() != Some(id) && d.to.step.as_deref() != Some(id));
    set(p, owner, f)
}
pub fn delete_transition(p: &Project, owner: &str, id: &str) -> Result<Project> {
    let mut f = flow(p, owner)?;
    f.transitions.retain(|t| t.id != id);
    set(p, owner, f)
}
pub fn delete_data(p: &Project, owner: &str, id: &str) -> Result<Project> {
    let mut f = flow(p, owner)?;
    f.data.retain(|d| d.id != id);
    set(p, owner, f)
}
/// Builds the full information edit and binding reconciliation without publication.
/// Preview and apply use this same operation; no invalid intermediate assignment.
pub fn information_candidate(
    p: &Project,
    owner: &str,
    link: DataLink,
    definition: Option<Contract>,
) -> Result<(Project, edit::Impact)> {
    let mut q = p.clone();
    let chosen = link.contract.clone();
    let id = link.id.clone();
    let f = q
        .behavior
        .get_mut(owner)
        .ok_or_else(|| ModelError::one("Missing flow"))?;
    if let Some(old) = f.data.iter_mut().find(|d| d.id == link.id) {
        *old = link;
    } else {
        f.data.push(link);
    }
    edit::add_definition(&mut q, chosen.as_ref(), definition)?;
    let mut impact = edit::binding_impact(
        &q,
        &[],
        &[(owner.into(), id.clone())],
        chosen.as_ref(),
        None,
    );
    // The draft itself already carries the choice; include its previous binding
    // in the review even though no propagation is needed for this exact item.
    if let Some(old) = p
        .behavior
        .get(owner)
        .and_then(|f| f.data.iter().find(|d| d.id == id))
        && old.contract != chosen
    {
        impact.data.push(edit::ChangedData {
            owner: owner.into(),
            id,
            name: old.name.clone(),
            previous: old.contract.clone(),
        });
    }
    edit::reconcile(&mut q, &impact, chosen.as_ref());
    invalidate_reviews(p, &mut q);
    validate(&q)?;
    Ok((q, impact))
}
pub fn outcome_users(p: &Project, owner: &str, outcome: &str) -> Vec<String> {
    p.behavior
        .iter()
        .flat_map(|(scope, f)| {
            f.transitions
                .iter()
                .filter(move |t| {
                    t.outcome.as_deref() == Some(outcome)
                        && f.step(&t.from).and_then(|s| s.target.as_deref()) == Some(owner)
                })
                .map(move |t| format!("{scope} / {} / {}", t.from, t.id))
        })
        .collect()
}
/// Exact behavior references useful for bounded deletion blockers in either layer.
pub fn references(p: &Project, identity: &str) -> Vec<String> {
    let mut refs = vec![];
    for (owner, f) in &p.behavior {
        if owner == identity {
            refs.push(format!("{owner}: owned control flow"));
        }
        for s in &f.steps {
            if s.target.as_deref() == Some(identity) {
                refs.push(format!("{owner} / {}: call {}", s.id, s.name));
            }
        }
        for d in &f.data {
            if d.from.port.as_deref() == Some(identity)
                || d.to.port.as_deref() == Some(identity)
                || d.exchange.as_deref() == Some(identity)
            {
                refs.push(format!("{owner} / {}: information {}", d.id, d.name));
            }
        }
    }
    refs
}
