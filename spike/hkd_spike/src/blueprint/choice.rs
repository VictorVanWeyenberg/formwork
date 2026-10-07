use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::num::NonZeroUsize;

/// JSON: `1` (exactly one), `2` (exactly two), `{"upTo": 2}`, `{"atLeast": 1}`.
///
/// serde requires the `untagged` variant to be declared *last*.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cardinality {
    UpTo(NonZeroUsize),
    AtLeast(NonZeroUsize),
    #[serde(untagged)]
    Exactly(NonZeroUsize),
}

impl Cardinality {
    pub const ONE: Cardinality = Cardinality::Exactly(NonZeroUsize::MIN);

    pub fn exactly(n: usize) -> Self {
        Self::Exactly(non_zero(n))
    }

    pub fn up_to(n: usize) -> Self {
        Self::UpTo(non_zero(n))
    }

    pub fn at_least(n: usize) -> Self {
        Self::AtLeast(non_zero(n))
    }

    pub fn accepts(&self, picked: usize) -> bool {
        match self {
            Cardinality::Exactly(n) => picked == n.get(),
            Cardinality::UpTo(n) => picked <= n.get(),
            Cardinality::AtLeast(n) => picked >= n.get(),
        }
    }
}

impl Default for Cardinality {
    fn default() -> Self {
        Cardinality::ONE
    }
}

fn non_zero(n: usize) -> NonZeroUsize {
    NonZeroUsize::new(n).expect("a cardinality of zero makes no sense")
}

impl Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cardinality::Exactly(n) if n.get() == 1 => write!(f, "one"),
            Cardinality::Exactly(n) => write!(f, "exactly {n}"),
            Cardinality::UpTo(n) => write!(f, "up to {n}"),
            Cardinality::AtLeast(n) => write!(f, "at least {n}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChoiceError {
    BadCardinality {
        prompt: String,
        expected: Cardinality,
        got: usize,
    },
    UnknownOption {
        prompt: String,
        label: String,
    },
}

impl Display for ChoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChoiceError::BadCardinality { prompt, expected, got } => {
                write!(f, "`{prompt}`: expected {expected} option(s), {got} selected")
            }
            ChoiceError::UnknownOption { prompt, label } => {
                write!(f, "`{prompt}`: `{label}` is not one of the options")
            }
        }
    }
}

impl std::error::Error for ChoiceError {}

/// One option: a label and the complete value(s) it stands for. No patches: the option *is* the
/// delta, at the position it fills.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChoiceOption<T> {
    pub label: String,
    pub value: T,
}

impl<T> ChoiceOption<T> {
    pub fn new(label: impl Into<String>, value: impl Into<T>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn values(&self) -> &T {
        &self.value
    }
}

/// A choice sits *at the position it fills*. Selection state (`selected`, by label) only appears
/// on an in-progress character, never in a book.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Choice<T> {
    #[serde(rename = "choose")]
    #[serde(skip_serializing_if = "cardinality_is_default")]
    pub cardinality: Cardinality,
    pub prompt: String,
    pub options: Vec<ChoiceOption<T>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected: Vec<String>,
}

fn cardinality_is_default(cardinality: &Cardinality) -> bool {
    cardinality == &Cardinality::ONE
}

impl<T> Choice<T> {
    /// Panics on duplicate labels: an authoring-time error, like the `unwrap()`s on content refs.
    pub fn new(
        cardinality: Cardinality,
        prompt: impl Into<String>,
        options: impl IntoIterator<Item = ChoiceOption<T>>,
    ) -> Self {
        let options: Vec<ChoiceOption<T>> = options.into_iter().collect();
        Self {
            cardinality,
            prompt: prompt.into(),
            options,
            selected: Vec::new(),
        }
    }

    pub fn one(prompt: impl Into<String>, options: impl IntoIterator<Item = ChoiceOption<T>>) -> Self {
        Self::new(Cardinality::ONE, prompt, options)
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn options(&self) -> &[ChoiceOption<T>] {
        &self.options
    }

    pub fn selected(&self) -> &[String] {
        &self.selected
    }

    pub fn is_label_selected(&self, label: &str) -> bool {
        self.selected.iter().any(|s| s == label)
    }

    pub(crate) fn select_label(&mut self, option: usize) {
        let label = &self.options[option].label;
        if !self.selected.contains(label) {
            self.selected.push(label.clone());
        }
    }

    pub(crate) fn unselect_label(&mut self, option: usize) {
        let label = &self.options[option].label;
        self.selected.retain(|s| s != label);
    }

    pub fn is_ok(&self) -> bool {
        self.selected.iter().all(|label| self.options.iter().any(|o| &o.label == label))
            && self.cardinality.accepts(self.selected.len())
    }

    /// Values of the selected options, in option (not selection) order.
    pub fn selected_values(&self) -> impl Iterator<Item = &T> {
        let selected = &self.selected;
        self.options
            .iter()
            .filter(move |o| selected.contains(&o.label))
            .map(|o| &o.value)
    }

    pub fn selected_values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let selected = &self.selected;
        self.options
            .iter_mut()
            .filter(move |o| selected.contains(&o.label))
            .map(|o| &mut o.value)
    }

    /// Consume the choice, yielding the picked values (flattened).
    pub fn make_one(self) -> Result<T, ChoiceError> {
        let Self { cardinality, prompt, options, selected } = self;
        if let Some(label) = selected.iter().find(|label| !options.iter().any(|o| &o.label == *label)) {
            return Err(ChoiceError::UnknownOption { prompt, label: label.clone() });
        }
        if !cardinality.accepts(selected.len()) {
            return Err(ChoiceError::BadCardinality { prompt, expected: cardinality, got: selected.len() });
        }
        Ok(options
            .into_iter()
            .filter(|o| selected.contains(&o.label))
            .map(|o| o.value)
            .next()
            .unwrap())
    }

    pub fn make_many(self) -> Result<Vec<T>, ChoiceError> {
        let Self { cardinality, prompt, options, selected } = self;
        if let Some(label) = selected.iter().find(|label| !options.iter().any(|o| &o.label == *label)) {
            return Err(ChoiceError::UnknownOption { prompt, label: label.clone() });
        }
        if !cardinality.accepts(selected.len()) {
            return Err(ChoiceError::BadCardinality { prompt, expected: cardinality, got: selected.len() });
        }
        Ok(options
            .into_iter()
            .filter(|o| selected.contains(&o.label))
            .map(|o| o.value)
            .collect())
    }
}
