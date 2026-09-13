//! Authoritative System Designer model. No renderer, network client, or web runtime.
pub mod edit;
pub mod exchange;
pub mod model;
pub mod storage;
#[cfg(feature = "desktop")]
pub mod ui;
/// These resources are part of the executable, never runtime sidecar requirements.
pub const INITIALIZATION: &str = include_str!("../assets/initialization.txt");
pub const APPLICATION_DESIGN: &str = include_str!("../design/system-designer.project.json");

pub mod behavior;
