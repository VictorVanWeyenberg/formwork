use crate::blueprint::{Blueprint, Choice, ChoiceError, ChoicePrompt};
use serde::{Deserialize, Serialize};

/// One position in a list of `T`s that may still have to be decided.
///
/// `Fixed` is transparent in JSON (a fixed value costs nothing extra); `Choose` is recognised by
/// its `choose` key. Variant order matters for `untagged`: a value never has a `choose` key, but
/// try the choice first anyway so a malformed choice is not silently read as a value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SlotManyTransitive<T: Blueprint> {
    Choose(Choice<T>),
    Fixed(Vec<T>),
}

impl<T: Blueprint> From<Vec<T>> for SlotManyTransitive<T> {
    fn from(value: Vec<T>) -> Self {
        SlotManyTransitive::Fixed(value)
    }
}

impl<T: Blueprint> From<Choice<T>> for SlotManyTransitive<T> {
    fn from(choice: Choice<T>) -> Self {
        SlotManyTransitive::Choose(choice)
    }
}

impl<T: Blueprint> SlotManyTransitive<T> {
    pub fn fixed(&self) -> Option<&Vec<T>> {
        match self {
            SlotManyTransitive::Fixed(value) => Some(value),
            SlotManyTransitive::Choose(_) => None,
        }
    }

    pub fn choice(&self) -> Option<&Choice<T>> {
        match self {
            SlotManyTransitive::Choose(choice) => Some(choice),
            SlotManyTransitive::Fixed(_) => None,
        }
    }

    pub fn choice_mut(&mut self) -> Option<&mut Choice<T>> {
        match self {
            SlotManyTransitive::Choose(choice) => Some(choice),
            SlotManyTransitive::Fixed(_) => None,
        }
    }
}

impl<T: Blueprint> SlotManyTransitive<T> {
    pub fn resolve(self) -> Result<Vec<T::Final>, ChoiceError> {
        match self {
            SlotManyTransitive::Fixed(value) => value
                .into_iter()
                .map(|x| x.resolve())
                .collect::<Result<Vec<_>, _>>(),
            SlotManyTransitive::Choose(choice) => choice
                .make_many()?
                .into_iter()
                .map(|x| x.resolve())
                .collect::<Result<Vec<_>, _>>(),
        }
    }

    pub fn visible_choices(&self) -> Vec<&dyn ChoicePrompt> {
        match self {
            SlotManyTransitive::Fixed(value) => {
                value.iter().flat_map(|x| x.visible_choices()).collect()
            }
            SlotManyTransitive::Choose(choice) => {
                let mut out: Vec<&dyn ChoicePrompt> = vec![choice];
                out.extend(choice.selected_values().flat_map(T::visible_choices));
                out
            }
        }
    }

    pub fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt> {
        match self {
            SlotManyTransitive::Fixed(value) => {
                value.iter_mut().find_map(|x| x.next_open_choice_mut())
            }
            SlotManyTransitive::Choose(choice) => {
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
