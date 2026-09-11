use system_designer::{
    APPLICATION_DESIGN, INITIALIZATION,
    exchange::{self, Scope},
    model::*,
};
#[test]
fn embedded_design_is_a_real_project() {
    let p = parse(APPLICATION_DESIGN).expect("self design");
    assert_eq!(p.system(&p.root).expect("root").nodes.len(), 7);
    assert_eq!(p.systems.len(), 8);
    assert!(p.systems.iter().all(|s| s.nodes.len() <= 8));
}
#[test]
fn self_design_all_scopes_round_trip() {
    let p = parse(APPLICATION_DESIGN).expect("project");
    for s in &p.systems {
        for scope in [Scope::Level, Scope::Subtree] {
            let packet = exchange::export(&p, &s.id, scope, None).expect("export");
            let q = exchange::replace(&p, &packet, &s.id).expect("replace");
            validate(&q).expect("valid result");
        }
        for n in &s.nodes {
            let packet =
                exchange::export(&p, &s.id, Scope::Component, Some(&n.id)).expect("component");
            exchange::replace(&p, &packet, &s.id).expect("replace");
        }
    }
}
#[test]
fn blank_start_does_not_load_self_design() {
    let p = Project::blank();
    assert!(p.contracts.is_empty());
    assert!(p.systems[0].nodes.is_empty());
}
#[test]
fn initialization_embeds_exact_protocol() {
    for text in [
        "system-designer-project",
        "system-designer-scope",
        "component",
        "level",
        "subtree",
        "base",
        "context",
        "eight",
    ] {
        assert!(INITIALIZATION.contains(text), "missing {text}");
    }
    assert!(!INITIALIZATION.to_lowercase().contains("foundry"));
}
