//! # Chapter 8: The Vanguard Retinue (Pick Cascade Slot: `#[slot(pick, cascade)]`)
//!
//! You cannot brave the Aetherial Rift alone. In the Skyguard Garrison, four veteran
//! guardians stand ready. The Council permits you to draft exactly 2 guardians to join
//! your bridge retinue and configure their combat specialties.
//!
//! ## Concept: The Pick Cascade Slot
//!
//! When you need to select a subset of candidate child sub-formworks from a pool, use
//! `#[slot(pick, cascade)]`:
//!
//! - **Wet Formwork (`SkyguardRosterFormwork`)**: The field has type
//!   `MultiSlot<SkyguardFormwork, Skyguard>`.
//! - **Dry Domain Model (`SkyguardRoster`)**: The field has type `Vec<Skyguard>`.
//!
//! ### Multi-Candidate Child Drafting
//!
//! A `#[slot(pick, cascade)]` combines multi-candidate selection with deep cascading resolution:
//! 1. **Candidate Pool**: The slot is populated with a [`MultiChoice`]
//!    holding candidate child formwork molds.
//! 2. **Dual-Tier Resolution**: Traversal first selects which candidate child formworks are drafted
//!    (subject to [`Cardinality`]). Then, it descends into the drafted
//!    children to satisfy their internal choices.
//! 3. **Filtered Curing**: Only the drafted child formworks are cured into the dry vector;
//!    unselected candidates are cleanly omitted!
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::choice::{Cardinality, MultiChoice, SingleChoice};
//! use formwork::tutorial::ch08_pick_cascade::{Skyguard, SkyguardRoster, RetinueDrafter};
//! use formwork::Formwork;
//!
//! // 1. Construct candidate guardians
//! let g1 = Skyguard::builder()
//!     .callsign("Astra-1".to_string())
//!     .specialty(SingleChoice::new("Set Specialty", vec!["Lancer".to_string(), "Shield".to_string()]))
//!     .build();
//! let g2 = Skyguard::builder()
//!     .callsign("Astra-2".to_string())
//!     .specialty(SingleChoice::new("Set Specialty", vec!["Sniper".to_string(), "Scout".to_string()]))
//!     .build();
//! let g3 = Skyguard::builder()
//!     .callsign("Astra-3".to_string())
//!     .specialty(SingleChoice::new("Set Specialty", vec!["Medic".to_string(), "Alchemist".to_string()]))
//!     .build();
//!
//! // 2. Offer candidates under Cardinality::Exact(2)
//! let candidate_pool = MultiChoice::new("Draft 2 Skyguards", vec![g1, g2, g3])
//!     .with_cardinality(Cardinality::Exact(2));
//!
//! let mut roster_wet = SkyguardRoster::builder()
//!     .squad_id("Alpha-Vanguard".to_string())
//!     .drafted_guards(candidate_pool)
//!     .build();
//!
//! assert!(!roster_wet.is_complete());
//!
//! // 3. Draft guards at indices 0 and 1, and satisfy their specialty choices
//! let mut drafter = RetinueDrafter::new(vec![0, 1], "Lancer");
//! roster_wet.walk_choices(&mut drafter);
//!
//! assert!(roster_wet.is_complete());
//! let roster_dry = roster_wet.resolve().expect("all drafted guards cured");
//! assert_eq!(roster_dry.drafted_guards.len(), 2);
//! assert_eq!(roster_dry.drafted_guards[0].callsign, "Astra-1");
//! assert_eq!(roster_dry.drafted_guards[1].callsign, "Astra-2");
//! ```

use crate::choice::{Cardinality, MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, FormworkError, formwork};

/// A specialist guardian for the skyguard retinue.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct Skyguard {
    /// Callsign of the guardian.
    pub callsign: String,
    /// Tactical specialty (single leaf slot).
    #[slot]
    pub specialty: String,
}

/// A drafted squad roster of specialist skyguards.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SkyguardRoster {
    /// Identification of the squad.
    pub squad_id: String,
    /// Drafted skyguard retainers (pick cascade slot).
    #[slot(pick, cascade)]
    pub drafted_guards: Vec<Skyguard>,
}

/// A visitor that drafts candidate guardians by index and auto-selects their internal specialties.
pub struct RetinueDrafter {
    draft_indices: Vec<usize>,
    preferred_specialty: String,
}

impl RetinueDrafter {
    /// Creates a new `RetinueDrafter`.
    pub fn new(draft_indices: Vec<usize>, preferred_specialty: impl Into<String>) -> Self {
        Self {
            draft_indices,
            preferred_specialty: preferred_specialty.into(),
        }
    }
}

// RetinueDrafter must visit SkyguardFormwork candidates in the pick slot, and String choices inside children
impl ChoiceVisitor<SkyguardFormwork> for RetinueDrafter {
    fn visit_single(&mut self, _choice: &mut SingleChoice<SkyguardFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<SkyguardFormwork>) -> VisitorFlow {
        for &idx in &self.draft_indices {
            let _ = choice.select(idx);
        }
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

impl ChoiceVisitor<String> for RetinueDrafter {
    fn visit_single(&mut self, choice: &mut SingleChoice<String>) -> VisitorFlow {
        if let Some(pos) = choice
            .options()
            .iter()
            .position(|opt| opt == &self.preferred_specialty)
        {
            let _ = choice.select(pos);
        } else if !choice.options().is_empty() {
            let _ = choice.select(0);
        }
        let _ = choice.confirm();
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<String>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

/// Runs the Chapter 8 tutorial, demonstrating pick cascade drafting and resolution.
///
/// Returns the resolved watertight [`SkyguardRoster`] domain model.
pub fn run() -> SkyguardRoster {
    let g1 = Skyguard::builder()
        .callsign("Astra-1".to_string())
        .specialty(SingleChoice::new(
            "Set Specialty",
            vec!["Lancer".to_string(), "Shield".to_string()],
        ))
        .build();
    let g2 = Skyguard::builder()
        .callsign("Astra-2".to_string())
        .specialty(SingleChoice::new(
            "Set Specialty",
            vec!["Sniper".to_string(), "Scout".to_string()],
        ))
        .build();
    let g3 = Skyguard::builder()
        .callsign("Astra-3".to_string())
        .specialty(SingleChoice::new(
            "Set Specialty",
            vec!["Medic".to_string(), "Alchemist".to_string()],
        ))
        .build();

    let candidate_pool = MultiChoice::new("Draft 2 Skyguards", vec![g1, g2, g3])
        .with_cardinality(Cardinality::Exact(2));

    let mut roster_wet = SkyguardRoster::builder()
        .squad_id("Alpha-Vanguard".to_string())
        .drafted_guards(candidate_pool)
        .build();

    assert!(!roster_wet.is_complete());
    assert!(matches!(
        roster_wet.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    let mut drafter = RetinueDrafter::new(vec![0, 1], "Lancer");
    roster_wet.walk_choices(&mut drafter);

    assert!(roster_wet.is_complete());
    let dry = roster_wet.resolve().expect("resolves cleanly");
    assert_eq!(dry.squad_id, "Alpha-Vanguard");
    assert_eq!(dry.drafted_guards.len(), 2);
    assert_eq!(dry.drafted_guards[0].callsign, "Astra-1");
    assert_eq!(dry.drafted_guards[1].callsign, "Astra-2");

    dry
}
