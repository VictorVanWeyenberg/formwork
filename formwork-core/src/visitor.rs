use crate::choice::{Cardinality, Freeform, MultiChoice, SingleChoice};
#[cfg(doc)]
use crate::formwork_trait::Formwork;

/// Control flow signal returned by a [`ChoiceVisitor`] to govern formwork tree traversal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitorFlow {
    /// Continue traversing the remaining choices in the formwork.
    Continue,
    /// Halt traversal immediately.
    Break,
    /// Rewind traversal to the previous made choice, unmaking it.
    Back,
}

impl VisitorFlow {
    /// Returns `true` if this signal is [`VisitorFlow::Continue`].
    pub fn is_continue(self) -> bool {
        self == Self::Continue
    }

    /// Returns `true` if this signal is [`VisitorFlow::Break`].
    pub fn is_break(self) -> bool {
        self == Self::Break
    }

    /// Returns `true` if this signal is [`VisitorFlow::Back`].
    pub fn is_back(self) -> bool {
        self == Self::Back
    }
}

/// A visitor for walking through and interacting with choices across a formwork hierarchy.
///
/// Implement this trait to inspect, prompt, auto-select, or reset choices during traversal
/// initiated by [`Formwork::walk_choices`].
///
/// # Example
/// ```rust
/// use formwork_core::choice::{SingleChoice, MultiChoice};
/// use formwork_core::visitor::{ChoiceVisitor, VisitorFlow};
///
/// struct ChoiceCounter {
///     count: usize,
/// }
///
/// impl<T> ChoiceVisitor<T> for ChoiceCounter {
///     fn visit_single(&mut self, _choice: &mut SingleChoice<T>) -> VisitorFlow {
///         self.count += 1;
///         VisitorFlow::Continue
///     }
///
///     fn visit_multi(&mut self, _choice: &mut MultiChoice<T>) -> VisitorFlow {
///         self.count += 1;
///         VisitorFlow::Continue
///     }
/// }
/// ```
pub trait ChoiceVisitor<T> {
    /// Invoked for each [`SingleChoice`] encountered during traversal.
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow;

    /// Invoked for each [`MultiChoice`] encountered during traversal.
    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow;

    /// Invoked for each [`Freeform`] choice encountered during traversal.
    ///
    /// Default implementation returns [`VisitorFlow::Continue`].
    fn visit_freeform(&mut self, _choice: &mut Freeform<T>) -> VisitorFlow {
        VisitorFlow::Continue
    }
}

impl<T, V: ChoiceVisitor<T> + ?Sized> ChoiceVisitor<T> for &mut V {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        (**self).visit_single(choice)
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        (**self).visit_multi(choice)
    }

    fn visit_freeform(&mut self, choice: &mut Freeform<T>) -> VisitorFlow {
        (**self).visit_freeform(choice)
    }
}

/// Built-in visitor that unmakes all choices in a formwork hierarchy.
///
/// Reverts made choices back to pending while saving their previous selections
/// in `ChoiceState::UnmadeWithHistory`, allowing them to be restored later with [`ConfirmAll`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnmakeAll;

impl<T> ChoiceVisitor<T> for UnmakeAll {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        if choice.is_made() {
            let _ = choice.unmake();
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        if choice.is_made() {
            let _ = choice.unmake();
        }
        VisitorFlow::Continue
    }

    fn visit_freeform(&mut self, choice: &mut Freeform<T>) -> VisitorFlow {
        if choice.is_made() {
            let _ = choice.unmake();
        }
        VisitorFlow::Continue
    }
}

/// Built-in visitor that re-confirms all unmade choices having an existing selection history.
///
/// If a choice was previously unmade using [`UnmakeAll`], `ConfirmAll` restores the previous
/// selections and marks the choice made again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConfirmAll;

impl<T> ChoiceVisitor<T> for ConfirmAll {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        let _ = choice.confirm();
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        let _ = choice.confirm();
        VisitorFlow::Continue
    }

    fn visit_freeform(&mut self, choice: &mut Freeform<T>) -> VisitorFlow {
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

/// Built-in utility visitor that automatically selects the first available options for all open/pending choices.
///
/// For [`SingleChoice`], selects index 0. For [`MultiChoice`], selects candidate options starting
/// from index 0 until the choice constraint is satisfied.
/// Useful in tests, prototyping, or default-preset generation to quickly satisfy all choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutoSelectFirst;

impl<T> ChoiceVisitor<T> for AutoSelectFirst {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        if !choice.is_made() && !choice.options().is_empty() {
            let _ = choice.select(0);
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        if !choice.is_made() {
            let target_count = match choice.cardinality {
                Cardinality::Exact(n) => n,
                Cardinality::AtLeast(n) => n,
                Cardinality::UpTo(n) => n.min(choice.options().len()),
            };
            for i in 0..target_count.min(choice.options().len()) {
                let _ = choice.select(i);
            }
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }

    fn visit_freeform(&mut self, choice: &mut Freeform<T>) -> VisitorFlow {
        if !choice.is_made() && choice.value().is_some() {
            let _ = choice.confirm();
        }
        VisitorFlow::Continue
    }
}
