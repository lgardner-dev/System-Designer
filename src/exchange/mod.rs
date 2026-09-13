mod canonical;
mod packet;
mod replace;
pub use canonical::{canonical_json, digest};
pub use packet::{Scope, export};
pub use replace::replace;

/// Dispatch by exact packet format while retaining the caller's explicit targets.
/// Full projects and unknown formats must use their own lifecycle, never a scope edit.
pub fn replace_packet(
    p: &crate::model::Project,
    value: &serde_json::Value,
    interface_system: &str,
    behavior_owner: &str,
) -> crate::model::Result<crate::model::Project> {
    match value.get("format").and_then(serde_json::Value::as_str) {
        Some("system-designer-scope") => replace(p, value, interface_system),
        Some("system-designer-behavior-scope") => {
            crate::behavior::replace(p, value, behavior_owner)
        }
        _ => Err(crate::model::ModelError::one(
            "Unknown scope packet format; expected system-designer-scope or system-designer-behavior-scope",
        )),
    }
}
