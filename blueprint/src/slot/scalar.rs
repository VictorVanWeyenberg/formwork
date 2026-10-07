use crate::cardinality::Cardinality;
use crate::choice::Choice;
use crate::error::BlueprintError;
use serde::{Deserialize, Serialize};

/// A single-value slot. Resolves to `T`.
///
/// Can be:
/// - `Fixed(T)`: Finalized value, no choice required.
/// - `Choose(Choice<T>)`: A choice slot that holds choice state (`selected_index`, `is_made`, `history`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Slot<T> {
    Fixed(T),
    Choose(Choice<T>),
}

impl<T> Slot<T> {
    pub fn fixed(value: T) -> Self {
        Slot::Fixed(value)
    }

    pub fn choose(choice: Choice<T>) -> Self {
        Slot::Choose(choice)
    }

    /// Alias for backwards compatibility.
    pub fn pending(choice: Choice<T>) -> Self {
        Slot::Choose(choice)
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, Slot::Fixed(_))
    }

    pub fn is_pending(&self) -> bool {
        match self {
            Slot::Fixed(_) => false,
            Slot::Choose(c) => !c.is_made(),
        }
    }

    pub fn is_made(&self) -> bool {
        match self {
            Slot::Fixed(_) => false,
            Slot::Choose(c) => c.is_made(),
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Slot::Fixed(_) => true,
            Slot::Choose(c) => c.is_made(),
        }
    }

    pub fn as_fixed(&self) -> Option<&T> {
        match self {
            Slot::Fixed(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_fixed_mut(&mut self) -> Option<&mut T> {
        match self {
            Slot::Fixed(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_choice(&self) -> Option<&Choice<T>> {
        match self {
            Slot::Choose(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_choice_mut(&mut self) -> Option<&mut Choice<T>> {
        match self {
            Slot::Choose(c) => Some(c),
            _ => None,
        }
    }

    pub fn selected_value(&self) -> Option<&T> {
        match self {
            Slot::Fixed(v) => Some(v),
            Slot::Choose(c) => c.selected_value(),
        }
    }

    pub fn selected_value_mut(&mut self) -> Option<&mut T> {
        match self {
            Slot::Fixed(v) => Some(v),
            Slot::Choose(c) => c.selected_value_mut(),
        }
    }

    pub fn prompt(&self) -> &str {
        match self {
            Slot::Fixed(_) => "",
            Slot::Choose(c) => c.prompt(),
        }
    }

    pub fn cardinality(&self) -> Cardinality {
        Cardinality::ONE
    }

    pub fn option_labels(&self) -> Vec<&str> {
        match self {
            Slot::Fixed(_) => vec![],
            Slot::Choose(c) => c.option_labels(),
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        match self {
            Slot::Fixed(_) => None,
            Slot::Choose(c) => c.selected_index(),
        }
    }

    pub fn selected_indices(&self) -> &[usize] {
        match self {
            Slot::Fixed(_) => &[],
            Slot::Choose(c) => c.selected_indices(),
        }
    }


    pub fn select(&mut self, index: usize) -> Result<(), BlueprintError> {
        match self {
            Slot::Choose(c) => c.select(index),
            Slot::Fixed(_) => Err(BlueprintError::Custom("cannot select on a fixed slot".into())),
        }
    }

    pub fn unmake(&mut self) -> Result<(), BlueprintError> {
        match self {
            Slot::Choose(c) => c.unmake(),
            Slot::Fixed(_) => Err(BlueprintError::Custom("cannot unmake a fixed slot".into())),
        }
    }

    pub fn confirm(&mut self) -> Result<(), BlueprintError> {
        match self {
            Slot::Choose(c) => c.confirm(),
            Slot::Fixed(_) => Ok(()),
        }
    }

    /// Resolves the slot to its final value `T`.
    pub fn resolve(self) -> Result<T, BlueprintError> {
        match self {
            Slot::Fixed(value) => Ok(value),
            Slot::Choose(choice) => choice.resolve(),
        }
    }
}

impl<T> From<T> for Slot<T> {
    fn from(value: T) -> Self {
        Slot::Fixed(value)
    }
}

impl<T> From<Choice<T>> for Slot<T> {
    fn from(choice: Choice<T>) -> Self {
        Slot::Choose(choice)
    }
}
