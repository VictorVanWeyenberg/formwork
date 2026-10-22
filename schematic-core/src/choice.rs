//! Choice abstractions and state machines for Higher-Kinded Data blueprints.
//!
//! This module provides the primary choice containers ([`SingleChoice`] and [`MultiChoice`]),
//! cardinality constraints ([`Cardinality`]), and lifecycle tracking ([`ChoiceState`]).
//!
//! # Universal Calling Contract: "Select -> Confirm -> Resolve"
//!
//! Across all slot kinds and cardinality constraints, choices follow a single unified contract:
//! 1. **Select**: Call [`SingleChoice::select`] or [`MultiChoice::select`] to stage
//!    candidate options. Calling `select` records the choice but **never** transitions the state to
//!    [`ChoiceState::Made`].
//! 2. **Confirm**: Call [`SingleChoice::confirm`] or [`MultiChoice::confirm`] to
//!    validate cardinality constraints and transition the choice to [`ChoiceState::Made`].
//! 3. **Resolve**: Only choices in the [`ChoiceState::Made`] state can be resolved into concrete domain values.
//!    Attempting to resolve an unconfirmed choice returns [`SchematicError::UnresolvedChoice`].
//!
//! # Example
//!
//! ```rust
//! use schematic_core::choice::{SingleChoice, ChoiceState};
//!
//! let mut choice = SingleChoice::new("Select Class", vec!["Fighter", "Wizard", "Rogue"]);
//!
//! // 1. Selection stages the candidate (does NOT make the choice)
//! choice.select(0).unwrap();
//! assert!(!choice.is_made());
//!
//! // 2. Confirmation transitions to Made
//! choice.confirm().unwrap();
//! assert!(choice.is_made());
//!
//! // 3. Resolve into watertight value
//! let resolved = choice.resolve_exact().unwrap();
//! assert_eq!(resolved, "Fighter");
//! ```

use crate::error::SchematicError;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Specifies selection bounds and constraints for a [`MultiChoice`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Cardinality {
    /// Up to `n` options may be selected ($0 \le k \le n$).
    UpTo(usize),
    /// At least `n` options must be selected ($k \ge n$).
    AtLeast(usize),
    /// Exactly `n` options must be selected ($k = n$).
    Exact(usize),
}

impl Cardinality {
    /// Cardinality requiring exactly one selection (`Exact(1)`).
    pub const ONE: Self = Self::Exact(1);

    /// Checks whether a given selection count satisfies this cardinality constraint.
    ///
    /// # Arguments
    /// * `count` - The number of selected options.
    ///
    /// # Returns
    /// `true` if `count` satisfies the cardinality rule, `false` otherwise.
    ///
    /// # Examples
    /// ```rust
    /// use schematic_core::choice::Cardinality;
    ///
    /// assert!(Cardinality::Exact(2).accepts(2));
    /// assert!(!Cardinality::Exact(2).accepts(1));
    /// assert!(Cardinality::UpTo(3).accepts(2));
    /// assert!(!Cardinality::UpTo(3).accepts(4));
    /// assert!(Cardinality::AtLeast(1).accepts(5));
    /// assert!(!Cardinality::AtLeast(1).accepts(0));
    /// ```
    pub fn accepts(&self, count: usize) -> bool {
        match self {
            Self::UpTo(n) => count <= *n,
            Self::AtLeast(n) => count >= *n,
            Self::Exact(n) => count == *n,
        }
    }

    /// Returns the maximum number of selections permitted by this cardinality rule.
    ///
    /// Returns `usize::MAX` for open-ended constraints such as [`Cardinality::AtLeast`].
    ///
    /// # Returns
    /// Upper selection limit.
    pub fn max(&self) -> usize {
        match self {
            Self::UpTo(n) | Self::Exact(n) => *n,
            Self::AtLeast(_) => usize::MAX,
        }
    }

    /// Returns the minimum number of selections required by this cardinality rule.
    ///
    /// Returns `0` for optional constraints such as [`Cardinality::UpTo`].
    ///
    /// # Returns
    /// Lower selection threshold.
    pub fn min(&self) -> usize {
        match self {
            Self::UpTo(_) => 0,
            Self::AtLeast(n) | Self::Exact(n) => *n,
        }
    }
}

impl Display for Cardinality {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UpTo(n) => write!(f, "up to {n}"),
            Self::AtLeast(n) => write!(f, "at least {n}"),
            Self::Exact(n) => write!(f, "exactly {n}"),
        }
    }
}

