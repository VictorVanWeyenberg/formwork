use super::option::ChoiceOption;
use crate::error::BlueprintError;
use serde::{Deserialize, Serialize};

/// A choice for a single-value slot. Resolves to exactly one `T`.
///
/// Because a scalar slot can only ever resolve to a single value, this type has
/// NO cardinality field. It is impossible by construction to configure a scalar
/// choice with a cardinality other than 1.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice<T> {
    pub prompt: String,
    pub options: Vec<ChoiceOption<T>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_index: Option<usize>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_made: bool,
}

impl<T> Choice<T> {
    pub fn new(
        prompt: impl Into<String>,
        options: impl IntoIterator<Item = ChoiceOption<T>>,
    ) -> Self {
        Self {
            prompt: prompt.into(),
            options: options.into_iter().collect(),
            selected_index: None,
            is_made: false,
        }
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
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

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    pub fn selected_indices(&self) -> &[usize] {
        match &self.selected_index {
            Some(idx) => std::slice::from_ref(idx),
            None => &[],
        }
    }


    pub fn selected_value(&self) -> Option<&T> {
        self.selected_index
            .and_then(|idx| self.options.get(idx))
            .map(|opt| &opt.value)
    }

    pub fn selected_value_mut(&mut self) -> Option<&mut T> {
        let idx = self.selected_index?;
        self.options.get_mut(idx).map(|opt| &mut opt.value)
    }

    pub fn select(&mut self, index: usize) -> Result<(), BlueprintError> {
        if index >= self.options.len() {
            return Err(BlueprintError::IndexOutOfBounds {
                prompt: self.prompt.clone(),
                index,
                total_options: self.options.len(),
            });
        }
        self.selected_index = Some(index);
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
        if let Some(idx) = self.selected_index {
            if idx >= self.options.len() {
                return Err(BlueprintError::IndexOutOfBounds {
                    prompt: self.prompt.clone(),
                    index: idx,
                    total_options: self.options.len(),
                });
            }
            self.selected_index = Some(idx);
            self.is_made = true;
            Ok(())
        } else {
            Err(BlueprintError::UnresolvedChoice {
                prompt: self.prompt.clone(),
            })
        }
    }

    pub fn resolve(mut self) -> Result<T, BlueprintError> {
        if !self.is_made {
            return Err(BlueprintError::UnresolvedChoice {
                prompt: self.prompt,
            });
        }
        let index = self
            .selected_index
            .ok_or_else(|| BlueprintError::UnresolvedChoice {
                prompt: self.prompt.clone(),
            })?;
        if index >= self.options.len() {
            return Err(BlueprintError::IndexOutOfBounds {
                prompt: self.prompt,
                index,
                total_options: self.options.len(),
            });
        }
        Ok(self.options.remove(index).value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choice_serde_omits_defaults() {
        let choice = Choice::new(
            "Pick a number",
            vec![ChoiceOption::new("One", 1), ChoiceOption::new("Two", 2)],
        );

        let json = serde_json::to_string(&choice).unwrap();
        assert!(!json.contains("selected_index"));
        assert!(!json.contains("is_made"));

        let deserialized: Choice<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.selected_index, None);
        assert!(!deserialized.is_made);
    }

    #[test]
    fn test_choice_serde_includes_when_selected() {
        let mut choice = Choice::new(
            "Pick a number",
            vec![ChoiceOption::new("One", 1), ChoiceOption::new("Two", 2)],
        );
        choice.select(1).unwrap();

        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains("\"selected_index\":1"));
        assert!(json.contains("\"is_made\":true"));

        let deserialized: Choice<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.selected_index, Some(1));
        assert!(deserialized.is_made);
    }

    #[test]
    fn test_choice_unmake_and_confirm() {
        let mut choice = Choice::new(
            "Pick",
            vec![ChoiceOption::new("A", 10), ChoiceOption::new("B", 20)],
        );
        choice.select(0).unwrap();
        assert!(choice.is_made());
        assert_eq!(choice.selected_index(), Some(0));

        choice.unmake().unwrap();
        assert!(!choice.is_made());
        assert_eq!(choice.selected_index(), Some(0));

        choice.confirm().unwrap();
        assert!(choice.is_made());
        assert_eq!(choice.selected_index(), Some(0));
    }
}
