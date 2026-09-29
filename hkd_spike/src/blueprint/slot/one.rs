use serde::{Deserialize, Serialize};
use crate::blueprint::{Choice, ChoiceError, ChoicePrompt};

/// One position in a list of `T`s that may still have to be decided.
///
/// `Fixed` is transparent in JSON (a fixed value costs nothing extra); `Choose` is recognised by
/// its `choose` key. Variant order matters for `untagged`: a value never has a `choose` key, but
/// try the choice first anyway so a malformed choice is not silently read as a value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlotOne<T> {
    Choose(Choice<T>),
    Fixed(T),
}

impl<T> From<T> for SlotOne<T> {
    fn from(value: T) -> Self {
        SlotOne::Fixed(value)
    }
}

impl<T> From<Choice<T>> for SlotOne<T> {
    fn from(choice: Choice<T>) -> Self {
        SlotOne::Choose(choice)
    }
}

impl<T> SlotOne<T> {
    pub fn fixed(&self) -> Option<&T> {
        match self {
            SlotOne::Fixed(value) => Some(value),
            SlotOne::Choose(_) => None,
        }
    }

    pub fn choice(&self) -> Option<&Choice<T>> {
        match self {
            SlotOne::Choose(choice) => Some(choice),
            SlotOne::Fixed(_) => None,
        }
    }

    pub fn choice_mut(&mut self) -> Option<&mut Choice<T>> {
        match self {
            SlotOne::Choose(choice) => Some(choice),
            SlotOne::Fixed(_) => None,
        }
    }
}

impl<T> SlotOne<T> {
    pub fn resolve(self) -> Result<T, ChoiceError> {
        match self {
            SlotOne::Fixed(value) => Ok(value),
            SlotOne::Choose(choice) => choice.make_one(),
        }
    }

    pub fn visible_choices(&self) -> Vec<&dyn ChoicePrompt> {
        match self {
            SlotOne::Fixed(_) => vec![],
            SlotOne::Choose(choice) => {
                vec![choice]
            }
        }
    }

    pub fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt> {
        match self {
            SlotOne::Fixed(_) => None,
            SlotOne::Choose(choice) => {
                if !choice.is_ok() {
                    Some(choice)
                } else {
                    None
                }
            }
        }
    }
}