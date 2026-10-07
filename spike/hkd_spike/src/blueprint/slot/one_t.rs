use serde::{Deserialize, Serialize};
use crate::blueprint::{Blueprint, Choice, ChoiceError, ChoicePrompt};

/// One position in a list of `T`s that may still have to be decided.
///
/// `Fixed` is transparent in JSON (a fixed value costs nothing extra); `Choose` is recognised by
/// its `choose` key. Variant order matters for `untagged`: a value never has a `choose` key, but
/// try the choice first anyway so a malformed choice is not silently read as a value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlotOneTransitive<T: Blueprint> {
    Choose(Choice<T>),
    Fixed(T),
}

impl<T: Blueprint> From<T> for SlotOneTransitive<T> {
    fn from(value: T) -> Self {
        SlotOneTransitive::Fixed(value)
    }
}

impl<T: Blueprint> From<Choice<T>> for SlotOneTransitive<T> {
    fn from(choice: Choice<T>) -> Self {
        SlotOneTransitive::Choose(choice)
    }
}

impl<T: Blueprint> SlotOneTransitive<T> {
    pub fn fixed(&self) -> Option<&T> {
        match self {
            SlotOneTransitive::Fixed(value) => Some(value),
            SlotOneTransitive::Choose(_) => None,
        }
    }

    pub fn choice(&self) -> Option<&Choice<T>> {
        match self {
            SlotOneTransitive::Choose(choice) => Some(choice),
            SlotOneTransitive::Fixed(_) => None,
        }
    }

    pub fn choice_mut(&mut self) -> Option<&mut Choice<T>> {
        match self {
            SlotOneTransitive::Choose(choice) => Some(choice),
            SlotOneTransitive::Fixed(_) => None,
        }
    }
}

impl<T: Blueprint> SlotOneTransitive<T> {
    pub fn resolve(self) -> Result<T::Final, ChoiceError> {
        match self {
            SlotOneTransitive::Fixed(value) => value.resolve(),
            SlotOneTransitive::Choose(choice) => choice.make_one()?.resolve(),
        }
    }

    pub fn visible_choices(&self) -> Vec<&dyn ChoicePrompt> {
        match self {
            SlotOneTransitive::Fixed(value) => value.visible_choices(),
            SlotOneTransitive::Choose(choice) => {
                let mut out: Vec<&dyn ChoicePrompt> = vec![choice];
                out.extend(choice.selected_values().flat_map(T::visible_choices));
                out
            }
        }
    }

    pub fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt> {
        match self {
            SlotOneTransitive::Fixed(value) => value.next_open_choice_mut(),
            SlotOneTransitive::Choose(choice) => {
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