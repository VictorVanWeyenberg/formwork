//! HKD Blueprint library proof of concept.
//!
//! This crate contains the manually written proof-of-concept for the macro-generated
//! blueprint architecture, demonstrating:
//! - Localized `Mode` traits with `Draft` and `Final` mode implementations per struct.
//! - Watertight domain types in `Final` mode.
//! - Stateful `Slot` types (`Choose`, `Fixed`).
//! - Breadth-first visitor traversal and choice selection/unmaking with history preservation.
//! - Comprehensive abstract (`Root`/`Branch`/`Leaf`) and D&D 5e (`Race`/`Trait`/`Effect`) models.

pub mod choice;
pub mod error;
pub mod visitor;
pub mod slot;
pub mod blueprint_trait;
pub mod abstract_model;
pub mod dnd_model;


pub use choice::{Choice, ChoiceState, Cardinality};
pub use error::BlueprintError;
pub use visitor::{
    AutoSelectFirst, ChoiceVisitor,
    ConfirmAll, UnmakeAll, VisitorFlow,
};
pub use slot::Slot;
pub use blueprint_trait::{Blueprint, Level, Walkable};
