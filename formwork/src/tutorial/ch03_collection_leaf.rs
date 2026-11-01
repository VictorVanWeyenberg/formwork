//! # Chapter 3: The Relic Armory (Collection Leaf Slot: `#[slot(collection)]`)
//!
//! Venturing into the void squalls of Aethelgard requires an artisan toolbelt stocked with
//! both standardized implements and specialized, attuned instruments.
//!
//! ## Concept: The Collection Leaf Slot
//!
//! When an entity holds an ordered list of items where each element can be either a fixed
//! value or an unconfirmed choice, use `#[slot(collection)]`:
//!
//! - **Wet Formwork (`ArtisanToolbeltFormwork`)**: The field has type
//!   `Vec<SingleSlot<String, String>>`. Each element in the vector is an independent slot.
//! - **Dry Domain Model (`ArtisanToolbelt`)**: The field has type `Vec<String>`.
//!
//! ### Piecemeal Builder Mutators
//!
//! A hallmark of `formwork`'s ergonomics is its piecemeal collection builder methods:
//! - `.with_<field>(item)`: Appends an individual element—either a raw value `T` or a
//!   [`SingleChoice<T>`](crate::choice::SingleChoice).
//! - `.<field>(iter)`: Populates the collection from an iterable collection of items or choices.
//!
//! When traversing with a visitor via `.walk_choices()`, each choice-bearing element in the
//! collection is visited in sequential index order.
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::choice::SingleChoice;
//! use formwork::tutorial::ch03_collection_leaf::{ArtisanToolbelt, ToolSelector};
//! use formwork::Formwork;
//!
//! // Build the toolbelt mixing fixed gear with a pending choice
//! let mut wet_toolbelt = ArtisanToolbelt::builder()
//!     .owner("Sylvan-Smith".to_string())
//!     .with_tools("Aether-Chisel".to_string()) // Fixed item (already cured)
//!     .with_tools(SingleChoice::new(           // Open choice item
//!         "Choose Secondary Instrument",
//!         vec!["Astral-Compass".to_string(), "Prism-Lens".to_string()],
//!     ))
//!     .build();
//!
//! assert!(!wet_toolbelt.is_complete());
//!
//! // Satisfy the choice via ToolSelector visitor
//! let mut selector = ToolSelector::new("Astral-Compass");
//! wet_toolbelt.walk_choices(&mut selector);
//!
//! assert!(wet_toolbelt.is_complete());
//! let dry_toolbelt = wet_toolbelt.resolve().expect("resolved successfully");
//! assert_eq!(dry_toolbelt.tools, vec!["Aether-Chisel".to_string(), "Astral-Compass".to_string()]);
//! ```

use crate::choice::{MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, FormworkError, formwork};

/// An artisan's toolbelt holding equipment and instruments.
///
/// In wet mode (`ArtisanToolbeltFormwork`), `tools` is a `Vec<SingleSlot<String, String>>`.
/// In dry mode (`ArtisanToolbelt`), `tools` is a `Vec<String>`.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct ArtisanToolbelt {
    /// The owner of the toolbelt (fixed invariant).
    pub owner: String,
    /// An ordered collection of tools and relics (collection leaf slot).
    #[slot(collection)]
    pub tools: Vec<String>,
}

/// A visitor that matches choices on tool options and selects the desired instrument.
pub struct ToolSelector {
    target: String,
}

impl ToolSelector {
    /// Creates a new `ToolSelector` seeking the target tool.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }
}

impl ChoiceVisitor<String> for ToolSelector {
    fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
        if let Some(pos) = choice
            .options()
            .iter()
            .position(|tool| tool == &self.target)
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

/// Runs the Chapter 3 tutorial, demonstrating collection slot construction and resolution.
///
/// Returns the resolved watertight [`ArtisanToolbelt`] domain model.
pub fn run() -> ArtisanToolbelt {
    let mut wet_toolbelt = ArtisanToolbelt::builder()
        .owner("Sylvan-Smith".to_string())
        .with_tools("Aether-Chisel".to_string())
        .with_tools(SingleChoice::new(
            "Choose Secondary Instrument",
            vec!["Astral-Compass".to_string(), "Prism-Lens".to_string()],
        ))
        .build();

    assert!(!wet_toolbelt.is_complete());
    assert!(matches!(
        wet_toolbelt.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    let mut selector = ToolSelector::new("Astral-Compass");
    wet_toolbelt.walk_choices(&mut selector);

    assert!(wet_toolbelt.is_complete());
    let dry = wet_toolbelt.resolve().expect("resolved successfully");
    assert_eq!(
        dry.tools,
        vec!["Aether-Chisel".to_string(), "Astral-Compass".to_string()]
    );

    dry
}
