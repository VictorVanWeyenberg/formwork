use crate::error::SchematicError;
use serde::{Deserialize, Serialize};

/// Cardinality constraint for a [`MultiChoice`].
///
/// Defines how many options can or must be selected from the available options.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cardinality {
    /// Allows selecting up to `n` options (inclusive).
    UpTo(usize),
    /// Requires selecting at least `n` options (inclusive).
    AtLeast(usize),
    /// Requires selecting exactly `n` options.
    #[serde(untagged)]
    Exact(usize),
}

impl std::fmt::Display for Cardinality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cardinality::Exact(n) => write!(f, "exactly {}", n),
            Cardinality::UpTo(n) => write!(f, "up to {}", n),
            Cardinality::AtLeast(n) => write!(f, "at least {}", n),
        }
    }
}

impl Cardinality {
    /// Convenience constant representing exactly one selection.
    pub const ONE: Cardinality = Cardinality::Exact(1);

    /// Checks whether the given selection count satisfies this cardinality constraint.
    pub fn accepts(&self, count: usize) -> bool {
        match self {
            Cardinality::Exact(n) => count == *n,
            Cardinality::UpTo(n) => count <= *n,
            Cardinality::AtLeast(n) => count >= *n,
        }
    }

    /// Returns the maximum number of items permitted under this cardinality constraint.
    pub fn max(&self) -> usize {
        match self {
            Cardinality::Exact(n) => *n,
            Cardinality::UpTo(n) => *n,
            Cardinality::AtLeast(_) => usize::MAX,
        }
    }
}

/// Internal plumbing: tracks the lifecycle state of a choice.
///
/// # Warning
/// This is an **internal plumbing** type used by [`SingleChoice`] and [`MultiChoice`].
/// Application code should never manipulate or inspect `ChoiceState` directly.
/// Instead, interact with choices through their high-level methods:
/// [`select`](SingleChoice::select), [`unmake`](SingleChoice::unmake),
/// and [`confirm`](SingleChoice::confirm).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceState {
    /// Choice has not been made yet.
    #[default]
    Pending,
    /// Choice was previously made but has been unmade, preserving the previous selection history.
    UnmadeWithHistory(Vec<usize>),
    /// Choice has been confirmed with the specified option indices.
    Made(Vec<usize>),
}

fn is_default_state(s: &ChoiceState) -> bool {
    *s == ChoiceState::Pending
}

/// A user-facing choice where exactly one option must be selected.
///
/// `SingleChoice` encapsulates a human-readable prompt and a collection of candidate options
/// of type `T`. It tracks whether a selection has been made, unmade, or is still pending.
///
/// # Example
/// ```rust
/// use schematic_core::choice::SingleChoice;
///
/// let mut choice = SingleChoice::new(
///     "Pick a starter tool",
///     vec!["Smith's Tools".to_string(), "Brewer's Supplies".to_string()],
/// );
/// assert!(!choice.is_made());
///
/// choice.select(0).unwrap();
/// assert!(choice.is_made());
/// assert_eq!(choice.resolve_exact().unwrap(), "Smith's Tools");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SingleChoice<T> {
    pub prompt: String,
    pub options: Vec<T>,
    #[serde(default, skip_serializing_if = "is_default_state")]
    pub state: ChoiceState,
}

impl<T> SingleChoice<T> {
    pub fn new(prompt: impl Into<String>, options: impl IntoIterator<Item = T>) -> Self {
        Self {
            prompt: prompt.into(),
            options: options.into_iter().collect(),
            state: ChoiceState::Pending,
        }
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn options(&self) -> &[T] {
        &self.options
    }

    pub fn is_made(&self) -> bool {
        matches!(self.state, ChoiceState::Made(_))
    }

    pub fn selected_indices(&self) -> &[usize] {
        match &self.state {
            ChoiceState::Made(idx) | ChoiceState::UnmadeWithHistory(idx) => idx.as_slice(),
            ChoiceState::Pending => &[],
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_indices().first().copied()
    }

    pub fn selected_values(&self) -> Vec<&T> {
        let indices = self.selected_indices();
        self.options
            .iter()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, opt)| opt)
            .collect()
    }

