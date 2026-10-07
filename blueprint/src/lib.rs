//! HKD Blueprint library proof of concept.
//!
//! This crate contains the manually written proof-of-concept for the macro-generated
//! blueprint architecture, demonstrating:
//! - Localized `Mode` traits with `Draft` and `Final` mode implementations per struct.
//! - Watertight domain types in `Final` mode.
//! - Stateful `Slot` and `SlotCollection` types (`Choose`, `Fixed`).
//! - Breadth-first visitor traversal and choice selection/unmaking with history preservation.
//! - Comprehensive abstract (`Root`/`Branch`/`Leaf`) and D&D 5e (`Race`/`Trait`/`Effect`) models.

pub mod cardinality;
pub mod choice;
pub mod error;
pub mod visitor;
pub mod slot;
pub mod blueprint_trait;
pub mod abstract_model;
pub mod dnd_model;

pub use cardinality::Cardinality;
pub use choice::{Choice, ChoiceCollection, ChoiceOption};
pub use error::BlueprintError;
pub use visitor::{
    AutoSelectFirst, ChoiceCollectionVisitor, ChoiceVisitor,
    ConfirmAll, Counter, TargetUnmaker, UnmakeAll, VisitorFlow, StepBack,
};
pub use slot::{Slot, SlotCollection};
pub use blueprint_trait::{Blueprint, Level, Walkable};
