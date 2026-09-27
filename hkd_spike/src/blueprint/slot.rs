use crate::blueprint::{Blueprint, Cardinality, Choice, ChoiceError, ChoicePrompt};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlotMany<T> {
    Choose(Choice<T>),
    Fixed(Vec<T>),
}

impl<T> From<Vec<T>> for SlotMany<T> {
    fn from(value: Vec<T>) -> Self {
        SlotMany::Fixed(value)
    }
}

impl<T> From<Choice<T>> for SlotMany<T> {
    fn from(choice: Choice<T>) -> Self {
        SlotMany::Choose(choice)
    }
}

impl<T> SlotMany<T> {
    pub fn fixed(&self) -> Option<&Vec<T>> {
        match self {
            SlotMany::Fixed(value) => Some(value),
            SlotMany::Choose(_) => None,
        }
    }

    pub fn choice(&self) -> Option<&Choice<T>> {
        match self {
            SlotMany::Choose(choice) => Some(choice),
            SlotMany::Fixed(_) => None,
        }
    }

    pub fn choice_mut(&mut self) -> Option<&mut Choice<T>> {
        match self {
            SlotMany::Choose(choice) => Some(choice),
            SlotMany::Fixed(_) => None,
        }
    }
}

impl<T: Blueprint> SlotMany<T> {
    /// A slot resolves to zero or more finals: a choice may pick several options and an option may
    /// carry several values.
    pub fn resolve(self) -> Result<Vec<T::Final>, ChoiceError> {
        match self {
            SlotMany::Fixed(value) => value.into_iter().map(|b| b.resolve()).collect(),
            SlotMany::Choose(choice) => choice.make_many()?.into_iter().map(T::resolve).collect(),
        }
    }

    pub fn visible_choices(&self) -> Vec<&dyn ChoicePrompt> {
        match self {
            SlotMany::Fixed(value) => value.iter().flat_map(|b| b.visible_choices()).collect(),
            SlotMany::Choose(choice) => {
                let mut out: Vec<&dyn ChoicePrompt> = vec![choice];
                out.extend(choice.selected_values().flat_map(T::visible_choices));
                out
            }
        }
    }

    pub fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt> {
        match self {
            SlotMany::Fixed(value) => value.iter_mut().find_map(|mut b| b.next_open_choice_mut()),
            SlotMany::Choose(choice) => {
                if !choice.is_ok() {
                    return Some(choice);
                }
                choice
                    .selected_values_mut()
                    .find_map(T::next_open_choice_mut)
            }
        }
    }
}
