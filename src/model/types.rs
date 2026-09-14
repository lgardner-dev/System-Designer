use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
pub const FORMAT: &str = "system-designer-project";
pub const SCOPE_FORMAT: &str = "system-designer-scope";
pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub type Layout = BTreeMap<String, BTreeMap<String, Position>>;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub format: String,
    pub version: u32,
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub root: String,
    pub contracts: Vec<Contract>,
    pub systems: Vec<System>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub layout: Layout,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub behavior: BTreeMap<String, crate::behavior::Flow>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub flow_layout: Layout,
}
impl Project {
    pub fn blank() -> Self {
        Self {
            format: FORMAT.into(),
            version: 1,
            id: format!("project.{}", uuid::Uuid::new_v4()),
            name: "Untitled system".into(),
            purpose: String::new(),
            root: "root".into(),
            contracts: vec![],
            systems: vec![System {
                id: "root".into(),
                nodes: vec![],
                edges: vec![],
            }],
            layout: Layout::new(),
            behavior: BTreeMap::new(),
            flow_layout: Layout::new(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct System {
    pub id: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub kind: Kind,
    pub ports: Vec<Port>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_string"
    )]
    pub child: Option<String>,
}
fn present_string<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}
// An explicit null draft binding is valid; omitting the required binding is not.
fn nullable<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    Option::<T>::deserialize(d)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Component,
    Work,
    Decision,
    Authority,
    Controller,
    Record,
    Success,
    Failure,
}
impl Kind {
    pub const ALL: [Self; 8] = [
        Self::Component,
        Self::Work,
        Self::Decision,
        Self::Authority,
        Self::Controller,
        Self::Record,
        Self::Success,
        Self::Failure,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::Work => "work",
            Self::Decision => "decision",
            Self::Authority => "authority",
            Self::Controller => "controller",
            Self::Record => "record",
            Self::Success => "success",
            Self::Failure => "failure",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    #[serde(rename = "in")]
    In,
    #[serde(rename = "out")]
    Out,
}
impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Self::In => Self::Out,
            Self::Out => Self::In,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::In => "Input",
            Self::Out => "Output",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractRef {
    pub id: String,
    pub version: u64,
}
impl std::fmt::Display for ContractRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.id, self.version)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub id: String,
    pub name: String,
    pub direction: Direction,
    #[serde(deserialize_with = "nullable")]
    pub contract: Option<ContractRef>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    #[serde(deserialize_with = "nullable")]
    pub node: Option<String>,
    pub port: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub id: String,
    pub from: Endpoint,
    pub to: Endpoint,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_string"
    )]
    pub label: Option<String>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
/// Version-3 layout range keeps practical f32 scene precision at supported zoom.
pub const MAX_LAYOUT_COORDINATE: f64 = 1_000_000.0;
impl Position {
    pub fn valid_for(self, version: u32) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && match version {
                1 | 2 => self.x >= 0.0 && self.y >= 0.0,
                3 => self.x.abs() <= MAX_LAYOUT_COORDINATE && self.y.abs() <= MAX_LAYOUT_COORDINATE,
                _ => false,
            }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub id: String,
    pub version: u64,
    pub name: String,
    pub purpose: String,
    pub definition: Shape,
}
impl Contract {
    pub fn reference(&self) -> ContractRef {
        ContractRef {
            id: self.id.clone(),
            version: self.version,
        }
    }
    pub fn draft(id: String) -> Self {
        Self {
            id,
            version: 1,
            name: "New contract".into(),
            purpose: String::new(),
            definition: Shape::Object { fields: vec![] },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Shape {
    String,
    Integer,
    Number,
    Boolean,
    Enum { values: Vec<String> },
    Object { fields: Vec<Field> },
    Array { items: Box<Shape> },
}
impl Shape {
    pub fn label(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Enum { .. } => "enum",
            Self::Object { .. } => "object",
            Self::Array { .. } => "array",
        }
    }
    pub fn of_type(s: &str) -> Self {
        match s {
            "integer" => Self::Integer,
            "number" => Self::Number,
            "boolean" => Self::Boolean,
            "enum" => Self::Enum {
                values: vec!["value".into()],
            },
            "object" => Self::Object { fields: vec![] },
            "array" => Self::Array {
                items: Box::new(Self::String),
            },
            _ => Self::String,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub required: bool,
    pub schema: Shape,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_string"
    )]
    pub description: Option<String>,
}
