use serde::{Deserialize, Serialize};
use crate::blueprint::{Blueprint, Choice, ChoiceError, ChoicePrompt};

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

impl<T> SlotMany<T> {
    /// A slot resolves to zero or more finals: a choice may pick several options and an option may
    /// carry several values.
    pub fn resolve(self) -> Result<Vec<T>, ChoiceError> {
        match self {
            SlotMany::Fixed(value) => Ok(value),
            SlotMany::Choose(choice) => choice.make_many(),
        }
    }

    pub fn visible_choices(&self) -> Vec<&dyn ChoicePrompt> {
        match self {
            SlotMany::Fixed(_) => vec![],
            SlotMany::Choose(choice) => {
                vec![choice]
            }
        }
    }

    pub fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt> {
        match self {
            SlotMany::Fixed(_) => None,
            SlotMany::Choose(choice) => {
                if !choice.is_ok() {
                    Some(choice)
                } else {
                    None
                }
            }
        }
    }
}
