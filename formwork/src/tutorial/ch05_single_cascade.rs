//! # Chapter 5: The Astral Core Sanctum (Single Cascade Slot: `#[slot(cascade)]`)
//!
//! To navigate the vast rift between floating sky-islands, you begin constructing the
//! **Floating Sanctum**. At its heart lies the **Core Sanctum**—a delicate internal engine
//! that requires its own resonant catalyst crystal.
//!
//! ## Concept: The Single Cascade Slot
//!
//! Real-world architectures are rarely flat. Often, a domain struct contains another complex
//! domain struct whose internal choices must be configured during creation.
//!
//! In `formwork`, adding `cascade` to a slot attribute (e.g. `#[slot(cascade)]`) creates a
//! **Single Cascade Slot**:
//!
//! - **Wet Formwork (`FloatingSanctumFormwork`)**: The field has type
//!   `SingleSlot<CoreSanctumFormwork, CoreSanctum>`. It holds an unconfirmed child formwork mold!
//! - **Dry Domain Model (`FloatingSanctum`)**: The field has type `CoreSanctum`.
//!
//! ### Recursive Walking and Automatic Curing
//!
//! Cascading slots unlock the true power of Higher-Kinded Data:
//! 1. **Unified Traversal**: Calling `.walk_choices()` on the parent formwork recursively
//!    steps into all child formworks, visiting their internal choices.
//! 2. **Cascade Completeness**: The parent is only complete when all of its own choices AND
//!    all choices in its cascading children are satisfied.
//! 3. **Automatic Curing**: Calling `.resolve()` on the parent automatically consumes and cures
//!    the child formwork, returning a watertight dry parent containing the watertight dry child!
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::choice::SingleChoice;
//! use formwork::tutorial::ch05_single_cascade::{CoreSanctum, FloatingSanctum, CatalystSelector};
//! use formwork::Formwork;
//!
//! // 1. Construct the child wet formwork with an open catalyst choice
//! let child_wet = CoreSanctum::builder()
//!     .sanctum_id("Sanctum-Prime".to_string())
//!     .catalyst_crystal(SingleChoice::new(
//!         "Choose Resonant Catalyst Crystal",
//!         vec!["Sun-Shard".to_string(), "Aether-Gem".to_string()],
//!     ))
//!     .build();
//!
//! // 2. Pass the wet child formwork directly into the parent builder
//! let mut parent_wet = FloatingSanctum::builder()
//!     .name("Aethel-Vessel".to_string())
//!     .core(child_wet)
//!     .build();
//!
//! // Parent is incomplete because child has an unconfirmed choice
//! assert!(!parent_wet.is_complete());
//!
//! // 3. Walk parent choices: traversal visits the child's catalyst choice recursively!
//! let mut selector = CatalystSelector::new("Sun-Shard");
//! parent_wet.walk_choices(&mut selector);
//!
//! assert!(parent_wet.is_complete());
//!
//! // 4. Resolving parent automatically cures child into dry CoreSanctum
//! let parent_dry = parent_wet.resolve().expect("cascading resolution succeeds");
//! assert_eq!(parent_dry.name, "Aethel-Vessel");
//! assert_eq!(parent_dry.core.sanctum_id, "Sanctum-Prime");
//! assert_eq!(parent_dry.core.catalyst_crystal, "Sun-Shard");
//! ```

use crate::choice::{MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, FormworkError, formwork};

/// The internal power core of a sanctum.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct CoreSanctum {
    /// Identifier for this power core.
    pub sanctum_id: String,
    /// The catalyst crystal driving the core (single leaf slot).
    #[slot]
    pub catalyst_crystal: String,
}

/// A floating sky-sanctum containing an embedded core engine.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct FloatingSanctum {
    /// Name of the floating sanctum.
    pub name: String,
    /// The embedded core sanctum (single cascade slot).
    #[slot(cascade)]
    pub core: CoreSanctum,
}

/// A visitor that selects a specific catalyst crystal.
pub struct CatalystSelector {
    target: String,
}

impl CatalystSelector {
    /// Creates a new `CatalystSelector` seeking the target catalyst.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }
}

impl ChoiceVisitor<String> for CatalystSelector {
    fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
        if let Some(pos) = choice
            .options()
            .iter()
            .position(|opt| opt == &self.target)
        {
            let _ = choice.select(pos);
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<String>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<CoreSanctumFormwork> for CatalystSelector {
    fn visit_single(&mut self, _choice: &mut SingleChoice<CoreSanctumFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<CoreSanctumFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

/// Runs the Chapter 5 tutorial, demonstrating cascading child formwork walking and resolution.
///
/// Returns the resolved watertight [`FloatingSanctum`] domain model.
pub fn run() -> FloatingSanctum {
    let child_wet = CoreSanctum::builder()
        .sanctum_id("Sanctum-Prime".to_string())
        .catalyst_crystal(SingleChoice::new(
            "Choose Resonant Catalyst Crystal",
            vec!["Sun-Shard".to_string(), "Aether-Gem".to_string()],
        ))
        .build();

    let mut parent_wet = FloatingSanctum::builder()
        .name("Aethel-Vessel".to_string())
        .core(child_wet)
        .build();

    assert!(!parent_wet.is_complete());
    assert!(matches!(
        parent_wet.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    let mut selector = CatalystSelector::new("Sun-Shard");
    parent_wet.walk_choices(&mut selector);

    assert!(parent_wet.is_complete());
    let dry = parent_wet.resolve().expect("resolves cleanly");
    assert_eq!(dry.name, "Aethel-Vessel");
    assert_eq!(dry.core.sanctum_id, "Sanctum-Prime");
    assert_eq!(dry.core.catalyst_crystal, "Sun-Shard");

    dry
}
