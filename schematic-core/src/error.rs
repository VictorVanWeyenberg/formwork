use crate::choice::Cardinality;
use thiserror::Error;

/// Errors that can occur during schematic choice manipulation, traversal, or resolution.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SchematicError {
    /// Attempted to resolve a schematic or slot while an unresolved choice remained pending or unmade.
    #[error("unresolved choice: `{prompt}`")]
    UnresolvedChoice {
        /// The prompt associated with the unresolved choice.
        prompt: String,
    },

    /// The number of selected options violated the [`Cardinality`] constraint.
    #[error("`{prompt}`: expected {expected}, but {actual} option(s) were selected")]
    InvalidCardinality {
        /// The prompt associated with the choice.
        prompt: String,
        /// The expected cardinality rule.
        expected: Cardinality,
        /// The actual number of selected options.
        actual: usize,
    },

    /// An option index provided to [`select`](crate::choice::SingleChoice::select) was outside the bounds of available options.
    #[error("`{prompt}`: option index {index} is out of bounds (total options: {total_options})")]
    IndexOutOfBounds {
        /// The prompt associated with the choice.
        prompt: String,
        /// The invalid option index.
        index: usize,
        /// Total number of options available.
        total_options: usize,
    },

    /// The same option index was selected more than once in a [`MultiChoice`](crate::choice::MultiChoice).
    #[error("`{prompt}`: option index {index} was selected more than once")]
    DuplicateSelection {
        /// The prompt associated with the choice.
        prompt: String,
        /// The duplicate index.
        index: usize,
    },

    /// Attempted to unmake a choice that was not currently in a made state.
    #[error("`{prompt}`: choice cannot be unmade because it is not currently made")]
    CannotUnmakeNotMade {
        /// The prompt associated with the choice.
        prompt: String,
    },

    /// A custom or application-specific schematic error.
    #[error("{0}")]
    Custom(String),
}
