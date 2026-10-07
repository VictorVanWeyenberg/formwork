use crate::cardinality::Cardinality;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum BlueprintError {
    #[error("unresolved choice: `{prompt}`")]
    UnresolvedChoice { prompt: String },

    #[error("`{prompt}`: expected {expected}, but {actual} option(s) were selected")]
    InvalidCardinality {
        prompt: String,
        expected: Cardinality,
        actual: usize,
    },

    #[error("`{prompt}`: option index {index} is out of bounds (total options: {total_options})")]
    IndexOutOfBounds {
        prompt: String,
        index: usize,
        total_options: usize,
    },

    #[error("`{prompt}`: option index {index} was selected more than once")]
    DuplicateSelection { prompt: String, index: usize },

    #[error("`{prompt}`: choice cannot be unmade because it is not currently made")]
    CannotUnmakeNotMade { prompt: String },

    #[error("{0}")]
    Custom(String),
}
