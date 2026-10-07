use super::option::ChoiceOption;
use crate::cardinality::Cardinality;
use crate::error::BlueprintError;
use serde::{Deserialize, Serialize};

/// A choice for a collection slot. Resolves to `Vec<T>`.
///
/// Here, because the slot holds a collection, picking multiple items (M of N)
/// is supported and governed by the `cardinality` field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceCollection<T> {
    pub prompt: String,
    pub cardinality: Cardinality,
    pub options: Vec<ChoiceOption<T>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_indices: Vec<usize>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_made: bool,
}

impl<T> ChoiceCollection<T> {
    pub fn new(
        prompt: impl Into<String>,
        cardinality: Cardinality,
        options: impl IntoIterator<Item = ChoiceOption<T>>,
    ) -> Self {
        Self {
            prompt: prompt.into(),
            cardinality,
            options: options.into_iter().collect(),
            selected_indices: Vec::new(),
            is_made: false,
        }
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn cardinality(&self) -> Cardinality {
        self.cardinality
    }

    pub fn options(&self) -> &[ChoiceOption<T>] {
        &self.options
    }

    pub fn option_labels(&self) -> Vec<&str> {
        self.options.iter().map(|opt| opt.label.as_str()).collect()
    }

    pub fn is_made(&self) -> bool {
        self.is_made
    }

    pub fn selected_indices(&self) -> &[usize] {
        &self.selected_indices
    }

    pub fn selected_values(&self) -> Option<Vec<&T>> {
        if !self.is_made && self.selected_indices.is_empty() {
            None
        } else {
            let vals: Vec<&T> = self
                .selected_indices
                .iter()
                .filter_map(|&i| self.options.get(i).map(|opt| &opt.value))
                .collect();
            Some(vals)
        }
    }

    pub fn selected_values_mut(&mut self) -> Option<Vec<&mut T>> {
        if !self.is_made && self.selected_indices.is_empty() {
            None
        } else {
            let mut vals = Vec::new();
            for &idx in self.selected_indices.iter() {
                if idx < self.options.len() {
                    let opt_ptr = &mut self.options[idx].value as *mut T;
                    // Safe because indices are unique
                    unsafe {
                        vals.push(&mut *opt_ptr);
                    }
                }
            }
            Some(vals)
        }
    }

    pub fn select(&mut self, indices: &[usize]) -> Result<(), BlueprintError> {
        if !self.cardinality.accepts(indices.len()) {
            return Err(BlueprintError::InvalidCardinality {
                prompt: self.prompt.clone(),
                expected: self.cardinality,
                actual: indices.len(),
            });
        }

        // Validate bounds and duplicates
        for (pos, &idx) in indices.iter().enumerate() {
            if idx >= self.options.len() {
                return Err(BlueprintError::IndexOutOfBounds {
                    prompt: self.prompt.clone(),
                    index: idx,
                    total_options: self.options.len(),
                });
            }
            if indices[..pos].contains(&idx) {
                return Err(BlueprintError::DuplicateSelection {
                    prompt: self.prompt.clone(),
                    index: idx,
                });
            }
        }

        self.selected_indices = indices.to_vec();
        self.is_made = true;
        Ok(())
    }

    pub fn unmake(&mut self) -> Result<(), BlueprintError> {
        if !self.is_made {
            return Err(BlueprintError::CannotUnmakeNotMade {
                prompt: self.prompt.clone(),
            });
        }
        self.is_made = false;
        Ok(())
    }

    pub fn confirm(&mut self) -> Result<(), BlueprintError> {
        if self.is_made {
            return Ok(());
        }
        let indices = if !self.selected_indices.is_empty() {
            self.selected_indices.clone()
        } else {
            Vec::new()
        };

        if !self.cardinality.accepts(indices.len()) {
            return Err(BlueprintError::InvalidCardinality {
                prompt: self.prompt.clone(),
                expected: self.cardinality,
                actual: indices.len(),
            });
        }

        for (pos, &idx) in indices.iter().enumerate() {
            if idx >= self.options.len() {
                return Err(BlueprintError::IndexOutOfBounds {
                    prompt: self.prompt.clone(),
                    index: idx,
                    total_options: self.options.len(),
                });
            }
            if indices[..pos].contains(&idx) {
                return Err(BlueprintError::DuplicateSelection {
                    prompt: self.prompt.clone(),
                    index: idx,
                });
            }
        }

        self.selected_indices = indices.clone();
        self.is_made = true;
        Ok(())
    }

    pub fn resolve(self) -> Result<Vec<T>, BlueprintError> {
        if !self.is_made {
            return Err(BlueprintError::UnresolvedChoice {
                prompt: self.prompt,
            });
        }
        if !self.cardinality.accepts(self.selected_indices.len()) {
            return Err(BlueprintError::InvalidCardinality {
                prompt: self.prompt,
                expected: self.cardinality,
                actual: self.selected_indices.len(),
            });
        }
        for &idx in &self.selected_indices {
            if idx >= self.options.len() {
                return Err(BlueprintError::IndexOutOfBounds {
                    prompt: self.prompt,
                    index: idx,
                    total_options: self.options.len(),
                });
            }
        }
        let mut indexed_options: Vec<(usize, Option<T>)> = self
            .options
            .into_iter()
            .map(|o| Some(o.value))
            .enumerate()
            .collect();

        let mut out = Vec::with_capacity(self.selected_indices.len());
        for idx in self.selected_indices {
            if let Some((_, val_opt)) = indexed_options.get_mut(idx) {
                if let Some(val) = val_opt.take() {
                    out.push(val);
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice_collection_serde_omits_defaults() {
        let choice = ChoiceCollection::new(
            "Pick skills",
            Cardinality::AtLeast(1),
            vec![ChoiceOption::new("Athletics", "Athletics".to_string())],
        );

        let json = serde_json::to_string(&choice).unwrap();
        assert!(!json.contains("selected_indices"));
        assert!(!json.contains("is_made"));

        let deserialized: ChoiceCollection<String> = serde_json::from_str(&json).unwrap();
        assert!(deserialized.selected_indices.is_empty());
        assert!(!deserialized.is_made);
    }

    #[test]
    fn test_choice_collection_serde_includes_when_selected() {
        let mut choice = ChoiceCollection::new(
            "Pick skills",
            Cardinality::AtLeast(1),
            vec![
                ChoiceOption::new("Athletics", "Athletics".to_string()),
                ChoiceOption::new("Stealth", "Stealth".to_string()),
            ],
        );
        choice.select(&[1]).unwrap();

        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains("\"selected_indices\":[1]"));
        assert!(json.contains("\"is_made\":true"));

        let deserialized: ChoiceCollection<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.selected_indices, vec![1]);
        assert!(deserialized.is_made);
    }
}
