use serde_json::{Value, json};
use system_designer::{edit, model::*};
fn fixture() -> Project {
    parse(include_str!("fixtures/project.json")).expect("fixture")
}
fn invalid(f: impl FnOnce(&mut Value)) {
    let mut value: Value =
        serde_json::from_str(include_str!("fixtures/project.json")).expect("JSON");
    f(&mut value);
    assert!(parse(&value.to_string()).is_err());
}
#[test]
fn blank_root() {
    validate(&Project::blank()).expect("blank");
}
#[test]
fn fixture_with_feedback_loops() {
    validate(&fixture()).expect("feedback");
}
#[test]
fn full_json_round_trip() {
    let p = fixture();
    assert_eq!(
        parse(&serde_json::to_string(&p).expect("serialize")).expect("parse"),
        p
    );
}
#[test]
fn arbitrary_containment_depth() {
    let mut p = Project::blank();
    p.root = "s0".into();
    p.systems.clear();
    for i in 0..=2048 {
        p.systems.push(System {
            id: format!("s{i}"),
            nodes: if i < 2048 {
                vec![Node {
                    id: format!("n{i}"),
                    name: format!("n{i}"),
                    purpose: String::new(),
                    kind: Kind::Component,
                    ports: vec![],
                    child: Some(format!("s{}", i + 1)),
                }]
            } else {
                vec![]
            },
            edges: vec![],
        });
    }
    validate(&p).expect("deep valid");
    assert_eq!(p.descendants("s0").len(), 2049);
    assert_eq!(p.ancestors("s2048").len(), 2048);
}
#[test]
fn more_than_eight_allowed() {
    let mut p = Project::blank();
    for _ in 0..37 {
        p = edit::add_node(&p, "root").expect("add").0;
    }
    assert_eq!(p.system("root").expect("root").nodes.len(), 37);
    validate(&p).expect("large level");
}
#[test]
fn duplicate_node() {
    invalid(|p| {
        let n = p["systems"][0]["nodes"][0].clone();
        p["systems"][0]["nodes"]
            .as_array_mut()
            .expect("array")
            .push(n);
    });
}
#[test]
fn duplicate_port() {
    invalid(|p| p["systems"][0]["nodes"][0]["ports"][1]["id"] = json!("A.in"));
}
#[test]
fn duplicate_edge() {
    invalid(|p| p["systems"][0]["edges"][1]["id"] = json!("incoming"));
}
#[test]
fn structural_id_spaces_overlap() {
    invalid(|p| p["systems"][0]["nodes"][0]["id"] = json!("root"));
}
#[test]
fn missing_root() {
    invalid(|p| p["root"] = json!("missing"));
}
#[test]
fn missing_child() {
    invalid(|p| p["systems"][0]["nodes"][0]["child"] = json!("missing"));
}
#[test]
fn two_owners() {
    invalid(|p| p["systems"][0]["nodes"][1]["child"] = json!("a"));
}
#[test]
fn root_cannot_have_owner() {
    invalid(|p| p["systems"][2]["nodes"][0]["child"] = json!("root"));
}
#[test]
fn disconnected_cycle() {
    invalid(|p| {
        p["systems"].as_array_mut().expect("systems").extend([
            json!({
                "id":"cycle1",
                "nodes":[{
                    "id":"c1",
                    "name":"C1",
                    "purpose":"",
                    "kind":"component",
                    "ports":[],
                    "child":"cycle2"
                }],
                "edges":[]
            }),
            json!({
                "id":"cycle2",
                "nodes":[{
                    "id":"c2",
                    "name":"C2",
                    "purpose":"",
                    "kind":"component",
                    "ports":[],
                    "child":"cycle1"
                }],
                "edges":[]
            }),
        ]);
    });
}
#[test]
fn orphan_system() {
    invalid(|p| {
        p["systems"].as_array_mut().expect("systems").push(json!({
            "id":"orphan",
            "nodes":[],
            "edges":[]
        }))
    });
}
#[test]
fn cross_level_rejected() {
    invalid(|p| {
        p["systems"][2]["edges"][0]["from"] = json!({
            "node":"B",
            "port":"B.out"
        })
    });
}
#[test]
fn missing_endpoint() {
    invalid(|p| p["systems"][0]["edges"][0]["from"]["node"] = json!("gone"));
}
#[test]
fn wrong_port_owner() {
    invalid(|p| p["systems"][0]["edges"][0]["from"]["port"] = json!("A.out"));
}
#[test]
fn wrong_direction() {
    invalid(|p| p["systems"][0]["edges"][0]["from"]["port"] = json!("B.in"));
}
#[test]
fn wrong_boundary_direction() {
    invalid(|p| p["systems"][1]["edges"][0]["from"]["port"] = json!("A.out"));
}
#[test]
fn invented_root_boundary() {
    invalid(|p| p["systems"][0]["edges"][0]["from"]["node"] = Value::Null);
}
#[test]
fn missing_contract() {
    invalid(|p| p["systems"][0]["nodes"][0]["ports"][0]["contract"]["id"] = json!("gone"));
}
#[test]
fn mismatched_version() {
    invalid(|p| p["systems"][0]["nodes"][0]["ports"][0]["contract"]["version"] = json!(2));
}
#[test]
fn connected_null_rejected() {
    invalid(|p| p["systems"][0]["nodes"][0]["ports"][0]["contract"] = Value::Null);
}
#[test]
fn unassigned_unconnected_port_allowed() {
    let p = fixture();
    let q = edit::add_port(&p, "B", Direction::In).expect("draft");
    assert!(
        q.node("B")
            .expect("B")
            .1
            .ports
            .last()
            .expect("port")
            .contract
            .is_none()
    );
}
#[test]
fn missing_contract_field_is_not_null() {
    invalid(|p| {
        p["systems"][0]["nodes"][0]["ports"][0]
            .as_object_mut()
            .expect("port")
            .remove("contract");
    });
}
#[test]
fn missing_endpoint_node_is_not_boundary() {
    invalid(|p| {
        p["systems"][0]["edges"][0]["from"]
            .as_object_mut()
            .expect("endpoint")
            .remove("node");
    });
}
#[test]
fn explicit_null_child_rejected() {
    invalid(|p| p["systems"][2]["nodes"][0]["child"] = Value::Null);
}
#[test]
fn unknown_project_field() {
    invalid(|p| p["authority"] = json!("invented"));
}
#[test]
fn duplicate_public_boundary_rejected() {
    invalid(|p| p["systems"][1]["ports"] = json!([]));
}
#[test]
fn invented_edge_contract_rejected() {
    invalid(|p| {
        p["systems"][0]["edges"][0]["contract"] = json!({
            "id":"T",
            "version":1
        })
    });
}
#[test]
fn unknown_shape_keyword() {
    invalid(|p| p["contracts"][0]["definition"]["minimum"] = json!(0));
}
#[test]
fn invalid_schema_duplicate_field() {
    invalid(|p| {
        let f = p["contracts"][0]["definition"]["fields"][0].clone();
        p["contracts"][0]["definition"]["fields"]
            .as_array_mut()
            .expect("fields")
            .push(f);
    });
}
#[test]
fn empty_enum() {
    invalid(|p| {
        p["contracts"][0]["definition"] = json!({
            "type":"enum",
            "values":[]
        })
    });
}
#[test]
fn duplicate_enum_value() {
    invalid(|p| {
        p["contracts"][0]["definition"] = json!({
            "type":"enum",
            "values":["a","a"]
        })
    });
}
#[test]
fn all_shape_forms() {
    let mut p = fixture();
    p.contracts[0].definition = Shape::Array {
        items: Box::new(Shape::Object {
            fields: vec![
                Field {
                    name: "a".into(),
                    required: false,
                    description: Some("Intent".into()),
                    schema: Shape::Enum {
                        values: vec!["x".into(), "y".into()],
                    },
                },
                Field {
                    name: "b".into(),
                    required: true,
                    description: None,
                    schema: Shape::Boolean,
                },
                Field {
                    name: "n".into(),
                    required: true,
                    description: None,
                    schema: Shape::Number,
                },
                Field {
                    name: "i".into(),
                    required: true,
                    description: None,
                    schema: Shape::Integer,
                },
            ],
        }),
    };
    assert_eq!(
        parse(&serde_json::to_string(&p).expect("serialize")).expect("parse"),
        p
    );
}
#[test]
fn unsafe_integer_version() {
    invalid(|p| p["contracts"][0]["version"] = json!(MAX_SAFE_INTEGER + 1));
}
#[test]
fn negative_layout() {
    invalid(|p| {
        p["layout"] = json!({
            "root":{
                "A":{
                    "x":-1,
                    "y":0
                }
            }
        })
    });
}
#[test]
fn nonlocal_layout() {
    invalid(|p| {
        p["layout"] = json!({
            "root":{
                "W":{
                    "x":1,
                    "y":0
                }
            }
        })
    });
}
#[test]
fn boundary_mirrors_owner() {
    let mut p = fixture();
    p.node_mut("A").expect("A").ports[0].name = "Changed".into();
    assert_eq!(p.boundary("a")[0].name, "Changed");
    assert_eq!(
        p.effective_direction(
            "a",
            &Endpoint {
                node: None,
                port: "A.in".into()
            }
        ),
        Some(Direction::Out)
    );
}
#[test]
fn external_labels_derived() {
    let mut p = fixture();
    p.node_mut("B").expect("B").name = "New source".into();
    assert_eq!(
        p.external_connections("a")[0]["links"][0]["name"],
        "New source"
    );
    assert_eq!(
        p.external_connections("x")[0]["links"][0]["name"],
        "Parent boundary"
    );
}
#[test]
fn create_only_one_child() {
    let p = fixture();
    assert!(edit::create_child(&p, "A").is_err());
    let (q, id) = edit::create_child(&p, "B").expect("child");
    assert_eq!(q.owner(&id).expect("owner").1.id, "B");
}
#[test]
fn safe_recursive_node_delete() {
    let p = fixture();
    let q = edit::delete_node(&p, "A").expect("delete");
    assert_eq!(q.systems.len(), 1);
    assert!(q.system("root").expect("root").edges.is_empty());
    assert_eq!(p.systems.len(), 3);
}
#[test]
fn removing_internals_keeps_owner() {
    let q = edit::delete_child(&fixture(), "A").expect("delete");
    assert_eq!(q.systems.len(), 1);
    assert_eq!(q.system("root").expect("root").edges.len(), 2);
    assert_eq!(q.node("A").expect("A").1.ports.len(), 2);
}
#[test]
fn port_delete_cleans_projected_edges() {
    let q = edit::delete_port(&fixture(), "A", "A.in").expect("delete");
    assert_eq!(q.system("root").expect("root").edges.len(), 1);
    assert_eq!(q.system("a").expect("a").edges.len(), 1);
    assert_eq!(q.system("x").expect("x").edges.len(), 2);
}
#[test]
fn undo_redo_snapshot() {
    let p = fixture();
    let mut s = edit::Store::new(p.clone()).expect("store");
    s.publish("Delete", edit::delete_node(&p, "A").expect("delete"))
        .expect("publish");
    assert!(s.undo());
    assert_eq!(s.project(), &p);
    assert!(s.redo());
    assert_eq!(s.project().systems.len(), 1);
}
#[test]
fn invalid_publication_preserves_redo() {
    let p = fixture();
    let mut s = edit::Store::new(p.clone()).expect("store");
    s.publish("Delete", edit::delete_node(&p, "A").expect("delete"))
        .expect("publish");
    s.undo();
    let mut bad = p.clone();
    bad.root = "missing".into();
    assert!(s.publish("bad", bad).is_err());
    assert_eq!(s.project(), &p);
    assert!(s.redo());
}
#[test]
fn history_is_bounded() {
    let mut s = edit::Store::new(Project::blank()).expect("store");
    for i in 0..60 {
        let mut q = s.project().clone();
        q.name = format!("Revision {i}");
        s.publish("rename", q).expect("publish");
    }
    let mut count = 0;
    while s.undo() {
        count += 1;
    }
    assert_eq!(count, 50);
}
#[test]
fn delete_referenced_type_rejected() {
    assert!(
        edit::delete_contract(
            &fixture(),
            &ContractRef {
                id: "T".into(),
                version: 1
            }
        )
        .is_err()
    );
}
#[test]
fn explicit_shared_definition_confirmation() {
    let p = fixture();
    let mut c = p.contracts[0].clone();
    let r = c.reference();
    c.purpose = "New meaning".into();
    assert!(edit::save_contract(&p, c.clone(), Some(&r), false).is_err());
    assert!(edit::save_contract(&p, c, Some(&r), true).is_ok());
}

#[test]
fn frame_snapshot_remains_immutable_after_publication() {
    let p = fixture();
    let mut store = edit::Store::new(p.clone()).expect("store");
    let before = store.snapshot();
    let same = store.snapshot();
    assert!(std::sync::Arc::ptr_eq(&before, &same));
    let q = edit::delete_node(&p, "A").expect("candidate");
    store.publish("Delete", q).expect("publish");
    assert_eq!(before.as_ref(), &p);
    assert_ne!(store.project(), before.as_ref());
}
