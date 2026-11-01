//! # Chapter 4: The Council's Decree (Pick Leaf Slot: `#[slot(pick)]`)
//!
//! The Grand Council convenes in the high spire. To balance your celestial soul-matrix,
//! the Council decrees that you must attune to a specific combination of elemental runes
//! chosen from the primordial candidate pool.
//!
//! ## Concept: The Pick Leaf Slot
//!
//! Unlike `#[slot(collection)]` (where each element is an independent choice or item),
//! `#[slot(pick)]` selects a subset from a single candidate pool subject to cardinality
//! rules:
//!
//! - **Wet Formwork (`RunicMatrixFormwork`)**: The field has type `MultiSlot<String, String>`,
//!   holding an unconfirmed [`MultiChoice<String>`](crate::choice::MultiChoice) or a pre-selected list.
//! - **Dry Domain Model (`RunicMatrix`)**: The field has type `Vec<String>`.
//!
//! ### Cardinality Constraints
//!
//! A [`MultiChoice`] is governed by [`Cardinality`]:
//! - [`Cardinality::Exact(n)`](crate::choice::Cardinality::Exact): Exactly `n` options must be selected.
//! - [`Cardinality::AtLeast(n)`](crate::choice::Cardinality::AtLeast): At least `n` options must be selected.
//! - [`Cardinality::UpTo(n)`](crate::choice::Cardinality::UpTo): At most `n` options may be selected.
//!
//! Calling `.confirm()` on a `MultiChoice` before the cardinality constraint is satisfied returns
//! [`FormworkError::InvalidCardinality`].
//!
//! ## Executable Example
//!
//! ```rust
//! use formwork::choice::{Cardinality, MultiChoice};
//! use formwork::tutorial::ch04_pick_leaf::{RunicMatrix, MatrixAttuner};
//! use formwork::Formwork;
//!
//! // Build the matrix with an open MultiChoice requiring exactly 2 runes
//! let mut wet_matrix = RunicMatrix::builder()
//!     .matrix_name("Dawn-Ward".to_string())
//!     .attuned_runes(MultiChoice::new(
//!         "Select 2 Primordial Runes",
//!         vec![
//!             "Solar".to_string(),
//!             "Void".to_string(),
//!             "Starlight".to_string(),
//!             "Gale".to_string(),
//!         ],
//!     ).with_cardinality(Cardinality::Exact(2)))
//!     .build();
//!
//! assert!(!wet_matrix.is_complete());
//!
//! // Attune "Solar" and "Starlight" runes
//! let mut attuner = MatrixAttuner::new(vec!["Solar", "Starlight"]);
//! wet_matrix.walk_choices(&mut attuner);
//!
//! assert!(wet_matrix.is_complete());
//! let dry_matrix = wet_matrix.resolve().expect("attunement satisfied");
//! assert_eq!(dry_matrix.attuned_runes, vec!["Solar".to_string(), "Starlight".to_string()]);
//! ```

use crate::choice::{Cardinality, MultiChoice, SingleChoice};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use crate::{Formwork, FormworkError, formwork};

/// A Realm-Crafter's attuned runic matrix.
///
/// In wet mode (`RunicMatrixFormwork`), `attuned_runes` is a `MultiSlot<String, String>`.
/// In dry mode (`RunicMatrix`), it resolves into a watertight `Vec<String>`.
#[formwork(builder)]
#[derive(Clone, Debug, PartialEq)]
pub struct RunicMatrix {
    /// The name of this matrix configuration (fixed invariant).
    pub matrix_name: String,
    /// A multi-selection of elemental runes (pick leaf slot).
    #[slot(pick)]
    pub attuned_runes: Vec<String>,
}

/// A visitor implementing multi-selection by matching target rune names in candidate pools.
pub struct MatrixAttuner {
    targets: Vec<String>,
}

impl MatrixAttuner {
    /// Creates a new `MatrixAttuner` with the specified target rune names.
    pub fn new<I, S>(targets: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            targets: targets.into_iter().map(Into::into).collect(),
        }
    }
}

impl ChoiceVisitor<String> for MatrixAttuner {
    fn visit_single(&mut self, _choice: &mut SingleChoice<String>) -> VisitorFlow {
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<String>) -> VisitorFlow {
        for target in &self.targets {
            if let Some(pos) = choice.options().iter().position(|opt| opt == target) {
                let _ = choice.select(pos);
            }
        }
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

/// Runs the Chapter 4 tutorial, demonstrating pick slot configuration, cardinality, and resolution.
///
/// Returns the resolved watertight [`RunicMatrix`] domain model.
pub fn run() -> RunicMatrix {
    let mut wet_matrix = RunicMatrix::builder()
        .matrix_name("Dawn-Ward".to_string())
        .attuned_runes(
            MultiChoice::new(
                "Select 2 Primordial Runes",
                vec![
                    "Solar".to_string(),
                    "Void".to_string(),
                    "Starlight".to_string(),
                    "Gale".to_string(),
                ],
            )
            .with_cardinality(Cardinality::Exact(2)),
        )
        .build();

    assert!(!wet_matrix.is_complete());
    assert!(matches!(
        wet_matrix.clone().resolve(),
        Err(FormworkError::WetChoice { .. })
    ));

    let mut attuner = MatrixAttuner::new(vec!["Solar", "Starlight"]);
    wet_matrix.walk_choices(&mut attuner);

    assert!(wet_matrix.is_complete());
    let dry = wet_matrix.resolve().expect("resolved successfully");
    assert_eq!(
        dry.attuned_runes,
        vec!["Solar".to_string(), "Starlight".to_string()]
    );

    dry
}