/// Internal lifecycle state tracking for a choice.
///
/// Stores staged selections and previous history generically via `H`:
/// - [`SingleChoice`] instantiates `ChoiceState<usize>` (stack-only scalar, zero heap allocations).
/// - [`MultiChoice`] instantiates `ChoiceState<Vec<usize>>` (dynamic collection of selected indices).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum ChoiceState<H> {
    /// Choice is open; no selections have been staged or made.
    #[default]
    Pending,
    /// Selections are staged (in-progress) or an existing made choice was unmade, preserving selection history.
    UnmadeWithHistory(H),
    /// Choice has been confirmed as made with selection history `H`.
    Made(H),
}

impl<H> ChoiceState<H> {
    /// Returns `true` if the choice is in the [`ChoiceState::Pending`] state.
    pub fn is_pending(&self) -> bool {
        matches!(self, ChoiceState::Pending)
    }

    /// Returns `true` if the choice is in the [`ChoiceState::Made`] state.
    pub fn is_made(&self) -> bool {
        matches!(self, ChoiceState::Made(_))
    }

    /// Returns `true` if the choice is in the [`ChoiceState::UnmadeWithHistory`] state.
    pub fn is_unmade(&self) -> bool {
        matches!(self, ChoiceState::UnmadeWithHistory(_))
    }

    /// Returns a reference to the inner selection data if staged, unmade, or made.
    pub fn selection_data(&self) -> Option<&H> {
        match self {
            ChoiceState::Made(h) | ChoiceState::UnmadeWithHistory(h) => Some(h),
            ChoiceState::Pending => None,
        }
    }
}

fn is_default_state<H>(s: &ChoiceState<H>) -> bool {
    s.is_pending()
}

/// A user-facing choice where exactly one option must be selected and confirmed.
///
/// Operates on stack-only `ChoiceState<usize>` with zero heap allocation overhead.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SingleChoice<T> {
    /// The user-facing question or prompt describing this choice.
    pub prompt: String,
    /// The candidate options available to pick from.
    pub options: Vec<T>,
    /// The lifecycle state tracking staged or confirmed selections.
    #[serde(default, skip_serializing_if = "is_default_state")]
    pub state: ChoiceState<usize>,
}

impl<T> SingleChoice<T> {
    /// Creates a new [`SingleChoice`] in the [`ChoiceState::Pending`] state.
    ///
    /// # Arguments
    /// * `prompt` - A descriptive label or question for this choice.
    /// * `options` - An iterable sequence of candidate options.
    ///
    /// # Examples
    /// ```rust
    /// use schematic_core::choice::SingleChoice;
    ///
    /// let choice = SingleChoice::new("Pick weapon", vec!["Sword", "Bow"]);
    /// assert_eq!(choice.prompt(), "Pick weapon");
    /// assert_eq!(choice.options(), &["Sword", "Bow"]);
    /// ```
    pub fn new(prompt: impl Into<String>, options: impl IntoIterator<Item = T>) -> Self {
        Self {
            prompt: prompt.into(),
            options: options.into_iter().collect(),
            state: ChoiceState::Pending,
        }
    }

    /// Returns a reference to the choice prompt.
    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    /// Returns a slice of all available candidate options.
    pub fn options(&self) -> &[T] {
        &self.options
    }

    /// Returns a mutable slice of all available candidate options.
    pub fn options_mut(&mut self) -> &mut [T] {
        &mut self.options
    }