    pub fn selected_values_mut(&mut self) -> Vec<&mut T> {
        let indices = match &self.state {
            ChoiceState::Made(idx) | ChoiceState::UnmadeWithHistory(idx) => idx.clone(),
            ChoiceState::Pending => vec![],
        };
        self.options
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, opt)| opt)
            .collect()
    }

    pub fn select(&mut self, index: usize) -> Result<(), SchematicError> {
        if index >= self.options.len() {
            return Err(SchematicError::IndexOutOfBounds {
                prompt: self.prompt.clone(),
                index,
                total_options: self.options.len(),
            });
        }
        if let ChoiceState::Made(_) = self.state {
            return Err(SchematicError::Custom("Choice already made".into()));
        }
        self.state = ChoiceState::Made(vec![index]);
        Ok(())
    }

    pub fn unmake(&mut self) -> Result<(), SchematicError> {
        match &self.state {
            ChoiceState::Made(idx) => {
                self.state = ChoiceState::UnmadeWithHistory(idx.clone());
                Ok(())
            }
            ChoiceState::UnmadeWithHistory(_) => {
                self.state = ChoiceState::Pending;
                Ok(())
            }
            ChoiceState::Pending => Err(SchematicError::CannotUnmakeNotMade {
                prompt: self.prompt.clone(),
            }),
        }
    }

    pub fn confirm(&mut self) -> Result<(), SchematicError> {
        if let ChoiceState::Made(_) = self.state {
            return Ok(());
        }
        let indices = match &self.state {
            ChoiceState::Pending => vec![],
            ChoiceState::UnmadeWithHistory(idx) => idx.clone(),
            ChoiceState::Made(idx) => idx.clone(),
        };

        if indices.len() == 1 {
            self.state = ChoiceState::Made(indices);
            Ok(())
        } else {
            Err(SchematicError::InvalidCardinality {
                prompt: self.prompt.clone(),
                expected: Cardinality::Exact(1),
                actual: indices.len(),
            })
        }
    }

    pub fn resolve_exact(self) -> Result<T, SchematicError> {
        if !self.is_made() {
            return Err(SchematicError::UnresolvedChoice {
                prompt: self.prompt,
            });
        }
        let indices = match self.state {
            ChoiceState::Made(idx) => idx,
            _ => unreachable!(),
        };
        let idx = indices[0];
        let mut options = self.options;
        Ok(options.remove(idx))
    }
}

fn default_cardinality() -> Cardinality {
    Cardinality::Exact(1)
}

fn is_default_cardinality(c: &Cardinality) -> bool {
    *c == Cardinality::Exact(1)
}

/// A user-facing choice where multiple options can be selected based on a [`Cardinality`] constraint.
///
/// `MultiChoice` holds a prompt, a list of candidate options, and a [`Cardinality`] rule
/// (such as `Exact(2)`, `UpTo(3)`, or `AtLeast(1)`). When selections meet the maximum cardinality,
/// the choice is automatically marked made. For range cardinalities like `UpTo` or `AtLeast`,
/// calling [`confirm`](MultiChoice::confirm) verifies the selection count before marking it made.
///
/// # Example
/// ```rust
/// use schematic_core::choice::{MultiChoice, Cardinality};
///
/// let mut choice = MultiChoice::new(
///     "Pick two bonus skills",
///     vec!["Athletics".to_string(), "Stealth".to_string(), "Perception".to_string()],
/// ).with_cardinality(Cardinality::Exact(2));
///
/// choice.select(0).unwrap();
/// assert!(!choice.is_made());
///
/// choice.select(2).unwrap();
/// assert!(choice.is_made());
/// assert_eq!(choice.resolve().unwrap(), vec!["Athletics", "Perception"]);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiChoice<T> {
    pub prompt: String,
    pub options: Vec<T>,
    #[serde(
        rename = "choose",
        default = "default_cardinality",
        skip_serializing_if = "is_default_cardinality"
    )]
    pub cardinality: Cardinality,
    #[serde(default, skip_serializing_if = "is_default_state")]
    pub state: ChoiceState,
}

impl<T> MultiChoice<T> {
    pub fn new(prompt: impl Into<String>, options: impl IntoIterator<Item = T>) -> Self {
        Self {
            prompt: prompt.into(),
            options: options.into_iter().collect(),
            cardinality: Cardinality::Exact(1),
            state: ChoiceState::Pending,
        }
    }

