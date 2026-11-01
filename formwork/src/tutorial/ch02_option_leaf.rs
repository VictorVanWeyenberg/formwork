//! # Chapter 2: The Optional Boon (Option Leaf Slot: `#[slot(option)]`)
//!
//! As you step into the Celestial Sanctum, celestial alignments offer an optional patron
//! blessing. Some artisans choose to bind themselves to a stellar patron, while others walk
//! unaligned, drawing purely unshaped aetherial power.
//!
//! ## Concept: The Option Leaf Slot
//!
//! Real-world domain entities often have optional traits or attributes (such as an optional
//! secondary patron, nickname, or discount code). In `formwork`, annotating a field with
//! `#[slot(option)]` creates an **Option Leaf Slot**:
//!
//! - **Wet Formwork (`CrafterAffiliationFormwork`)**: The field has type
//!   `Option<SingleSlot<String, String>>`. It can be `None`, a concrete `Some("value")`, or
//!   `Some(SingleChoice<String>)`.
//! - **Dry Domain Model (`CrafterAffiliation`)**: The field has type `Option<String>`.
//!
//! ### Immediate Completeness on Omission
//!
//! A crucial feature of optional slots is **omission ergonomics**:
//! - If the builder omits the optional slot (or sets it to `None`), the slot contains no
//!   pending choices. The formwork is **immediately complete** without requiring any visitor walking!
//! - If the builder populates the slot with a [`SingleChoice`],
//!   the formwork remains incomplete until the choice is visited and confirmed.
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::choice::SingleChoice;
//! use formwork::tutorial::ch02_option_leaf::{CrafterAffiliation, BlessingSelector};
//! use formwork::Formwork;
//!
//! // 1. Omission: Unaligned crafter (slot omitted, defaults to None)
//! let unaligned_wet = CrafterAffiliation::builder()
//!     .crafter_name("Lyra".to_string())
//!     .build();
//!
//! // The formwork is immediately complete because no choices are pending!
//! assert!(unaligned_wet.is_complete());
//! let unaligned_dry = unaligned_wet.resolve().expect("immediate resolution");
//! assert_eq!(unaligned_dry.celestial_blessing, None);
//!
//! // 2. Choice-bearing: Blessed crafter with an open choice
//! let mut blessed_wet = CrafterAffiliation::builder()
//!     .crafter_name("Kaelen".to_string())
//!     .celestial_blessing(Some(SingleChoice::new(
//!         "Choose Celestial Blessing",
//!         vec!["Dawn-Crest".to_string(), "Twilight-Veil".to_string()],
//!     )))
//!     .build();
//!
//! // Incomplete until choice is resolved
//! assert!(!blessed_wet.is_complete());
//!
//! let mut selector = BlessingSelector::new("Dawn-Crest");
//! blessed_wet.walk_choices(&mut selector);
//!
//! assert!(blessed_wet.is_complete());
//! let blessed_dry = blessed_wet.resolve().expect("resolved after visitor");
//! assert_eq!(blessed_dry.celestial_blessing, Some("Dawn-Crest".to_string()));
//! ```

use crate::choice::{MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, FormworkError, formwork};

/// A Realm-Crafter's celestial patron affiliation.
///
/// In wet mode (`CrafterAffiliationFormwork`), `celestial_blessing` is an
/// `Option<SingleSlot<String, String>>`.
/// In dry mode (`CrafterAffiliation`), it is an `Option<String>`.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct CrafterAffiliation {
    /// The artisan's name (fixed invariant).
    pub crafter_name: String,
    /// An optional celestial patron blessing (option leaf slot).
    #[slot(option)]
    pub celestial_blessing: Option<String>,
}

/// A visitor that selects a specific blessing from available options.
pub struct BlessingSelector {
    target: String,
}

impl BlessingSelector {
    /// Creates a new `BlessingSelector` targeting the named blessing.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }
}

impl ChoiceVisitor<String> for BlessingSelector {
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

/// Runs the Chapter 2 tutorial, demonstrating both omitted and choice-bearing optional slots.
///
/// Returns a tuple containing `(unaligned_crafter, blessed_crafter)`.
pub fn run() -> (CrafterAffiliation, CrafterAffiliation) {
    // 1. Omitted optional slot
    let unaligned_wet = CrafterAffiliation::builder()
        .crafter_name("Lyra".to_string())
        .build();

    assert!(unaligned_wet.is_complete());
    let unaligned = unaligned_wet.resolve().expect("resolves immediately");
    assert_eq!(unaligned.celestial_blessing, None);

    // 2. Populated optional slot with open choice
    let mut blessed_wet = CrafterAffiliation::builder()
        .crafter_name("Kaelen".to_string())
        .celestial_blessing(Some(SingleChoice::new(
            "Choose Celestial Blessing",
            vec!["Dawn-Crest".to_string(), "Twilight-Veil".to_string()],
        )))
        .build();

    assert!(!blessed_wet.is_complete());
    assert!(matches!(
        blessed_wet.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    let mut selector = BlessingSelector::new("Dawn-Crest");
    blessed_wet.walk_choices(&mut selector);

    assert!(blessed_wet.is_complete());
    let blessed = blessed_wet.resolve().expect("resolves after choice is made");
    assert_eq!(blessed.celestial_blessing, Some("Dawn-Crest".to_string()));

    (unaligned, blessed)
}