    /// Returns a reference to the underlying [`ChoiceState`].
    pub fn state(&self) -> &ChoiceState<usize> {
        &self.state
    }

    /// Returns `true` if the choice has been confirmed into [`ChoiceState::Made`].
    pub fn is_made(&self) -> bool {
        self.state.is_made()
    }

    /// Returns the currently staged or confirmed option index, if any.
    pub fn selected_index(&self) -> Option<usize> {
        self.state.selection_data().copied()
    }

    /// Returns a slice containing the currently selected index, or empty if pending.
    pub fn selected_indices(&self) -> &[usize] {
        match &self.state {
            ChoiceState::Made(idx) | ChoiceState::UnmadeWithHistory(idx) => {
                std::slice::from_ref(idx)
            }
            ChoiceState::Pending => &[],
        }
    }

    /// Returns a reference to the selected option, if one is staged or confirmed.
    pub fn selected_value(&self) -> Option<&T> {
        self.selected_index().and_then(|i| self.options.get(i))
    }

    /// Returns a vector of references to selected options.
    pub fn selected_values(&self) -> Vec<&T> {
        self.selected_value().into_iter().collect()
    }

    /// Returns a mutable reference to the selected option, if one is staged or confirmed.
    pub fn selected_value_mut(&mut self) -> Option<&mut T> {
        match self.selected_index() {
            Some(i) if i < self.options.len() => Some(&mut self.options[i]),
            _ => None,
        }
    }

    /// Returns a vector of mutable references to selected options.
    pub fn selected_values_mut(&mut self) -> Vec<&mut T> {
        match self.selected_index() {
            Some(i) if i < self.options.len() => vec![&mut self.options[i]],
            _ => vec![],
        }
    }

    /// Stages a candidate option at the specified `index`.
    ///
    /// In accordance with the universal choice contract, calling `select` stages the option
    /// into [`ChoiceState::UnmadeWithHistory`] and does **NOT** transition to [`ChoiceState::Made`].
    /// To confirm the choice, call [`confirm`](Self::confirm).
    ///
    /// Subsequent calls to `select` replace the staged index in-place without erroring.
    ///
    /// # Arguments
    /// * `index` - The zero-based index of the option to stage.
    ///
    /// # Errors
    /// Returns [`SchematicError::IndexOutOfBounds`] if `index >= options.len()`.
    ///
    /// # Examples
    /// ```rust
    /// use schematic_core::choice::SingleChoice;
    ///
    /// let mut choice = SingleChoice::new("Color", vec!["Red", "Blue"]);
    /// choice.select(1).unwrap();
    /// assert_eq!(choice.selected_index(), Some(1));
    /// assert!(!choice.is_made()); // Staged, not Made!
    /// ```
    pub fn select(&mut self, index: usize) -> Result<(), SchematicError> {
        if index >= self.options.len() {
            return Err(SchematicError::IndexOutOfBounds {
                prompt: self.prompt.clone(),
                index,
                total_options: self.options.len(),
            });
        }
        self.state = ChoiceState::UnmadeWithHistory(index);
        Ok(())
    }

    /// Confirms the staged selection, transitioning state to [`ChoiceState::Made`].
    ///
    /// If the choice is already confirmed, this call is an idempotent no-op returning `Ok(())`.
    ///
    /// # Errors
    /// Returns [`SchematicError::InvalidCardinality`] if called while in [`ChoiceState::Pending`].
    ///
    /// # Examples
    /// ```rust
    /// use schematic_core::choice::SingleChoice;
    ///
    /// let mut choice = SingleChoice::new("Color", vec!["Red", "Blue"]);
    /// choice.select(0).unwrap();
    /// choice.confirm().unwrap();
    /// assert!(choice.is_made());
    /// ```
    pub fn confirm(&mut self) -> Result<(), SchematicError> {
        match self.state {
            ChoiceState::Made(_) => Ok(()),
            ChoiceState::UnmadeWithHistory(idx) => {
                self.state = ChoiceState::Made(idx);
                Ok(())
            }
            ChoiceState::Pending => Err(SchematicError::InvalidCardinality {
                prompt: self.prompt.clone(),
                expected: Cardinality::Exact(1),
                actual: 0,
            }),
        }
    }

