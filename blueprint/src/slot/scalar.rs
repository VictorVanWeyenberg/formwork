use crate::choice::{Choice, Cardinality};
use crate::error::BlueprintError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Slot<T> {
    Fixed(Vec<T>),
    Choose(Choice<T>),
}

impl<T> Slot<T> {
    pub fn fixed(value: T) -> Self {
        Slot::Fixed(vec![value])
    }

    pub fn fixed_vec(value: Vec<T>) -> Self {
        Slot::Fixed(value)
    }

    pub fn choose(choice: Choice<T>) -> Self {
        Slot::Choose(choice)
    }

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

    pub fn as_fixed(&self) -> Option<&[T]> {
        match self {
            Slot::Fixed(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    pub fn as_fixed_mut(&mut self) -> Option<&mut Vec<T>> {
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

    pub fn selected_values(&self) -> Vec<&T> {
        match self {
            Slot::Fixed(v) => v.iter().collect(),
            Slot::Choose(c) => c.selected_values(),
        }
    }

    pub fn selected_values_mut(&mut self) -> Vec<&mut T> {
        match self {
            Slot::Fixed(v) => v.iter_mut().collect(),
            Slot::Choose(c) => c.selected_values_mut(),
        }
    }

    pub fn prompt(&self) -> &str {
        match self {
            Slot::Fixed(_) => "",
            Slot::Choose(c) => c.prompt(),
        }
    }

    pub fn cardinality(&self) -> Cardinality {
        match self {
            Slot::Fixed(v) => Cardinality::Exact(v.len()),
            Slot::Choose(c) => c.cardinality,
        }
    }

    pub fn option_labels(&self) -> Vec<&str> {
        match self {
            Slot::Fixed(_) => vec![],
            Slot::Choose(c) => c.option_labels(),
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

    pub fn resolve(self) -> Result<Vec<T>, BlueprintError> {
        match self {
            Slot::Fixed(value) => Ok(value),
            Slot::Choose(choice) => choice.resolve(),
        }
    }
}

impl<T> From<T> for Slot<T> {
    fn from(value: T) -> Self {
        Slot::fixed(value)
    }
}

impl<T> From<Vec<T>> for Slot<T> {
    fn from(value: Vec<T>) -> Self {
        Slot::fixed_vec(value)
    }
}

impl<T> From<Choice<T>> for Slot<T> {
    fn from(choice: Choice<T>) -> Self {
        Slot::Choose(choice)
    }
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Slot::Fixed(Vec::new())
    }
}
