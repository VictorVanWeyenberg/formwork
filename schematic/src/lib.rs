//! # Schematic: Higher-Kinded Data (HKD) Blueprints for Rust
//!
//! `schematic` separates incomplete, choice-bearing blueprints from watertight, resolved domain models.
//!
//! # Architecture: Blueprint vs. Watertight Domain Model
//!
//! Complex domain systems (such as tabletop RPG character builders, dynamic product configurators,
//! or tax/financial planning engines) require two very different views of the same data:
//!
//! 1. **The Blueprint (Unresolved Representation)**:
//!    During creation or editing, data is incomplete. Fields contain open questions, pending choices,
//!    or cascading nested selections. We need to inspect pending choices, auto-fill defaults, or revert
//!    selections without destroying the history of choices.
//! 2. **The Domain Model (Resolved Representation)**:
//!    Once completed and validated, application logic (combat engines, invoicing systems, analytics)
//!    needs a **watertight** struct. No `Option`s, no choice enums, no runtime unwraps.
//!
//! Instead of duplicating types (e.g. `CharacterDraft` vs `Character`) and maintaining fragile mapping code,
//! `schematic` leverages **Higher-Kinded Data (HKD)**:
//!
//! ```rust
//! use schematic::schematic;
//!
//! #[schematic]
//! pub struct Character {
//!     pub name: String,
//!     #[slot]
//!     pub class: String,
//! }
//! ```
//!
//! This macro expands `Character` into:
//! - `Character`: The resolved domain model (aliased to `Character<CharacterResolved>`).
//!   Its `class` field is simply a `String`.
//! - `CharacterSchematic`: The unresolved blueprint (aliased to `Character<CharacterUnresolved>`).
//!   Its `class` field is a `SingleSlot<String, String>` capable of holding a raw `String` or a [`SingleChoice<String>`](crate::choice::SingleChoice).
//! - Implementing [`Schematic`], enabling `.is_complete()`, `.resolve()`, and `.walk_choices()`.
//!
//! # Slot Decision Guide
//!
//! When declaring fields in a `#[schematic]` struct:
//!
//! - **Normal Field (no `#[slot]`)**: For invariants that are always known upfront (e.g., entity ID, creation timestamp).
//! - **Slot Field (`#[slot]`)**: For fields whose values may be decided later through choices.
//!
//! ## 2x3 Orthogonal Slot Matrix
//!
//! Slot attributes are configured along two orthogonal dimensions:
//! - **Cardinality / Multiplicity**: Single (scalar value), Collection (arbitrary items or choices), or Pick (pool of candidates with cardinality constraints).
//! - **Hierarchy**: Leaf (resolves to concrete data directly) vs. Cascading (`cascade`, recursively resolving child schematics).
//!
//! | Slot Kind | Leaf (Non-Cascading) | Cascading (`cascade`) |
//! | :--- | :--- | :--- |
//! | **Single** | `#[slot]` | `#[slot(cascade)]` |
//! | **Collection** | `#[slot(collection)]` | `#[slot(collection, cascade)]` |
//! | **Pick** | `#[slot(pick)]` | `#[slot(pick, cascade)]` |
//!
//! ## Slot Kinds Reference
//!
//! | Attribute | Field Type | Unresolved Blueprint Slot | Resolves To | Description |
//! | :--- | :--- | :--- | :--- | :--- |
//! | `#[slot]` | `T` | `SingleSlot<T, T>` | `T` | Single item or [`SingleChoice<T>`](crate::choice::SingleChoice). |
//! | `#[slot(collection)]` | `Vec<T>` | `Vec<MultiSlot<T, T>>` | `Vec<T>` | Collection of items, or [`MultiChoice<T>`](crate::choice::MultiChoice). |
//! | `#[slot(pick)]` | `Vec<T>` | `Vec<MultiSlot<T, T>>` | `Vec<T>` | Specialized collection for pick-N selections. |
//! | `#[slot(cascade)]` | `Child` | `SingleSlot<ChildSchematic, Child>` | `Child` | Cascading child schematic; resolving parent automatically resolves child. |
//! | `#[slot(collection, cascade)]` | `Vec<Child>` | `Vec<MultiSlot<ChildSchematic, Child>>` | `Vec<Child>` | Cascading collection of child schematics. |
//! | `#[slot(pick, cascade)]` | `Vec<Child>` | `Vec<MultiSlot<ChildSchematic, Child>>` | `Vec<Child>` | Cascading pick slot selecting child schematics subject to cardinality. |
//!
//! # Comprehensive End-to-End Example
//!
//! ```rust
//! use schematic::{schematic, Schematic, choice::SingleChoice};
//!
//! #[schematic(builder)]
//! #[derive(Clone, Debug, PartialEq)]
//! pub struct HeroClass {
//!     pub name: String,
//!     #[slot]
//!     pub primary_weapon: String,
//!     #[slot(collection)]
//!     pub proficiencies: Vec<String>,
//! }
//!
//! // Step 1: Create an unresolved blueprint using the generated builder
//! let mut hero_schematic: HeroClassSchematic = HeroClass::builder()
//!     .name("Fighter")
//!     .primary_weapon(SingleChoice::new("Choose martial weapon", vec![
//!         "Longsword".to_string(),
//!         "Battleaxe".to_string(),
//!     ]))
//!     .with_proficiencies("Shields".to_string())
//!     .build();
//!
//! // Step 2: Verify completeness (incomplete while choices remain)
//! assert!(!hero_schematic.is_complete());
//!
//! // Step 3: Satisfy choices using a visitor
//! hero_schematic.walk_choices(&mut schematic::visitor::AutoSelectFirst);
//! assert!(hero_schematic.is_complete());
//!
//! // Step 4: Resolve into watertight domain struct
//! let hero: HeroClass = hero_schematic.resolve().unwrap();
//! assert_eq!(hero.name, "Fighter");
//! assert_eq!(hero.primary_weapon, "Longsword");
//! assert_eq!(hero.proficiencies, vec!["Shields".to_string()]);
//! ```

pub use schematic_core::*;
pub use schematic_macro::*;