    /// Deselects the option at the given `index`, reverting the state to [`ChoiceState::Pending`].
    ///
    /// If the choice was previously [`ChoiceState::Made`], deselecting the option demotes it
    /// back to unconfirmed.
    ///
    /// # Arguments
    /// * `index` - The zero-based index of the option to unselect.
    ///
    /// # Errors
    /// Returns [`SchematicError::NotSelected`] if `index` was not currently staged or made.
    pub fn deselect(&mut self, index: usize) -> Result<(), SchematicError> {
        match self.state {
            ChoiceState::Made(curr) | ChoiceState::UnmadeWithHistory(curr) if curr == index => {
                self.state = ChoiceState::Pending;
                Ok(())
            }
            _ => Err(SchematicError::NotSelected {
                prompt: self.prompt.clone(),
                index,
            }),
        }
    }

    /// Alias for [`deselect`](Self::deselect).
    pub fn unselect(&mut self, index: usize) -> Result<(), SchematicError> {
        self.deselect(index)
    }

    /// Losslessly un-makes a confirmed choice, preserving the selection in [`ChoiceState::UnmadeWithHistory`].
    ///
    /// Calling [`confirm`](Self::confirm) afterwards restores the choice to [`ChoiceState::Made`].
    ///
    /// # Errors
    /// Returns [`SchematicError::CannotUnmakeNotMade`] if the choice is in [`ChoiceState::Pending`].
    pub fn unmake(&mut self) -> Result<(), SchematicError> {
        match self.state {
            ChoiceState::Made(idx) => {
                self.state = ChoiceState::UnmadeWithHistory(idx);
                Ok(())
            }
            ChoiceState::UnmadeWithHistory(_) => Ok(()),
            ChoiceState::Pending => Err(SchematicError::CannotUnmakeNotMade {
                prompt: self.prompt.clone(),
            }),
        }
    }

    /// Resolves the choice into its selected domain value.
    ///
    /// Consumes `self`.
    ///
    /// # Errors
    /// Returns [`SchematicError::UnresolvedChoice`] if the choice has not been confirmed into [`ChoiceState::Made`].
    pub fn resolve_exact(mut self) -> Result<T, SchematicError> {
        if !self.is_made() {
            return Err(SchematicError::UnresolvedChoice {
                prompt: self.prompt,
            });
        }
        let idx = match self.state {
            ChoiceState::Made(i) => i,
            _ => unreachable!(),
        };
        if idx >= self.options.len() {
            return Err(SchematicError::IndexOutOfBounds {
                prompt: self.prompt,
                index: idx,
                total_options: self.options.len(),
            });
        }
        Ok(self.options.swap_remove(idx))
    }
}

fn default_cardinality() -> Cardinality {
    Cardinality::ONE
}

fn is_default_cardinality(c: &Cardinality) -> bool {
    *c == Cardinality::ONE
}

/// A user-facing choice where multiple options may be picked subject to a [`Cardinality`] constraint.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MultiChoice<T> {
    /// The user-facing question or prompt describing this choice.
    pub prompt: String,
    /// The candidate options available to pick from.
    pub options: Vec<T>,
    /// The cardinality constraint governing how many options must/may be selected.
    #[serde(
        rename = "choose",
        default = "default_cardinality",
        skip_serializing_if = "is_default_cardinality"
    )]
    pub cardinality: Cardinality,
    /// The lifecycle state tracking staged or confirmed selections.
    #[serde(default, skip_serializing_if = "is_default_state")]
    pub state: ChoiceState<Vec<usize>>,
}

