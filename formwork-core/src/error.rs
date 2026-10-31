use crate::choice::Cardinality;
#[cfg(doc)]
use crate::choice::{MultiChoice, SingleChoice};
use thiserror::Error;

/// Errors that can occur during formwork choice manipulation, traversal, or resolution.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum FormworkError {
    /// Attempted to resolve a formwork or slot while a wet choice remained pending or unmade.
    #[error("wet choice remains pending: '{prompt}'")]
    WetChoice {
        /// The prompt associated with the wet choice.
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

    /// An option index provided to [`SingleChoice::select`] was outside the bounds of available options.
    #[error("`{prompt}`: option index {index} is out of bounds (total options: {total_options})")]
    IndexOutOfBounds {
        /// The prompt associated with the choice.
        prompt: String,
        /// The invalid option index.
        index: usize,
        /// Total number of options available.
        total_options: usize,
    },

    /// The same option index was selected more than once in a [`MultiChoice`].
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

    /// Attempted to unselect an option that was not currently selected.
    #[error("`{prompt}`: option index {index} is not currently selected")]
    NotSelected {
        /// The prompt associated with the choice.
        prompt: String,
        /// The unselected index.
        index: usize,
    },
}
