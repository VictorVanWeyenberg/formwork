
use crate::error::BlueprintError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cardinality {
    UpTo(usize),
    AtLeast(usize),
    #[serde(untagged)]
    Exact(usize),
}

impl std::fmt::Display for Cardinality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cardinality::Exact(n) => write!(f, "exactly {}", n),
            Cardinality::UpTo(n) => write!(f, "up to {}", n),
            Cardinality::AtLeast(n) => write!(f, "at least {}", n),
            
        }
    }
}

impl Cardinality {
    pub const ONE: Cardinality = Cardinality::Exact(1);

    pub fn accepts(&self, count: usize) -> bool {
        match self {
            Cardinality::Exact(n) => count == *n,
            Cardinality::UpTo(n) => count <= *n,
            Cardinality::AtLeast(n) => count >= *n,
            
        }
    }

    pub fn max(&self) -> usize {
        match self {
            Cardinality::Exact(n) => *n,
            Cardinality::UpTo(n) => *n,
            Cardinality::AtLeast(_) => usize::MAX,
            
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChoiceState {
    Pending,
    UnmadeWithHistory(Vec<usize>),
    Made(Vec<usize>),
}

impl Default for ChoiceState {
    fn default() -> Self {
        ChoiceState::Pending
    }
}

fn default_cardinality() -> Cardinality {
    Cardinality::Exact(1)
}

fn is_default_cardinality(c: &Cardinality) -> bool {
    *c == Cardinality::Exact(1)
}

fn is_default_state(s: &ChoiceState) -> bool {
    *s == ChoiceState::Pending
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice<T> {
    pub prompt: String,
    pub options: Vec<T>,
    #[serde(rename = "choose", default = "default_cardinality", skip_serializing_if = "is_default_cardinality")]
    pub cardinality: Cardinality,
    #[serde(default, skip_serializing_if = "is_default_state")]
    pub state: ChoiceState,
}

impl<T> Choice<T> {
    pub fn new(
        prompt: impl Into<String>,
        options: impl IntoIterator<Item = T>,
    ) -> Self {
        Self {
            prompt: prompt.into(),
            options: options.into_iter().collect(),
            cardinality: Cardinality::Exact(1),
            state: ChoiceState::Pending,
        }
    }

    pub fn with_cardinality(mut self, cardinality: Cardinality) -> Self {
        self.cardinality = cardinality;
        self
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn options(&self) -> &[T] {
        &self.options
    }

    pub fn is_made(&self) -> bool {
        matches!(self.state, ChoiceState::Made(_))
    }

    pub fn selected_indices(&self) -> &[usize] {
        match &self.state {
            ChoiceState::Made(idx) => idx.as_slice(),
            ChoiceState::UnmadeWithHistory(idx) => idx.as_slice(),
            ChoiceState::Pending => &[],
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected_indices().first().copied()
    }

    pub fn selected_values(&self) -> Vec<&T> {
        let indices = self.selected_indices();
        self.options
            .iter()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, opt)| opt)
            .collect()
    }

    pub fn selected_values_mut(&mut self) -> Vec<&mut T> {
        let indices = match &self.state {
            ChoiceState::Made(idx) => idx.clone(),
            ChoiceState::UnmadeWithHistory(idx) => idx.clone(),
            ChoiceState::Pending => vec![],
        };
        self.options
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| indices.contains(i))
            .map(|(_, opt)| opt)
            .collect()
    }

    pub fn select(&mut self, index: usize) -> Result<(), BlueprintError> {
        if index >= self.options.len() {
            return Err(BlueprintError::IndexOutOfBounds {
                prompt: self.prompt.clone(),
                index,
                total_options: self.options.len(),
            });
        }
        
        let mut indices = match &self.state {
            ChoiceState::Pending => vec![],
            ChoiceState::UnmadeWithHistory(history) => {
                if self.cardinality.max() == 1 {
                    vec![]
                } else {
                    history.clone()
                }
            },
            ChoiceState::Made(_) => return Err(BlueprintError::Custom("Choice already made".into())),
        };

        if !indices.contains(&index) {
            indices.push(index);
        } else {
            return Err(BlueprintError::DuplicateSelection {
                prompt: self.prompt.clone(),
                index,
            });
        }

        if indices.len() == self.cardinality.max() {
            self.state = ChoiceState::Made(indices);
        } else {
            self.state = ChoiceState::UnmadeWithHistory(indices);
        }
        
        Ok(())
    }

    pub fn unmake(&mut self) -> Result<(), BlueprintError> {
        match &self.state {
            ChoiceState::Made(indices) => {
                let mut new_history = indices.clone();
                if self.cardinality.max() > 1 {
                    new_history.pop();
                }
                if new_history.is_empty() {
                    self.state = ChoiceState::Pending;
                } else {
                    self.state = ChoiceState::UnmadeWithHistory(new_history);
                }
                Ok(())
            }
            ChoiceState::UnmadeWithHistory(indices) => {
                let mut new_history = indices.clone();
                if self.cardinality.max() > 1 {
                    new_history.pop();
                }
                if new_history.is_empty() {
                    self.state = ChoiceState::Pending;
                } else {
                    self.state = ChoiceState::UnmadeWithHistory(new_history);
                }
                Ok(())
            }
            ChoiceState::Pending => Err(BlueprintError::CannotUnmakeNotMade {
                prompt: self.prompt.clone(),
            }),
        }
    }

    pub fn confirm(&mut self) -> Result<(), BlueprintError> {
        if let ChoiceState::Made(_) = self.state {
            return Ok(());
        }
        
        let indices = match &self.state {
            ChoiceState::Pending => vec![],
            ChoiceState::UnmadeWithHistory(idx) => idx.clone(),
            ChoiceState::Made(idx) => idx.clone(),
        };

        if self.cardinality.accepts(indices.len()) {
            self.state = ChoiceState::Made(indices);
            Ok(())
        } else {
            Err(BlueprintError::InvalidCardinality {
                prompt: self.prompt.clone(),
                expected: self.cardinality,
                actual: indices.len(),
            })
        }
    }

    pub fn resolve_exact(self) -> Result<T, BlueprintError> {
        let prompt = self.prompt.clone();
        let mut results = self.resolve()?;
        if results.len() == 1 {
            Ok(results.pop().unwrap())
        } else {
            Err(BlueprintError::Custom(format!(
                "Expected exactly one resolution for choice '{}', got {}",
                prompt,
                results.len()
            )))
        }
    }

    pub fn resolve(mut self) -> Result<Vec<T>, BlueprintError> {
        if !self.is_made() {
            return Err(BlueprintError::UnresolvedChoice {
                prompt: self.prompt,
            });
        }
        
        let indices = match self.state {
            ChoiceState::Made(idx) => idx,
            _ => unreachable!(),
        };
        
        let mut sorted_indices = indices.clone();
        sorted_indices.sort_unstable();
        sorted_indices.dedup();
        
        let mut result = Vec::new();
        // Remove from the end to avoid shifting indices
        for idx in sorted_indices.into_iter().rev() {
            if idx < self.options.len() {
                result.push(self.options.remove(idx));
            }
        }
        result.reverse();
        
        Ok(result)
    }
}
