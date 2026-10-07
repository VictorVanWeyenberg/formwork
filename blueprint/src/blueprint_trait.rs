use crate::error::BlueprintError;
use crate::visitor::{ConfirmAll, Counter, TargetUnmaker, UnmakeAll, VisitorFlow};

/// Status and continuation signal for a single level in depth-limited BFS traversal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
    pub flow: VisitorFlow,
    pub deeper: bool,
}

impl Level {
    pub fn continue_level(deeper: bool) -> Self {
        Self {
            flow: VisitorFlow::Continue,
            deeper,
        }
    }

    pub fn break_level() -> Self {
        Self {
            flow: VisitorFlow::Break,
            deeper: false,
        }
    }

    pub fn combine(&mut self, other: Level) {
        if other.flow == VisitorFlow::Break {
            self.flow = VisitorFlow::Break;
        }
        if other.deeper {
            self.deeper = true;
        }
    }
}

/// Helper trait implemented by macro-generated draft structs to support depth-limited BFS traversal.
pub trait Walkable<V> {
    fn walk_level(&mut self, visitor: &mut V, depth: usize) -> Level;
}

/// Trait implemented by draft representations of blueprint types.
pub trait Blueprint: Sized {
    /// The final, watertight domain type.
    type Final;

    /// Checks if all choices in this hierarchy are currently made/satisfied.
    fn is_complete(&self) -> bool;

    /// Consumes the draft and converts it into its final form if all choices are satisfied.
    fn resolve(self) -> Result<Self::Final, BlueprintError>;

    /// Traverses choices in the blueprint breadth-first, invoking `visitor` on each choice.
    fn walk_choices<V>(&mut self, visitor: &mut V) -> VisitorFlow
    where
        Self: Walkable<V>,
    {
        let mut depth = 0;
        loop {
            let level = self.walk_level(visitor, depth);
            if level.flow == VisitorFlow::Break {
                return VisitorFlow::Break;
            }
            if !level.deeper {
                return VisitorFlow::Continue;
            }
            depth += 1;
        }
    }

    /// Unmakes the last made choice in breadth-first walk order, preserving its selection.
    ///
    /// Returns `true` if a choice was unmade, or `false` if no choices were made.
    fn unmake_last_choice(&mut self) -> bool
    where
        Self: Walkable<Counter> + Walkable<TargetUnmaker>,
    {
        let mut counter = Counter::default();
        self.walk_choices(&mut counter);
        if counter.count == 0 {
            return false;
        }

        let mut unmaker = TargetUnmaker::new(counter.count);
        self.walk_choices(&mut unmaker);
        true
    }

    /// Reverts all made choices in the blueprint back to pending, preserving history.
    fn unmake_all(&mut self)
    where
        Self: Walkable<UnmakeAll>,
    {
        let mut unmaker = UnmakeAll;
        self.walk_choices(&mut unmaker);
    }

    /// Re-confirms all unmade choices that have an existing selection history.
    fn confirm_all(&mut self)
    where
        Self: Walkable<ConfirmAll>,
    {
        let mut confirmer = ConfirmAll;
        self.walk_choices(&mut confirmer);
    }
}
