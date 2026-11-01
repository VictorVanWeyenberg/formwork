#![warn(missing_docs)]

//! # Formwork: Higher-Kinded Data (HKD) Formworks for Rust
//!
//! `formwork` separates fluid, choice-bearing wet formworks from solid, watertight dry domain models.
//!
//! # Architecture: Wet Formwork vs. Watertight Dry Domain Model
//!
//! Complex domain systems (such as tabletop RPG character creators, dynamic product configurators,
//! or tax/financial planning engines) require two very different views of the same data:
//!
//! 1. **The Wet Formwork (Fluid Representation)**:
//!    During creation or editing, data is incomplete and fluid. Fields contain open questions, pending choices,
//!    or cascading nested selections. We need to inspect pending choices, auto-fill defaults, or revert
//!    selections without destroying the history of choices.
//! 2. **The Dry Domain Model (Cured Representation)**:
//!    Once completed and validated, application logic (combat engines, invoicing systems, analytics)
//!    needs a **watertight** struct. No `Option`s, no choice enums, no runtime unwraps.
//!
//! ## Terminology
//! - **Formwork**: The temporary mold or structural frame holding incomplete data and pending choices.
//! - **Wet State (`*Wet`, `Slot::FixedWet`, `FormworkError::WetChoice`)**: Data in the fluid, mutable stage where decisions are pending, choices have selection histories, and constraints are evaluated.
//! - **Dry State (`*Dry`, `Slot::FixedDry`, `Formwork::Dry`)**: The solid, cured, permanent domain model. Watertight, rigid, zero optional wrappers, zero choice state.
//! - **Resolution**: Consuming the wet formwork to produce the watertight dry domain model via [`.resolve()`](crate::Formwork::resolve) (and [`.resolve_exact()`](crate::choice::SingleChoice::resolve_exact) for scalar choices and single slots).
//!
//! ## Example
//!
//! ```rust
//! use formwork::formwork;
//!
//! #[formwork]
//! pub struct Character {
//!     pub name: String,
//!     #[slot]
//!     pub class: String,
//! }
//! ```
//!
//! This macro expands `Character` into:
//! - `Character`: The dry domain model (aliased to `Character<CharacterDry>`).
//!   Its `class` field is simply a `String`.
//! - `CharacterFormwork`: The wet formwork mold (aliased to `Character<CharacterWet>`).
//!   Its `class` field is a `SingleSlot<String, String>` capable of holding a raw `String` or a [`SingleChoice<String>`](crate::choice::SingleChoice).
//! - Implementing [`Formwork`], enabling `.is_complete()`, `.resolve()`, and `.walk_choices()`.
//!
//! # Slot Decision Guide
//!
//! When declaring fields in a `#[formwork]` struct:
//!
//! - **Normal Field (no `#[slot]`)**: For invariants that are always known upfront (e.g., entity ID, creation timestamp).
//! - **Slot Field (`#[slot]`)**: For fields whose values may be decided later through choices.
//!
//! ## What slot to use
//!
//! Slot attributes are configured along two dimensions:
//! - **Cardinality / Multiplicity**: Single (scalar value), Option (optional value), Collection (arbitrary items or choices), or Pick (pool of candidates with cardinality constraints).
//! - **Hierarchy**: Leaf (resolves to concrete data directly) vs. Cascading (`cascade`, recursively resolving child formworks).
//!
//! | Slot Kind | Leaf (Non-Cascading) | Cascading (`cascade`) |
//! | :--- | :--- | :--- |
//! | **Single** | `#[slot]` | `#[slot(cascade)]` |
//! | **Option** | `#[slot(option)]` | `#[slot(option, cascade)]` |
//! | **Collection** | `#[slot(collection)]` | `#[slot(collection, cascade)]` |
//! | **Pick** | `#[slot(pick)]` | `#[slot(pick, cascade)]` |
//!
//! ## Slot Kinds Reference
//!
//! | Attribute | Field Type | Wet Formwork Slot | Resolves To | Description |
//! | :--- | :--- | :--- | :--- | :--- |
//! | `#[slot]` | `T` | `SingleSlot<T, T>` | `T` | Single item or [`SingleChoice<T>`](crate::choice::SingleChoice). |
//! | `#[slot(option)]` | `Option<T>` | `Option<SingleSlot<T, T>>` | `Option<T>` | Optional item or [`SingleChoice<T>`](crate::choice::SingleChoice). |
//! | `#[slot(collection)]` | `Vec<T>` | `Vec<SingleSlot<T, T>>` | `Vec<T>` | Ordered collection of individual items or [`SingleChoice<T>`](crate::choice::SingleChoice). |
//! | `#[slot(pick)]` | `Vec<T>` | `MultiSlot<T, T>` | `Vec<T>` | Multi-selection pick slot choosing items via [`MultiChoice<T>`](crate::choice::MultiChoice). |
//! | `#[slot(cascade)]` | `Child` | `SingleSlot<ChildFormwork, Child>` | `Child` | Cascading child formwork; resolving parent automatically resolves child. |
//! | `#[slot(option, cascade)]` | `Option<Child>` | `Option<SingleSlot<ChildFormwork, Child>>` | `Option<Child>` | Cascading optional child formwork. |
//! | `#[slot(collection, cascade)]` | `Vec<Child>` | `Vec<SingleSlot<ChildFormwork, Child>>` | `Vec<Child>` | Cascading collection of child formworks. |
//! | `#[slot(pick, cascade)]` | `Vec<Child>` | `MultiSlot<ChildFormwork, Child>` | `Vec<Child>` | Cascading pick slot selecting child formworks subject to cardinality. |
//!
//! # Comprehensive End-to-End Example
//!
//! ```rust
//! use formwork::{formwork, Formwork, choice::SingleChoice};
//!
//! #[formwork(builder)]
//! #[derive(Clone, Debug, PartialEq)]
//! pub struct HeroClass {
//!     pub name: String,
//!     #[slot]
//!     pub primary_weapon: String,
//!     #[slot(collection)]
//!     pub proficiencies: Vec<String>,
//! }
//!
//! // Step 1: Create a wet formwork using the generated builder
//! let mut hero_formwork: HeroClassFormwork = HeroClass::builder()
//!     .name("Fighter")
//!     .primary_weapon(SingleChoice::new("Choose martial weapon", vec![
//!         "Longsword".to_string(),
//!         "Battleaxe".to_string(),
//!     ]))
//!     .with_proficiencies("Shields".to_string())
//!     .build();
//!
//! // Step 2: Verify completeness (incomplete while choices remain)
//! assert!(!hero_formwork.is_complete());
//!
//! // Step 3: Satisfy choices using a visitor
//! hero_formwork.walk_choices(&mut formwork::visitor::AutoSelectFirst);
//! assert!(hero_formwork.is_complete());
//!
//! // Step 4: Resolve into watertight dry domain struct
//! let hero: HeroClass = hero_formwork.resolve().unwrap();
//! assert_eq!(hero.name, "Fighter");
//! assert_eq!(hero.primary_weapon, "Longsword");
//! assert_eq!(hero.proficiencies, vec!["Shields".to_string()]);
//! ```
//!
//! # Interactive Tutorial: The Chronicle of the Realm-Crafter
//!
//! To explore hands-on, progressive guides through every slot kind and traversal strategy,
//! visit the [`tutorial`] module. Follow along the 10 chapters to master constructing,
//! inspecting, stepping, and curing fluid formworks into solid domain models.

extern crate self as formwork;

pub use formwork_core::*;
pub use formwork_macro::*;
pub use typed_builder;

pub mod tutorial;
