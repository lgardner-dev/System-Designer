//! Design-time control flow. Never executes predicates, calls, or project code.
mod changes;
mod edit;
pub use changes::BehaviorChanges;
pub use edit::*;
mod exchange;
mod extract;
mod layout;
mod model;
mod validate;
pub use exchange::{export, replace};
pub use extract::{Extraction, Region, apply, candidates, preview};
pub use layout::{effective_positions, footprint, positions};
pub use model::*;
pub use validate::{check, complexity, issues};
