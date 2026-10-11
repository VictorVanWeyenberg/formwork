use crate::choice::{MultiChoice, SingleChoice};

/// Control flow signal returned by a [`ChoiceVisitor`] to govern schematic tree traversal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitorFlow {
    /// Continue traversing the remaining choices in the schematic.
    Continue,
    /// Halt traversal immediately.
    Break,
}

/// A visitor for walking through and interacting with choices across a schematic hierarchy.
///
/// Implement this trait to inspect, prompt, auto-select, or reset choices during traversal
/// initiated by [`Schematic::walk_choices`](crate::Schematic::walk_choices).
///
/// # Example
/// ```rust
/// use schematic_core::choice::{SingleChoice, MultiChoice};
/// use schematic_core::visitor::{ChoiceVisitor, VisitorFlow};
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
}

impl<T, V: ChoiceVisitor<T> + ?Sized> ChoiceVisitor<T> for &mut V {
    fn visit_single(&mut self, choice: &mut SingleChoice<T>) -> VisitorFlow {
        (**self).visit_single(choice)
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        (**self).visit_multi(choice)
    }
}

/// Built-in visitor that unmakes all choices in a schematic hierarchy.
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
        }
        VisitorFlow::Continue
    }

    fn visit_multi(&mut self, choice: &mut MultiChoice<T>) -> VisitorFlow {
        if !choice.is_made() {
            for i in 0..choice.options().len() {
                if choice.is_made() {
                    break;
                }
                let _ = choice.select(i);
            }
            if !choice.is_made() {
                let _ = choice.confirm();
            }
        }
        VisitorFlow::Continue
    }
}
