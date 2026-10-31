#![warn(missing_docs)]

//! Core runtime library for Higher-Kinded Data (HKD) formwork structures.
//!
//! # Architecture Overview
//!
//! `formwork-core` provides runtime foundations for structures that separate a
//! wet, choice-bearing formwork from a cured, watertight dry domain model.
//!
//! ## The Concrete Metaphor
//! - **Formwork**: The temporary mold or structural frame holding incomplete data and pending choices.
//! - **Wet State (`*Wet`, `Slot::FixedWet`, `FormworkError::WetChoice`)**: Data in the fluid, mutable stage where decisions are pending, choices have selection histories, and constraints are evaluated.
//! - **Dry State (`*Dry`, `Slot::FixedDry`, `Formwork::Dry`)**: The solid, cured, permanent domain model. Watertight, rigid, zero optional wrappers, zero choice state.
//! - **Resolution**: Consuming the wet formwork to produce the watertight dry domain model via [`.resolve()`](crate::formwork_trait::Formwork::resolve) (and [`.resolve_exact()`](crate::choice::SingleChoice::resolve_exact) for scalar choices and single slots).
//!
//! Types in this crate are strictly divided into two categories:
//!
//! ## 1. User-Facing Types
//!
//! These types form the public interface for defining, inspecting, and manipulating
//! choices and wet formwork structures:
//!
//! | Type | Description |
//! | :--- | :--- |
//! | [`SingleChoice`] | A choice where exactly one option must be selected from a list of candidates. |
//! | [`MultiChoice`] | A choice where multiple options are selected according to a [`Cardinality`] constraint. |
//! | [`Cardinality`] | Specifies selection limits on multi-choices (`Exact(n)`, `UpTo(n)`, `AtLeast(n)`). |
//! | [`Formwork`] | The primary trait implemented by wet formworks (`<Name>Formwork`), providing completeness checks, resolution into watertight dry models, and tree traversal. |
//! | [`ChoiceVisitor`] | Trait for implementing custom choice traversal logic across a formwork tree. |
//! | [`VisitorFlow`] | Control flow signal (`Continue`, `Break`, or `Back`) returned during choice visitation. |
//! | [`FormworkSession`] | An interactive session driving choices one-by-one with caller control and rewind support. |
//! | [`StepResult`] | Outcome of stepping through an interactive session (`Choice`, `Complete`, or `AtStart`). |
//! | [`AutoSelectFirst`], [`ConfirmAll`], [`UnmakeAll`] | Built-in visitor implementations for common operations (batch selection, unmaking, re-confirming). |
//! | [`FormworkError`] | Error type returned when choice constraints are violated or an incomplete formwork is resolved. |
//!
//! ### When to interact with User-Facing Types
//! - Pass [`SingleChoice`] or [`MultiChoice`] directly to macro-generated builder methods (e.g. `.with_<field>(SingleChoice::new(...))`).
//! - Query choice states and prompts in UI layers or interactive CLIs.
//! - Call [`Formwork::walk_choices`] with a [`ChoiceVisitor`] to traverse, display, or automate decisions.
//! - Call [`Formwork::resolve`] to produce the final, watertight dry domain model.
//!
//! ## 2. Internal Plumbing Types
//!
//! | Type | Role |
//! | :--- | :--- |
//! | [`Slot`], [`SingleSlot`], [`MultiSlot`] | Generic enums and type aliases storing slot data across wet and dry modes. |
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
//! > user-facing choices ([`SingleChoice`], [`MultiChoice`]), and the [`Formwork`] trait.

/// Choice abstractions, state machines, and cardinality constraints.
pub mod choice;
/// Error types returned during formwork choice manipulation and resolution.
pub mod error;
/// Traits and level structures defining formwork behavior and traversal.
pub mod formwork_trait;
/// Internal slot representations and containers.
pub mod slot;
/// Visitor pattern implementation for inspecting and manipulating choices.
pub mod visitor;

pub use choice::{Cardinality, ChoiceState, MultiChoice, SingleChoice};
pub use error::FormworkError;
pub use formwork_trait::{Formwork, FormworkSession, Level, StepOutcome, StepResult, Walkable};
pub use slot::{MultiSlot, SingleSlot, Slot};
pub use visitor::{AutoSelectFirst, ChoiceVisitor, ConfirmAll, UnmakeAll, VisitorFlow};