impl<T> MultiChoice<T> {
    /// Creates a new [`MultiChoice`] defaulting to [`Cardinality::ONE`].
    ///
    /// # Arguments
    /// * `prompt` - A descriptive label or question for this choice.
    /// * `options` - An iterable sequence of candidate options.
    pub fn new(prompt: impl Into<String>, options: impl IntoIterator<Item = T>) -> Self {
        Self {
            prompt: prompt.into(),
            options: options.into_iter().collect(),
            cardinality: Cardinality::ONE,
            state: ChoiceState::Pending,
        }
    }

    /// Builder method to specify the [`Cardinality`] constraint.
    pub fn with_cardinality(mut self, cardinality: Cardinality) -> Self {
        self.cardinality = cardinality;
        self
    }

    /// Returns the cardinality constraint for this choice.
    pub fn cardinality(&self) -> Cardinality {
        self.cardinality
    }

    /// Returns a reference to the choice prompt.
    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    /// Returns a slice of all available candidate options.
    pub fn options(&self) -> &[T] {
        &self.options
    }

    /// Returns a mutable slice of all available candidate options.
    pub fn options_mut(&mut self) -> &mut [T] {
        &mut self.options
    }

    /// Returns a reference to the underlying [`ChoiceState`].
    pub fn state(&self) -> &ChoiceState<Vec<usize>> {
        &self.state
    }

    /// Returns `true` if the choice has been confirmed into [`ChoiceState::Made`].
    pub fn is_made(&self) -> bool {
        self.state.is_made()
    }

