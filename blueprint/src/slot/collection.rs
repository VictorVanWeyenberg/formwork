use crate::cardinality::Cardinality;
use crate::choice::ChoiceCollection;
use crate::error::BlueprintError;
use serde::{Deserialize, Serialize};

/// A collection slot where a choice yields multiple values. Resolves to `Vec<T>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotCollection<T> {
    Fixed(Vec<T>),
    Choose(ChoiceCollection<T>),
}

impl<T> SlotCollection<T> {
    pub fn fixed(values: Vec<T>) -> Self {
        SlotCollection::Fixed(values)
    }

    pub fn choose(choice: ChoiceCollection<T>) -> Self {
        SlotCollection::Choose(choice)
    }

    /// Alias for backwards compatibility.
    pub fn pending(choice: ChoiceCollection<T>) -> Self {
        SlotCollection::Choose(choice)
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self, SlotCollection::Fixed(_))
    }

    pub fn is_pending(&self) -> bool {
        match self {
            SlotCollection::Fixed(_) => false,
            SlotCollection::Choose(c) => !c.is_made(),
        }
    }

    pub fn is_made(&self) -> bool {
        match self {
            SlotCollection::Fixed(_) => false,
            SlotCollection::Choose(c) => c.is_made(),
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            SlotCollection::Fixed(_) => true,
            SlotCollection::Choose(c) => {
                c.is_made() && c.cardinality().accepts(c.selected_indices().len())
            }
        }
    }

    pub fn as_fixed(&self) -> Option<&[T]> {
        match self {
            SlotCollection::Fixed(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    pub fn as_fixed_mut(&mut self) -> Option<&mut [T]> {
        match self {
            SlotCollection::Fixed(v) => Some(v.as_mut_slice()),
            _ => None,
        }
    }

    pub fn as_choice(&self) -> Option<&ChoiceCollection<T>> {
        match self {
            SlotCollection::Choose(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_choice_mut(&mut self) -> Option<&mut ChoiceCollection<T>> {
        match self {
            SlotCollection::Choose(c) => Some(c),
            _ => None,
        }
    }

    pub fn selected_values(&self) -> Option<Vec<&T>> {
        match self {
            SlotCollection::Fixed(v) => Some(v.iter().collect()),
            SlotCollection::Choose(c) => c.selected_values(),
        }
    }

    pub fn selected_values_mut(&mut self) -> Option<Vec<&mut T>> {
        match self {
            SlotCollection::Fixed(v) => Some(v.iter_mut().collect()),
            SlotCollection::Choose(c) => c.selected_values_mut(),
        }
    }

    pub fn prompt(&self) -> &str {
        match self {
            SlotCollection::Fixed(_) => "",
            SlotCollection::Choose(c) => c.prompt(),
        }
    }

    pub fn cardinality(&self) -> Cardinality {
        match self {
            SlotCollection::Fixed(_) => Cardinality::ONE,
            SlotCollection::Choose(c) => c.cardinality(),
        }
    }

    pub fn option_labels(&self) -> Vec<&str> {
        match self {
            SlotCollection::Fixed(_) => vec![],
            SlotCollection::Choose(c) => c.option_labels(),
        }
    }

    pub fn selected_indices(&self) -> &[usize] {
        match self {
            SlotCollection::Fixed(_) => &[],
            SlotCollection::Choose(c) => c.selected_indices(),
        }
    }


    pub fn select(&mut self, indices: &[usize]) -> Result<(), BlueprintError> {
        match self {
            SlotCollection::Choose(c) => c.select(indices),
            SlotCollection::Fixed(_) => {
                Err(BlueprintError::Custom("cannot select on a fixed slot".into()))
            }
        }
    }

    pub fn unmake(&mut self) -> Result<(), BlueprintError> {
        match self {
            SlotCollection::Choose(c) => c.unmake(),
            SlotCollection::Fixed(_) => {
                Err(BlueprintError::Custom("cannot unmake a fixed slot".into()))
            }
        }
    }

    pub fn confirm(&mut self) -> Result<(), BlueprintError> {
        match self {
            SlotCollection::Choose(c) => c.confirm(),
            SlotCollection::Fixed(_) => Ok(()),
        }
    }

    /// Resolves the collection slot to its final `Vec<T>`.
    pub fn resolve(self) -> Result<Vec<T>, BlueprintError> {
        match self {
            SlotCollection::Fixed(values) => Ok(values),
            SlotCollection::Choose(choice) => choice.resolve(),
        }
    }
}

impl<T> From<Vec<T>> for SlotCollection<T> {
    fn from(values: Vec<T>) -> Self {
        SlotCollection::Fixed(values)
    }
}

impl<T> From<ChoiceCollection<T>> for SlotCollection<T> {
    fn from(choice: ChoiceCollection<T>) -> Self {
        SlotCollection::Choose(choice)
    }
}

impl<T> Default for SlotCollection<T> {
    fn default() -> Self {
        SlotCollection::Fixed(Vec::new())
    }
}
