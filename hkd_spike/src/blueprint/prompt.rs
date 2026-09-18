use crate::blueprint::{Cardinality, Choice};

/// What a UI needs from *any* choice, whatever is being chosen. Object-safe, so a draft can hand
/// out `&mut dyn ChoicePrompt` without an enum per chooseable kind (replaces `ChoiceKind`).
pub trait ChoicePrompt {
    fn prompt(&self) -> &str;
    fn cardinality(&self) -> &Cardinality;
    fn labels(&self) -> Vec<&str>;
    fn is_selected(&self, option: usize) -> bool;
    fn select(&mut self, option: usize);
    fn unselect(&mut self, option: usize);
    /// Whether the current selection satisfies the cardinality.
    fn is_ok(&self) -> bool;
}

impl<T> ChoicePrompt for Choice<T> {
    fn prompt(&self) -> &str {
        Choice::prompt(self)
    }

    fn cardinality(&self) -> &Cardinality {
        Choice::cardinality(self)
    }

    fn labels(&self) -> Vec<&str> {
        self.options().iter().map(|option| option.label()).collect()
    }

    fn is_selected(&self, option: usize) -> bool {
        self.is_label_selected(self.options()[option].label())
    }

    fn select(&mut self, option: usize) {
        Choice::select_label(self, option)
    }

    fn unselect(&mut self, option: usize) {
        Choice::unselect_label(self, option)
    }

    fn is_ok(&self) -> bool {
        Choice::is_ok(self)
    }
}