    /// Returns the first staged or confirmed index, if any.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_indices().first().copied()
    }

    /// Returns a slice of all currently staged or confirmed option indices.
    pub fn selected_indices(&self) -> &[usize] {
        match &self.state {
            ChoiceState::Made(indices) | ChoiceState::UnmadeWithHistory(indices) => indices,
            ChoiceState::Pending => &[],
        }
    }

    /// Returns a vector of references to selected options.
    pub fn selected_values(&self) -> Vec<&T> {
        self.selected_indices()
            .iter()
            .filter_map(|&i| self.options.get(i))
            .collect()
    }

    /// Returns a vector of mutable references to selected options.
    pub fn selected_values_mut(&mut self) -> Vec<&mut T> {
        let indices = self.selected_indices().to_vec();
        let mut seen = std::collections::HashSet::new();
        let unique_indices: Vec<usize> = indices.into_iter().filter(|&i| seen.insert(i)).collect();
        unique_indices
            .into_iter()
            .filter_map(|i| {
                if i < self.options.len() {
                    let ptr = self.options.as_mut_ptr();
                    // SAFETY: All indices returned are strictly unique and < options.len(), guaranteeing disjoint mutable references.
                    Some(unsafe { &mut *ptr.add(i) })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Stages an option at the specified `index`.
    ///
    /// In accordance with the universal choice contract, calling `select` stages the option
    /// into [`ChoiceState::UnmadeWithHistory`] and does **NOT** transition to [`ChoiceState::Made`].
    /// To confirm the choice, call [`confirm`](Self::confirm).
    ///
    /// # Arguments
    /// * `index` - The zero-based index of the option to stage.
    ///
    /// # Errors
    /// Returns:
    /// - [`SchematicError::IndexOutOfBounds`] if `index >= options.len()`.
    /// - [`SchematicError::DuplicateSelection`] if `index` was already staged.
    pub fn select(&mut self, index: usize) -> Result<(), SchematicError> {
        if index >= self.options.len() {
            return Err(SchematicError::IndexOutOfBounds {
                prompt: self.prompt.clone(),
                index,
                total_options: self.options.len(),
            });
        }

        let mut indices = match &self.state {
            ChoiceState::Made(idx) | ChoiceState::UnmadeWithHistory(idx) => idx.clone(),
            ChoiceState::Pending => vec![],
        };

        if indices.contains(&index) {
            return Err(SchematicError::DuplicateSelection {
                prompt: self.prompt.clone(),
                index,
            });
        }

        indices.push(index);
        self.state = ChoiceState::UnmadeWithHistory(indices);
        Ok(())
    }

    /// Confirms the staged selections, validating cardinality and transitioning state to [`ChoiceState::Made`].
    ///
    /// # Errors
    /// Returns [`SchematicError::InvalidCardinality`] if the staged count does not satisfy [`Cardinality::accepts`].
    pub fn confirm(&mut self) -> Result<(), SchematicError> {
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
            Err(SchematicError::InvalidCardinality {
                prompt: self.prompt.clone(),
                expected: self.cardinality,
                actual: indices.len(),
            })
        }
    }

    /// Deselects the option at the given `index`.
    ///
    /// If the choice was previously [`ChoiceState::Made`], removing an option demotes it
    /// back to [`ChoiceState::UnmadeWithHistory`] (or [`ChoiceState::Pending`] if empty).
    ///
    /// # Arguments
    /// * `index` - The zero-based index of the option to remove.
    ///
    /// # Errors
    /// Returns [`SchematicError::NotSelected`] if `index` was not currently staged or made.
    pub fn deselect(&mut self, index: usize) -> Result<(), SchematicError> {
        let mut indices = match &self.state {
            ChoiceState::Made(idx) | ChoiceState::UnmadeWithHistory(idx) => idx.clone(),
            ChoiceState::Pending => vec![],
        };

        if let Some(pos) = indices.iter().position(|&i| i == index) {
            indices.remove(pos);
            if indices.is_empty() {
                self.state = ChoiceState::Pending;
            } else {
                self.state = ChoiceState::UnmadeWithHistory(indices);
            }
            Ok(())
        } else {
            Err(SchematicError::NotSelected {
                prompt: self.prompt.clone(),
                index,
            })
        }
    }

    /// Alias for [`deselect`](Self::deselect).
    pub fn unselect(&mut self, index: usize) -> Result<(), SchematicError> {
        self.deselect(index)
    }

    /// Losslessly un-makes a confirmed choice, preserving the exact selection history.
    ///
    /// Does **NOT** drop or pop any items from history.
    /// Calling [`confirm`](Self::confirm) afterwards restores the choice to [`ChoiceState::Made`].
    ///
    /// # Errors
    /// Returns [`SchematicError::CannotUnmakeNotMade`] if the choice is in [`ChoiceState::Pending`].
    pub fn unmake(&mut self) -> Result<(), SchematicError> {
        match &self.state {
            ChoiceState::Made(indices) => {
                self.state = ChoiceState::UnmadeWithHistory(indices.clone());
                Ok(())
            }
            ChoiceState::UnmadeWithHistory(_) => Ok(()),
            ChoiceState::Pending => Err(SchematicError::CannotUnmakeNotMade {
                prompt: self.prompt.clone(),
            }),
        }
    }

    /// Resolves the choice into a vector of selected domain values in option declaration order.
    ///
    /// Consumes `self`.
    ///
    /// # Errors
    /// Returns [`SchematicError::UnresolvedChoice`] if the choice has not been confirmed into [`ChoiceState::Made`].
    pub fn resolve(self) -> Result<Vec<T>, SchematicError> {
        if !self.is_made() {
            return Err(SchematicError::UnresolvedChoice {
                prompt: self.prompt,
            });
        }

        let mut indices = match self.state {
            ChoiceState::Made(idx) => idx,
            _ => unreachable!(),
        };

        for &i in &indices {
            if i >= self.options.len() {
                return Err(SchematicError::IndexOutOfBounds {
                    prompt: self.prompt,
                    index: i,
                    total_options: self.options.len(),
                });
            }
        }

        // Preserve option order
        indices.sort_unstable();
        indices.dedup();

        if !self.cardinality.accepts(indices.len()) {
            return Err(SchematicError::InvalidCardinality {
                prompt: self.prompt,
                expected: self.cardinality,
                actual: indices.len(),
            });
        }

        let mut result = Vec::with_capacity(indices.len());
        for (i, opt) in self.options.into_iter().enumerate() {
            if indices.binary_search(&i).is_ok() {
                result.push(opt);
            }
        }
        Ok(result)
    }
}
