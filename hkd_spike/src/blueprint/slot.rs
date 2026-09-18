use crate::blueprint::{Blueprint, Choice, ChoiceError, ChoicePrompt};
use serde::{Deserialize, Serialize};

/// One position in a list of `T`s that may still have to be decided.
///
/// `Fixed` is transparent in JSON (a fixed value costs nothing extra); `Choose` is recognised by
/// its `choose` key. Variant order matters for `untagged`: a value never has a `choose` key, but
/// try the choice first anyway so a malformed choice is not silently read as a value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Slot<T> {
    Choose(Choice<T>),
    Fixed(T),
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

impl<T> Slot<T> {
    pub fn fixed(&self) -> Option<&T> {
        match self {
            Slot::Fixed(value) => Some(value),
            Slot::Choose(_) => None,
        }
    }

    pub fn choice(&self) -> Option<&Choice<T>> {
        match self {
            Slot::Choose(choice) => Some(choice),
            Slot::Fixed(_) => None,
        }
    }

    pub fn choice_mut(&mut self) -> Option<&mut Choice<T>> {
        match self {
            Slot::Choose(choice) => Some(choice),
            Slot::Fixed(_) => None,
        }
    }
}

impl<T: Blueprint> Slot<T> {
    /// A slot resolves to zero or more finals: a choice may pick several options and an option may
    /// carry several values.
    pub fn resolve(self) -> Result<Vec<T::Final>, ChoiceError> {
        match self {
            Slot::Fixed(value) => Ok(vec![value.resolve()?]),
            Slot::Choose(choice) => choice.make()?.into_iter().map(T::resolve).collect(),
        }
    }

    pub fn visible_choices(&self) -> Vec<&dyn ChoicePrompt> {
        match self {
            Slot::Fixed(value) => value.visible_choices(),
            Slot::Choose(choice) => {
                let mut out: Vec<&dyn ChoicePrompt> = vec![choice];
                out.extend(choice.selected_values().flat_map(T::visible_choices));
                out
            }
        }
    }

    pub fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt> {
        match self {
            Slot::Fixed(value) => value.next_open_choice_mut(),
            Slot::Choose(choice) => {
                if !choice.is_ok() {
                    return Some(choice);
                }
                choice.selected_values_mut().find_map(T::next_open_choice_mut)
            }
        }
    }
}

/// Resolve a whole slot list, flattening the picks.
pub fn resolve_slots<T: Blueprint>(slots: Vec<Slot<T>>) -> Result<Vec<T::Final>, ChoiceError> {
    let mut out = Vec::with_capacity(slots.len());
    for slot in slots {
        out.extend(slot.resolve()?);
    }
    Ok(out)
}
