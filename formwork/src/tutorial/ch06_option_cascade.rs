//! # Chapter 6: The Ephemeral Pavilion (Option Cascade Slot: `#[slot(option, cascade)]`)
//!
//! To navigate high-altitude squalls, an artisan may equip the vessel with an optional
//! deployable **Outrigger Wing**. If installed, the wing itself possesses customizable
//! stabilizer flaps that must be attuned before flight.
//!
//! ## Concept: The Option Cascade Slot
//!
//! When an entity optionally contains a child sub-formwork, use `#[slot(option, cascade)]`:
//!
//! - **Wet Formwork (`SkyVesselFormwork`)**: The field has type
//!   `Option<SingleSlot<OutriggerWingFormwork, OutriggerWing>>`.
//! - **Dry Domain Model (`SkyVessel`)**: The field has type `Option<OutriggerWing>`.
//!
//! ### Clean Traversal Skipping
//!
//! When traversing choices via `.walk_choices()`:
//! - If the option is `None`, traversal skips the branch entirely—zero overhead, zero panics.
//! - If the option is `Some(child)`, traversal seamlessly descends into the child formwork's choices.
//!
//! Upon resolution, if present, the child formwork is automatically cured into its watertight
//! dry domain representation.
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::choice::SingleChoice;
//! use formwork::tutorial::ch06_option_cascade::{OutriggerWing, SkyVessel, TrimSelector};
//! use formwork::Formwork;
//!
//! // 1. Omission: Streamlined light vessel without outrigger
//! let light_wet = SkyVessel::builder()
//!     .vessel_name("Zephyr-Dart".to_string())
//!     .build();
//!
//! assert!(light_wet.is_complete());
//! let light_dry = light_wet.resolve().expect("omitted cascade resolves immediately");
//! assert_eq!(light_dry.auxiliary_wing, None);
//!
//! // 2. Present: Heavy cruiser with an outrigger wing containing pending trim choice
//! let wing_wet = OutriggerWing::builder()
//!     .wing_id("Wing-Port".to_string())
//!     .stabilizer_trim(SingleChoice::new(
//!         "Set Stabilizer Trim",
//!         vec!["High-Lift".to_string(), "Speed-Glide".to_string()],
//!     ))
//!     .build();
//!
//! let mut heavy_wet = SkyVessel::builder()
//!     .vessel_name("Aura-Cruiser".to_string())
//!     .auxiliary_wing(wing_wet)
//!     .build();
//!
//! assert!(!heavy_wet.is_complete());
//!
//! let mut selector = TrimSelector::new("High-Lift");
//! heavy_wet.walk_choices(&mut selector);
//!
//! assert!(heavy_wet.is_complete());
//! let heavy_dry = heavy_wet.resolve().expect("resolved after child satisfied");
//! assert_eq!(heavy_dry.auxiliary_wing.unwrap().stabilizer_trim, "High-Lift");
//! ```

use crate::choice::{MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, FormworkError, formwork};

/// An optional aerodynamic outrigger wing.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct OutriggerWing {
    /// Identifier for this outrigger wing.
    pub wing_id: String,
    /// Aerodynamic stabilizer trim setting (single leaf slot).
    #[slot]
    pub stabilizer_trim: String,
}

/// A sky-vessel with an optional cascading outrigger wing.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct SkyVessel {
    /// Name of the sky-vessel.
    pub vessel_name: String,
    /// An optional cascading outrigger wing (option cascade slot).
    #[slot(option, cascade)]
    pub auxiliary_wing: Option<OutriggerWing>,
}

/// A visitor that chooses a specific stabilizer trim option.
pub struct TrimSelector {
    target: String,
}

impl TrimSelector {
    /// Creates a new `TrimSelector` seeking the target trim.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }
}

impl ChoiceVisitor<String> for TrimSelector {
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

impl ChoiceVisitor<OutriggerWingFormwork> for TrimSelector {
    fn visit_single(&mut self, _choice: &mut SingleChoice<OutriggerWingFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, _choice: &mut MultiChoice<OutriggerWingFormwork>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

/// Runs the Chapter 6 tutorial, demonstrating optional cascade behavior (omitted vs present).
///
/// Returns a tuple containing `(light_vessel, heavy_vessel)`.
pub fn run() -> (SkyVessel, SkyVessel) {
    // 1. Omitted optional cascade
    let light_wet = SkyVessel::builder()
        .vessel_name("Zephyr-Dart".to_string())
        .build();

    assert!(light_wet.is_complete());
    let light = light_wet.resolve().expect("resolves immediately");
    assert_eq!(light.auxiliary_wing, None);

    // 2. Present optional cascade with open choice
    let wing_wet = OutriggerWing::builder()
        .wing_id("Wing-Port".to_string())
        .stabilizer_trim(SingleChoice::new(
            "Set Stabilizer Trim",
            vec!["High-Lift".to_string(), "Speed-Glide".to_string()],
        ))
        .build();

    let mut heavy_wet = SkyVessel::builder()
        .vessel_name("Aura-Cruiser".to_string())
        .auxiliary_wing(wing_wet)
        .build();

    assert!(!heavy_wet.is_complete());
    assert!(matches!(
        heavy_wet.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    let mut selector = TrimSelector::new("High-Lift");
    heavy_wet.walk_choices(&mut selector);

    assert!(heavy_wet.is_complete());
    let heavy = heavy_wet.resolve().expect("resolves after choice is made");
    assert_eq!(
        heavy.auxiliary_wing.as_ref().unwrap().stabilizer_trim,
        "High-Lift"
    );

    (light, heavy)
}
