use crate::choice::{Choice};
use crate::error::BlueprintError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Slot<T> {
    Choose(Choice<T>),
    Fixed(T),
}

impl<T> Slot<T> {
    pub fn fixed(value: T) -> Self {
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

    pub fn selected_values(&self) -> Vec<&T> {
        match self {
            Slot::Fixed(v) => vec![v],
            Slot::Choose(c) => c.selected_values(),
        }
    }

    pub fn selected_values_mut(&mut self) -> Vec<&mut T> {
        match self {
            Slot::Fixed(v) => vec![v],
            Slot::Choose(c) => c.selected_values_mut(),
        }
    }

    pub fn resolve(self) -> Result<Vec<T>, BlueprintError> {
        match self {
            Slot::Fixed(value) => Ok(vec![value]),
            Slot::Choose(choice) => choice.resolve(),
        }
    }

    pub fn resolve_exact(self) -> Result<T, BlueprintError> {
        match self {
            Slot::Fixed(value) => Ok(value),
            Slot::Choose(choice) => choice.resolve_exact(),
        }
    }
}

impl<T> From<T> for Slot<T> {
    fn from(value: T) -> Self {
        Slot::fixed(value)
    }
}



impl<T> From<Choice<T>> for Slot<T> {
    fn from(choice: Choice<T>) -> Self {
        Slot::Choose(choice)
    }
}



impl<T: Default> Default for Slot<T> {
    fn default() -> Self {
        Slot::Fixed(T::default())
    }
}
