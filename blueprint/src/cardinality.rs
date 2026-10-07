use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Defines how many options must/may be selected for a choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cardinality {
    /// Exactly `n` options must be picked.
    Exact(usize),
    /// At least `n` options must be picked.
    AtLeast(usize),
    /// At most `n` options may be picked (including 0).
    UpTo(usize),
}

impl Cardinality {
    pub const ONE: Cardinality = Cardinality::Exact(1);

    /// Checks if the given number of selected items satisfies this cardinality constraint.
    pub fn accepts(&self, count: usize) -> bool {
        match self {
            Cardinality::Exact(n) => count == *n,
            Cardinality::AtLeast(n) => count >= *n,
            Cardinality::UpTo(n) => count <= *n,
        }
    }
}

impl Default for Cardinality {
    fn default() -> Self {
        Cardinality::ONE
    }
}

impl Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cardinality::Exact(1) => write!(f, "exactly 1"),
            Cardinality::Exact(n) => write!(f, "exactly {n}"),
            Cardinality::AtLeast(n) => write!(f, "at least {n}"),
            Cardinality::UpTo(n) => write!(f, "up to {n}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinality_accepts() {
        assert!(Cardinality::Exact(1).accepts(1));
        assert!(!Cardinality::Exact(1).accepts(0));
        assert!(!Cardinality::Exact(1).accepts(2));

        assert!(Cardinality::AtLeast(2).accepts(2));
        assert!(Cardinality::AtLeast(2).accepts(3));
        assert!(!Cardinality::AtLeast(2).accepts(1));

        assert!(Cardinality::UpTo(2).accepts(0));
        assert!(Cardinality::UpTo(2).accepts(1));
        assert!(Cardinality::UpTo(2).accepts(2));
        assert!(!Cardinality::UpTo(2).accepts(3));
    }
}
