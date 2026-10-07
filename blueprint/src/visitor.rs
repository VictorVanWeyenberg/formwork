use crate::choice::{Choice, ChoiceCollection};

/// Control flow signal for choice visitors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisitorFlow {
    Continue,
    Break,
    Back,
}

/// A visitor for walking through single-value choices of type `T` in a blueprint hierarchy.
pub trait ChoiceVisitor<T> {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow;
}

/// A visitor for walking through collection choices of type `T` in a blueprint hierarchy.
pub trait ChoiceCollectionVisitor<T> {
    fn visit_choice_collection(
        &mut self,
        choice: &mut ChoiceCollection<T>,
    ) -> VisitorFlow;
}

impl<T, V: ChoiceVisitor<T> + ?Sized> ChoiceVisitor<T> for &mut V {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow {
        (**self).visit_choice(choice)
    }
}

impl<T, V: ChoiceCollectionVisitor<T> + ?Sized> ChoiceCollectionVisitor<T> for &mut V {
    fn visit_choice_collection(
        &mut self,
        choice: &mut ChoiceCollection<T>,
    ) -> VisitorFlow {
        (**self).visit_choice_collection(choice)
    }
}

/// Utility visitor that counts how many choices are currently made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counter {
    pub count: usize,
}

impl<T> ChoiceVisitor<T> for Counter {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow {
        if choice.is_made() {
            self.count += 1;
        }
        VisitorFlow::Continue
    }
}

impl<T> ChoiceCollectionVisitor<T> for Counter {
    fn visit_choice_collection(
        &mut self,
        choice: &mut ChoiceCollection<T>,
    ) -> VisitorFlow {
        if choice.is_made() {
            self.count += 1;
        }
        VisitorFlow::Continue
    }
}

/// Utility visitor that unmakes the target-th made choice encountered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetUnmaker {
    pub current: usize,
    pub target: usize,
}

impl TargetUnmaker {
    pub fn new(target: usize) -> Self {
        Self { current: 0, target }
    }
}

impl<T> ChoiceVisitor<T> for TargetUnmaker {
    fn visit_choice(&mut self, choice: &mut Choice<T>) -> VisitorFlow {
        if choice.is_made() {
            self.current += 1;
            if self.current == self.target {
                let _ = choice.unmake();
                return VisitorFlow::Break;
            }
        }
        VisitorFlow::Continue
    }
}

impl<T> ChoiceCollectionVisitor<T> for TargetUnmaker {
    fn visit_choice_collection(
        &mut self,
        choice: &mut ChoiceCollection<T>,
    ) -> VisitorFlow {
        if choice.is_made() {
            self.current += 1;
            if self.current == self.target {
                let _ = choice.unmake();
                return VisitorFlow::Break;
            }
        }
        VisitorFlow::Continue
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

impl<T> ChoiceCollectionVisitor<T> for UnmakeAll {
    fn visit_choice_collection(
        &mut self,
        choice: &mut ChoiceCollection<T>,
    ) -> VisitorFlow {
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

impl<T> ChoiceCollectionVisitor<T> for ConfirmAll {
    fn visit_choice_collection(
        &mut self,
        choice: &mut ChoiceCollection<T>,
    ) -> VisitorFlow {
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

impl<T> ChoiceCollectionVisitor<T> for AutoSelectFirst {
    fn visit_choice_collection(
        &mut self,
        choice: &mut ChoiceCollection<T>,
    ) -> VisitorFlow {
        if !choice.is_made() && !choice.options().is_empty() {
            let _ = choice.select(&[0]);
        }
        VisitorFlow::Continue
    }
}

/// Utility visitor that immediately returns `VisitorFlow::Back` to step backward in the traversal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepBack;

impl<T> ChoiceVisitor<T> for StepBack {
    fn visit_choice(&mut self, _choice: &mut Choice<T>) -> VisitorFlow {
        VisitorFlow::Back
    }
}

impl<T> ChoiceCollectionVisitor<T> for StepBack {
    fn visit_choice_collection(
        &mut self,
        _choice: &mut ChoiceCollection<T>,
    ) -> VisitorFlow {
        VisitorFlow::Back
    }
}
