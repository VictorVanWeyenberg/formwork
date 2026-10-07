use crate::choice::Choice;

/// Control flow signal for choice visitors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitorFlow {
    Continue,
    Break,
}

/// A visitor for walking through choices of type `T` in a blueprint hierarchy.
pub trait ChoiceVisitor<T> {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow;
}

impl<T, V: ChoiceVisitor<T> + ?Sized> ChoiceVisitor<T> for &mut V {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow {
        (**self).visit_choice(choice)
    }
}

/// Utility visitor that unmakes all choices in a blueprint hierarchy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnmakeAll;

impl<T> ChoiceVisitor<T> for UnmakeAll {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow {
        if choice.is_made() {
            let _ = choice.unmake();
        }
        VisitorFlow::Continue
    }
}

/// Utility visitor that re-confirms all unmade choices that have an existing selection history.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConfirmAll;

impl<T> ChoiceVisitor<T> for ConfirmAll {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow {
        let _ = choice.confirm();
        VisitorFlow::Continue
    }
}

/// Utility visitor that automatically selects index 0 for all open choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutoSelectFirst;

impl<T> ChoiceVisitor<T> for AutoSelectFirst {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow {
        if !choice.is_made() && !choice.options().is_empty() {
            let _ = choice.select(0);
        }
        VisitorFlow::Continue
    }
}
