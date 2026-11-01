//! # Chapter 7: The Bastion Wardens (Collection Cascade Slot: `#[slot(collection, cascade)]`)
//!
//! To fend off void-beasts emerging from rift squalls, citadels mount an ordered sequence of
//! **Defense Bastions** along the outer perimeter. Each bastion is a distinct child component
//! requiring its beam frequency to be harmonized.
//!
//! ## Concept: The Collection Cascade Slot
//!
//! When an entity contains an ordered list of child sub-formworks, use `#[slot(collection, cascade)]`:
//!
//! - **Wet Formwork (`DefenseCitadelFormwork`)**: The field has type
//!   `Vec<SingleSlot<DefenseBastionFormwork, DefenseBastion>>`.
//! - **Dry Domain Model (`DefenseCitadel`)**: The field has type `Vec<DefenseBastion>`.
//!
//! ### Piecemeal Child Composition
//!
//! You can construct the collection piecemeal using generated mutator methods:
//! - `.with_<field>(child_formwork)`: Appends an individual child formwork mold.
//! - `.<field>(iter)`: Extends the collection with an iterator of child formworks.
//!
//! When calling `.walk_choices()` on the parent, traversal descends into each child bastion
//! in sequence. Upon resolution, every child formwork in the vector is cured into its watertight
//! dry representation.
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::choice::SingleChoice;
//! use formwork::tutorial::ch07_collection_cascade::{DefenseBastion, DefenseCitadel, FrequencySelector};
//! use formwork::Formwork;
//!
//! // 1. Construct child bastions with pending frequency choices
//! let b1 = DefenseBastion::builder()
//!     .position("Fore-Port".to_string())
//!     .beam_frequency(SingleChoice::new("Set Frequency", vec!["Pulse".to_string(), "Beam".to_string()]))
//!     .build();
//!
//! let b2 = DefenseBastion::builder()
//!     .position("Aft-Starboard".to_string())
//!     .beam_frequency(SingleChoice::new("Set Frequency", vec!["Pulse".to_string(), "Beam".to_string()]))
//!     .build();
//!
//! // 2. Assemble the parent citadel using piecemeal mutator .with_bastions(...)
//! let mut citadel_wet = DefenseCitadel::builder()
//!     .citadel_name("Sky-Bulwark".to_string())
//!     .with_bastions(b1)
//!     .with_bastions(b2)
//!     .build();
//!
//! assert!(!citadel_wet.is_complete());
//!
//! // 3. Walk all bastions with our FrequencySelector visitor
//! let mut selector = FrequencySelector::new("Beam");
//! citadel_wet.walk_choices(&mut selector);
//!
//! assert!(citadel_wet.is_complete());
//! let citadel_dry = citadel_wet.resolve().expect("all child bastions cured");
//! assert_eq!(citadel_dry.bastions.len(), 2);
//! assert_eq!(citadel_dry.bastions[0].beam_frequency, "Beam");
//! assert_eq!(citadel_dry.bastions[1].beam_frequency, "Beam");
//! ```

use crate::choice::{MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, FormworkError, formwork};

/// A defensive turret bastion mounted on the citadel.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct DefenseBastion {
    /// Mounting location on the hull.
    pub position: String,
    /// Weapon beam emission frequency (single leaf slot).
    #[slot]
    pub beam_frequency: String,
}

/// A defensive sky-citadel holding an array of bastions.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct DefenseCitadel {
    /// Name of the citadel fortress.
    pub citadel_name: String,
    /// An ordered collection of defense bastions (collection cascade slot).
    #[slot(collection, cascade)]
    pub bastions: Vec<DefenseBastion>,
}

/// A visitor that chooses a target frequency for all bastions.
pub struct FrequencySelector {
    target: String,
}

impl FrequencySelector {
    /// Creates a new `FrequencySelector` seeking the target frequency.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }
}

impl ChoiceVisitor<String> for FrequencySelector {
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

impl ChoiceVisitor<DefenseBastionFormwork> for FrequencySelector {
    fn visit_single(&mut self, _choice: &mut SingleChoice<DefenseBastionFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<DefenseBastionFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

/// Runs the Chapter 7 tutorial, demonstrating collection cascade construction and resolution.
///
/// Returns the resolved watertight [`DefenseCitadel`] domain model.
pub fn run() -> DefenseCitadel {
    let b1 = DefenseBastion::builder()
        .position("Fore-Port".to_string())
        .beam_frequency(SingleChoice::new(
            "Set Frequency",
            vec!["Pulse".to_string(), "Beam".to_string()],
        ))
        .build();

    let b2 = DefenseBastion::builder()
        .position("Aft-Starboard".to_string())
        .beam_frequency(SingleChoice::new(
            "Set Frequency",
            vec!["Pulse".to_string(), "Beam".to_string()],
        ))
        .build();

    let mut citadel_wet = DefenseCitadel::builder()
        .citadel_name("Sky-Bulwark".to_string())
        .with_bastions(b1)
        .with_bastions(b2)
        .build();

    assert!(!citadel_wet.is_complete());
    assert!(matches!(
        citadel_wet.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    let mut selector = FrequencySelector::new("Beam");
    citadel_wet.walk_choices(&mut selector);

    assert!(citadel_wet.is_complete());
    let dry = citadel_wet.resolve().expect("resolves cleanly");
    assert_eq!(dry.citadel_name, "Sky-Bulwark");
    assert_eq!(dry.bastions.len(), 2);
    assert_eq!(dry.bastions[0].beam_frequency, "Beam");
    assert_eq!(dry.bastions[1].beam_frequency, "Beam");

    dry
}
