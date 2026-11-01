//! # Chapter 1: The Artisan's Awakening (Single Leaf Slot: `#[slot]`)
//!
//! You awaken beneath the crystalline vault of the Sun-Spire Forge. Before embarking on
//! the Shattered Skylands of Aethelgard, every Realm-Crafter must carve their primary
//! **Keystone of Dawn**.
//!
//! ## Concept: The Single Leaf Slot
//!
//! When designing domain entities, some attributes are fixed invariants known at creation time
//! (such as an artisan's name or lineage), while others represent pending decisions that must be
//! chosen from candidate options (such as a primary crafting discipline).
//!
//! In `formwork`, annotating a field with `#[slot]` designates it as a **Single Leaf Slot**:
//!
//! - **Wet Formwork (`CrafterIdentityFormwork`)**: The field has type `SingleSlot<String, String>`,
//!   allowing it to hold either a concrete string or an unconfirmed [`SingleChoice<String>`](crate::choice::SingleChoice).
//! - **Dry Domain Model (`CrafterIdentity`)**: The field has type `String`. Once resolved, all
//!   decision scaffolding is discarded, producing a watertight model with zero unwraps.
//!
//! ## Step-by-Step Walkthrough
//!
//! 1. **Define the Blueprint**: Annotate your struct with `#[formwork(builder)]` and mark the
//!    choice-bearing field with `#[slot]`.
//! 2. **Construct with Builder**: Use `<Struct>::builder()` to populate fixed fields directly and
//!    supply a [`SingleChoice`] for the slot.
//! 3. **Inspect Completeness**: Check `.is_complete()`. Attempting to `.resolve()` while a choice
//!    remains unconfirmed fails safely with [`FormworkError::WetChoice`].
//! 4. **Traverse Choices with a Visitor**: Implement [`ChoiceVisitor`]
//!    or use built-in visitors to inspect and select options.
//! 5. **Cure into Watertight Model**: Call `.resolve()` to consume the wet formwork and receive the
//!    watertight dry struct.
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::choice::SingleChoice;
//! use formwork::tutorial::ch01_single_leaf::{CrafterIdentity, DisciplineSelector};
//! use formwork::Formwork;
//!
//! // 1. Construct the wet formwork with an unconfirmed discipline choice
//! let mut wet_crafter = CrafterIdentity::builder()
//!     .name("Valen".to_string())
//!     .discipline(SingleChoice::new(
//!         "Choose your Guild Discipline",
//!         vec![
//!             "Aether-Weaver".to_string(),
//!             "Runic-Smith".to_string(),
//!             "Sylvan-Shaper".to_string(),
//!         ],
//!     ))
//!     .build();
//!
//! // The formwork is wet and incomplete
//! assert!(!wet_crafter.is_complete());
//! assert!(wet_crafter.clone().resolve().is_err());
//!
//! // 2. Walk choices using our custom DisciplineSelector visitor
//! let mut selector = DisciplineSelector::new("Aether-Weaver");
//! wet_crafter.walk_choices(&mut selector);
//!
//! // The formwork is now complete
//! assert!(wet_crafter.is_complete());
//!
//! // 3. Cure into the watertight dry domain model
//! let dry_crafter = wet_crafter.resolve().expect("formwork is complete");
//! assert_eq!(dry_crafter.name, "Valen");
//! assert_eq!(dry_crafter.discipline, "Aether-Weaver");
//! ```

use crate::choice::SingleChoice;
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, FormworkError, formwork};

/// A Realm-Crafter's core identity.
///
/// In wet mode (`CrafterIdentityFormwork`), `discipline` holds a [`SingleSlot`](crate::SingleSlot).
/// In dry mode (`CrafterIdentity`), `discipline` is a plain [`String`].
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct CrafterIdentity {
    /// The artisan's name (fixed invariant).
    pub name: String,
    /// The primary crafting discipline (single leaf slot).
    #[slot]
    pub discipline: String,
}

/// A custom visitor that inspects available disciplines and selects a targeted discipline by name.
pub struct DisciplineSelector {
    target: String,
}

impl DisciplineSelector {
    /// Creates a new `DisciplineSelector` seeking the specified target discipline.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }
}

impl ChoiceVisitor<String> for DisciplineSelector {
    fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
        if let Some(pos) = choice
            .options()
            .iter()
            .position(|candidate| candidate == &self.target)
        {
            let _ = choice.select(pos);
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut crate::choice::MultiChoice<String>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

/// Runs the Chapter 1 tutorial, constructing, visiting, and curing a [`CrafterIdentity`].
///
/// Returns the resolved watertight [`CrafterIdentity`] domain model.
pub fn run() -> CrafterIdentity {
    // 1. Build wet formwork
    let mut wet_crafter = CrafterIdentity::builder()
        .name("Valen".to_string())
        .discipline(SingleChoice::new(
            "Choose your Guild Discipline",
            vec![
                "Aether-Weaver".to_string(),
                "Runic-Smith".to_string(),
                "Sylvan-Shaper".to_string(),
            ],
        ))
        .build();

    // Verify wet incomplete state
    assert!(!wet_crafter.is_complete());
    assert!(matches!(
        wet_crafter.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    // 2. Walk choices with custom visitor
    let mut visitor = DisciplineSelector::new("Aether-Weaver");
    wet_crafter.walk_choices(&mut visitor);

    // Verify completeness
    assert!(wet_crafter.is_complete());

    // 3. Resolve into watertight dry domain model
    let dry_crafter = wet_crafter.resolve().expect("resolution succeeds");
    assert_eq!(dry_crafter.name, "Valen");
    assert_eq!(dry_crafter.discipline, "Aether-Weaver");

    dry_crafter
}