    pub fn with_cardinality(mut self, cardinality: Cardinality) -> Self {
        self.cardinality = cardinality;
        self
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn options(&self) -> &[T] {
        &self.options
    }

    pub fn is_made(&self) -> bool {
        matches!(self.state, ChoiceState::Made(_))
    }

    pub fn selected_indices(&self) -> &[usize] {
        match &self.state {
            ChoiceState::Made(idx) | ChoiceState::UnmadeWithHistory(idx) => idx.as_slice(),
            ChoiceState::Pending => &[],
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_indices().first().copied()
    }

    pub fn selected_values(&self) -> Vec<&T> {
        let indices = self.selected_indices();
        self.options
            .iter()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, opt)| opt)
            .collect()
    }

    pub fn selected_values_mut(&mut self) -> Vec<&mut T> {
        let indices = match &self.state {
            ChoiceState::Made(idx) | ChoiceState::UnmadeWithHistory(idx) => idx.clone(),
            ChoiceState::Pending => vec![],
        };
        self.options
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, opt)| opt)
            .collect()
    }

    pub fn select(&mut self, index: usize) -> Result<(), SchematicError> {
        if index >= self.options.len() {
            return Err(SchematicError::IndexOutOfBounds {
                prompt: self.prompt.clone(),
                index,
                total_options: self.options.len(),
            });
        }

        let mut indices = match &self.state {
            ChoiceState::Pending => vec![],
            ChoiceState::UnmadeWithHistory(history) => {
                if self.cardinality.max() == 1 {
                    vec![]
                } else {
                    history.clone()
                }
            }
            ChoiceState::Made(_) => {
                return Err(SchematicError::Custom("Choice already made".into()))
            }
        };

        if !indices.contains(&index) {
            indices.push(index);
        } else {
            return Err(SchematicError::DuplicateSelection {
                prompt: self.prompt.clone(),
                index,
            });
        }

        if indices.len() == self.cardinality.max() {
            self.state = ChoiceState::Made(indices);
        } else {
            self.state = ChoiceState::UnmadeWithHistory(indices);
        }

        Ok(())
    }

    pub fn unmake(&mut self) -> Result<(), SchematicError> {
        match &self.state {
            ChoiceState::Made(indices) | ChoiceState::UnmadeWithHistory(indices) => {
                let mut new_history = indices.clone();
                if self.cardinality.max() > 1 {
                    new_history.pop();
                }
                if new_history.is_empty() {
                    self.state = ChoiceState::Pending;
                } else {
                    self.state = ChoiceState::UnmadeWithHistory(new_history);
                }
                Ok(())
            }
            ChoiceState::Pending => Err(SchematicError::CannotUnmakeNotMade {
                prompt: self.prompt.clone(),
            }),
        }
    }

    pub fn confirm(&mut self) -> Result<(), SchematicError> {
        if let ChoiceState::Made(_) = self.state {
            return Ok(());
        }

        let indices = match &self.state {
            ChoiceState::Pending => vec![],
            ChoiceState::UnmadeWithHistory(idx) => idx.clone(),
            ChoiceState::Made(idx) => idx.clone(),
        };

        if self.cardinality.accepts(indices.len()) {
            self.state = ChoiceState::Made(indices);
            Ok(())
        } else {
            Err(SchematicError::InvalidCardinality {
                prompt: self.prompt.clone(),
                expected: self.cardinality,
                actual: indices.len(),
            })
        }
    }

    pub fn resolve(mut self) -> Result<Vec<T>, SchematicError> {
        if !self.is_made() {
            return Err(SchematicError::UnresolvedChoice {
                prompt: self.prompt,
            });
        }

        let indices = match self.state {
            ChoiceState::Made(idx) => idx,
            _ => unreachable!(),
        };

        let mut sorted_indices = indices.clone();
        sorted_indices.sort_unstable();
        sorted_indices.dedup();

        let mut result = Vec::new();
        // Remove from the end to avoid shifting indices
        for idx in sorted_indices.into_iter().rev() {
            if idx < self.options.len() {
                result.push(self.options.remove(idx));
            }
        }
        result.reverse();

        Ok(result)
    }
}
