use serde_json::{Value, json};
use system_designer::{
    edit,
    exchange::{self, Scope},
    model::*,
};
fn fixture() -> Project {
    parse(include_str!("fixtures/project.json")).expect("fixture")
}
fn load(name: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name),
        )
        .expect("read"),
    )
    .expect("json")
}
#[test]
fn original_export_bytes_and_hashes() {
    let p = fixture();
    let cases = load("exports.json");
    for c in cases.as_array().expect("cases") {
        let scope = match c["scope"].as_str().expect("scope") {
            "component" => Scope::Component,
            "level" => Scope::Level,
            _ => Scope::Subtree,
        };
        let actual = exchange::export(
            &p,
            c["system"].as_str().expect("sid"),
            scope,
            c["node"].as_str(),
        )
        .expect("export");
        let expected = load(c["file"].as_str().expect("file"));
        assert_eq!(actual, expected, "{}", c["file"]);
    }
}
#[test]
fn unicode_protocol_matches_reference() {
    let p: Project = serde_json::from_value(load("unicode-project.json")).expect("project");
    assert_eq!(
        exchange::export(&p, "root", Scope::Component, Some("A")).expect("export"),
        load("unicode-scope.json")
    );
}
#[test]
fn original_replacement_matches_reference() {
    let p = fixture();
    let actual = exchange::replace(&p, &load("replacement.json"), "a").expect("replace");
    let expected: Project =
        serde_json::from_value(load("replacement-result.json")).expect("project");
    assert_eq!(actual, expected);
}
#[test]
fn stale_local_input_rejected() {
    let mut p = fixture();
    let packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    p.node_mut("X").expect("X").purpose = "changed".into();
    assert!(exchange::replace(&p, &packet, "a").is_err());
}
#[test]
fn stale_ancestor_rejected() {
    let mut p = fixture();
    let packet = exchange::export(&p, "x", Scope::Level, None).expect("export");
    p.node_mut("A").expect("A").purpose = "changed".into();
    assert!(exchange::replace(&p, &packet, "x").is_err());
}
#[test]
fn independent_hidden_child_edit_survives() {
    let mut p = fixture();
    let packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    p.node_mut("W").expect("W").purpose = "independent hidden change".into();
    let q = exchange::replace(&p, &packet, "a").expect("replace");
    assert_eq!(
        q.node("W").expect("W").1.purpose,
        "independent hidden change"
    );
}
#[test]
fn boundary_is_read_only() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Subtree, None).expect("export");
    packet["boundary"]["name"] = json!("invented owner");
    assert!(exchange::replace(&p, &packet, "a").is_err());
}
#[test]
fn context_is_read_only() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    packet["context"]["project"]["name"] = json!("invented context");
    assert!(exchange::replace(&p, &packet, "a").is_err());
}
#[test]
fn scope_import_cannot_rewrite_catalog() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    packet["contracts"][0]["purpose"] = json!("silent new meaning");
    assert!(exchange::replace(&p, &packet, "a").is_err());
}
#[test]
fn level_cannot_delete_hidden_child() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    packet["systems"][0]["nodes"] = json!([]);
    packet["systems"][0]["edges"] = json!([]);
    assert!(exchange::replace(&p, &packet, "a").is_err());
}
#[test]
fn subtree_can_remove_internal_descendants() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Subtree, None).expect("export");
    packet["systems"] = json!([{
        "id":"a",
        "nodes":[],
        "edges":[]
    }]);
    let q = exchange::replace(&p, &packet, "a").expect("replace");
    assert_eq!(q.systems.len(), 2);
    assert_eq!(q.system("root"), p.system("root"));
}
#[test]
fn component_preserves_siblings_and_internals() {
    let p = fixture();
    let mut packet = exchange::export(&p, "root", Scope::Component, Some("A")).expect("export");
    packet["component"]["purpose"] = json!("reviewed");
    let q = exchange::replace(&p, &packet, "root").expect("replace");
    assert_eq!(q.node("B").map(|(_, n)| n), p.node("B").map(|(_, n)| n));
    assert_eq!(q.system("a"), p.system("a"));
    assert_eq!(q.node("A").expect("A").1.purpose, "reviewed");
}
#[test]
fn component_cannot_break_wires() {
    let p = fixture();
    let mut packet = exchange::export(&p, "root", Scope::Component, Some("A")).expect("export");
    packet["component"]["ports"] = json!([]);
    assert!(exchange::replace(&p, &packet, "root").is_err());
}
#[test]
fn component_cannot_change_child() {
    let p = fixture();
    let mut packet = exchange::export(&p, "root", Scope::Component, Some("A")).expect("export");
    packet["component"]
        .as_object_mut()
        .expect("node")
        .remove("child");
    assert!(exchange::replace(&p, &packet, "root").is_err());
}
#[test]
fn missing_referenced_definition_rejected() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    packet["contracts"] = json!([]);
    assert!(exchange::replace(&p, &packet, "a").is_err());
}
#[test]
fn new_contract_versions_can_be_added() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    packet["contracts"]
        .as_array_mut()
        .expect("catalog")
        .push(json!({
            "id":"New",
            "version":1,
            "name":"New",
            "purpose":"",
            "definition":{
                "type":"boolean"
            }
        }));
    assert!(exchange::replace(&p, &packet, "a").is_ok());
}
#[test]
fn wrong_target_rejected() {
    let p = fixture();
    let packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    assert!(exchange::replace(&p, &packet, "x").is_err());
}
#[test]
fn wrong_project_rejected() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    packet["projectId"] = json!("other");
    assert!(exchange::replace(&p, &packet, "a").is_err());
}
#[test]
fn full_project_rejected_from_scope_action() {
    let p = fixture();
    assert!(exchange::replace(&p, &serde_json::to_value(&p).expect("value"), "root").is_err());
}
#[test]
fn unknown_packet_property_rejected() {
    let p = fixture();
    let mut packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    packet["override"] = json!(true);
    assert!(exchange::replace(&p, &packet, "a").is_err());
}
#[test]
fn canvas_positions_do_not_stale_scope() {
    let mut p = fixture();
    let packet = exchange::export(&p, "a", Scope::Level, None).expect("export");
    p.layout.insert(
        "a".into(),
        [("X".into(), Position { x: 20.0, y: 40.0 })].into(),
    );
    let q = exchange::replace(&p, &packet, "a").expect("replace");
    assert_eq!(q.layout, p.layout);
}
#[test]
fn contract_retype_matches_original() {
    let p = fixture();
    let request = edit::Connection {
        id: Some("incoming".into()),
        from: Endpoint {
            node: Some("B".into()),
            port: "B.out".into(),
        },
        to: Endpoint {
            node: Some("A".into()),
            port: "A.in".into(),
        },
        label: "Updated channel".into(),
        contract: ContractRef {
            id: "T".into(),
            version: 2,
        },
        new_contract: None,
        consent: true,
    };
    let q = edit::connect(&p, "root", &request).expect("retype");
    let expected: Project =
        serde_json::from_value(load("connection-result.json")).expect("project");
    assert_eq!(q, expected);
    assert_eq!(
        q.node("W").expect("W").1.ports[1]
            .contract
            .as_ref()
            .expect("T")
            .version,
        1
    );
}
#[test]
fn connected_retype_requires_consent() {
    let p = fixture();
    let request = edit::Connection {
        id: Some("incoming".into()),
        from: Endpoint {
            node: Some("B".into()),
            port: "B.out".into(),
        },
        to: Endpoint {
            node: Some("A".into()),
            port: "A.in".into(),
        },
        label: String::new(),
        contract: ContractRef {
            id: "T".into(),
            version: 2,
        },
        new_contract: None,
        consent: false,
    };
    assert!(edit::connect(&p, "root", &request).is_err());
}
#[test]
fn first_assignment_creates_contract_and_edge_together() {
    let p = fixture();
    let p = edit::add_port(&p, "A", Direction::Out).expect("port");
    let from = p
        .node("A")
        .expect("A")
        .1
        .ports
        .last()
        .expect("port")
        .id
        .clone();
    let p = edit::add_port(&p, "B", Direction::In).expect("port");
    let to = p
        .node("B")
        .expect("B")
        .1
        .ports
        .last()
        .expect("port")
        .id
        .clone();
    let c = Contract::draft("Fresh".into());
    let request = edit::Connection {
        id: None,
        from: Endpoint {
            node: Some("A".into()),
            port: from,
        },
        to: Endpoint {
            node: Some("B".into()),
            port: to,
        },
        label: "first".into(),
        contract: c.reference(),
        new_contract: Some(c),
        consent: false,
    };
    let q = edit::connect(&p, "root", &request).expect("connect");
    assert_eq!(q.contracts.len(), p.contracts.len() + 1);
    assert_eq!(q.system("root").expect("root").edges.len(), 3);
    assert_eq!(p.system("root").expect("root").edges.len(), 2);
}
