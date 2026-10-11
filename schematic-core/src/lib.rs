//! Core runtime library for Higher-Kinded Data (HKD) schematic blueprints.
//!
//! # Architecture Overview
//!
//! `schematic-core` provides the runtime foundations for schemas that separate an
//! unresolved, choice-bearing blueprint from a watertight, resolved domain model.
//!
//! Types in this crate are strictly divided into two categories:
//!
//! ## 1. User-Facing Types
//!
//! These types form the public interface for defining, inspecting, and manipulating
//! choices and unresolved schematics:
//!
//! | Type | Description |
//! | :--- | :--- |
//! | [`SingleChoice`] | A choice where exactly one option must be selected from a list of candidates. |
//! | [`MultiChoice`] | A choice where multiple options are selected according to a [`Cardinality`] constraint. |
//! | [`Cardinality`] | Specifies selection limits on multi-choices (`Exact(n)`, `UpTo(n)`, `AtLeast(n)`). |
//! | [`Schematic`] | The primary trait implemented by unresolved blueprints (`<Name>Schematic`), providing completeness checks, resolution into watertight models, and tree traversal. |
//! | [`ChoiceVisitor`] | Trait for implementing custom choice traversal logic across a schematic tree. |
//! | [`VisitorFlow`] | Control flow signal (`Continue` or `Break`) returned during choice visitation. |
//! | [`AutoSelectFirst`], [`ConfirmAll`], [`UnmakeAll`] | Built-in visitor implementations for common operations (batch selection, unmaking, re-confirming). |
//! | [`SchematicError`] | Error type returned when choice constraints are violated or an incomplete schematic is resolved. |
//!
//! ### When to interact with User-Facing Types
//! - Pass [`SingleChoice`] or [`MultiChoice`] directly to macro-generated builder methods (e.g. `.with_<field>(SingleChoice::new(...))`).
//! - Query choice states and prompts in UI layers or interactive CLIs.
//! - Call [`Schematic::walk_choices`] with a [`ChoiceVisitor`] to traverse, display, or automate decisions.
//! - Call [`Schematic::resolve`] to produce the final, watertight domain model.
//!
//! ## 2. Internal Plumbing Types
//!
//! | Type | Role |
//! | :--- | :--- |
//! | [`Slot`], [`SingleSlot`], [`MultiSlot`] | Generic enums and type aliases storing slot data across unresolved and resolved modes. |
//! | [`ChoiceState`] | Low-level state enum tracking index selections and unmade history. |
//! | [`Walkable`], [`Level`] | Recursive traversal mechanics for breadth-first visitor execution. |
//!
//! > [!WARNING]
//! > **Internal Plumbing Types**
//! >
//! > If your application code or end users ever interact directly with [`Slot`], [`SingleSlot`],
//! > [`MultiSlot`], [`ChoiceState`], or [`Walkable`], **something is wrong in the API usage**.
//! >
//! > The macro-generated builder and conversion traits automatically wrap raw values and choices
//! > into the appropriate internal slot structures. Always interact through generated builders,
//! > user-facing choices ([`SingleChoice`], [`MultiChoice`]), and the [`Schematic`] trait.

pub mod choice;
pub mod error;
pub mod schematic_trait;
pub mod slot;
pub mod visitor;

pub use choice::{Cardinality, ChoiceState, MultiChoice, SingleChoice};
pub use error::SchematicError;
pub use schematic_trait::{Level, Schematic, Walkable};
pub use slot::{MultiSlot, SingleSlot, Slot};
pub use visitor::{AutoSelectFirst, ChoiceVisitor, ConfirmAll, UnmakeAll, VisitorFlow};
