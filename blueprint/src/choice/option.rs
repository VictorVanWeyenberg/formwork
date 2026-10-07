use serde::{Deserialize, Serialize};

/// A single option within a choice, with a human-readable label and the concrete value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceOption<T> {
    pub label: String,
    pub value: T,
}

impl<T> ChoiceOption<T> {
    pub fn new(label: impl Into<String>, value: T) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}
