use serde::{Deserialize, Serialize};
use crate::blueprint::{Blueprint, Choice, ChoiceError, ChoicePrompt, Mode};

/// One position in a list of `T`s that may still have to be decided.
///
/// `Fixed` is transparent in JSON (a fixed value costs nothing extra); `Choose` is recognised by
/// its `choose` key. Variant order matters for `untagged`: a value never has a `choose` key, but
/// try the choice first anyway so a malformed choice is not silently read as a value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlotOne<M: Mode> {
    Choose(Choice<M::Draft>),
    Fixed(M::Final),
}

impl<M: Mode> From<M::Final> for SlotOne<M> {
    fn from(value: M::Final) -> Self {
        SlotOne::Fixed(value)
    }
}

impl<M: Mode> From<Choice<M::Draft>> for SlotOne<M> {
    fn from(choice: Choice<M::Draft>) -> Self {
        SlotOne::Choose(choice)
    }
}

impl<F, D> SlotOne<F, D> {
    pub fn fixed(&self) -> Option<&F> {
        match self {
            SlotOne::Fixed(value) => Some(value),
            SlotOne::Choose(_) => None,
        }
    }

    pub fn choice(&self) -> Option<&Choice<F>> {
        match self {
            SlotOne::Choose(choice) => Some(choice),
            SlotOne::Fixed(_) => None,
        }
    }

    pub fn choice_mut(&mut self) -> Option<&mut Choice<F>> {
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