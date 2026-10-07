//! HKD Schematic library proof of concept.
//!
//! This crate contains the manually written proof-of-concept for the macro-generated
//! schematic architecture, demonstrating:
//! - Localized `Mode` traits with `Unresolved` and `Resolved` mode implementations per struct.
//! - Watertight domain types in `Resolved` mode.
//! - Stateful `Slot` types (`Choose`, `Fixed`).
//! - Breadth-first visitor traversal and choice selection/unmaking with history preservation.
//! - Comprehensive abstract (`Root`/`Branch`/`Leaf`) and D&D 5e (`Race`/`Trait`/`Effect`) models.

pub mod choice;
pub mod error;
pub mod visitor;
pub mod slot;
pub mod schematic_trait;


pub use choice::{Choice, ChoiceState, Cardinality};
pub use error::SchematicError;
pub use visitor::{
    AutoSelectFirst, ChoiceVisitor,
    ConfirmAll, UnmakeAll, VisitorFlow,
};
pub use slot::Slot;
pub use schematic_trait::{Schematic, Level, Walkable};
