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

    pub fn back_level() -> Self {
        Self {
            flow: VisitorFlow::Back,
            deeper: false,
        }
    }

    pub fn from_flow(flow: VisitorFlow) -> Self {
        Self {
            flow,
            deeper: false,
        }
    }

    pub fn combine(&mut self, other: Level) {
        if other.flow == VisitorFlow::Back {
            self.flow = VisitorFlow::Back;
        } else if other.flow == VisitorFlow::Break && self.flow != VisitorFlow::Back {
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
    /// If the visitor returns `VisitorFlow::Back`, the traversal will automatically
    /// find and unmake the last choice that was made in the entire blueprint.
    fn walk_choices<V>(&mut self, visitor: &mut V) -> VisitorFlow
    where
        Self: Walkable<V> + Walkable<Counter> + Walkable<TargetUnmaker>,
    {
        let mut depth = 0;
        loop {
            let level = self.walk_level(visitor, depth);
            if level.flow == VisitorFlow::Break {
                return VisitorFlow::Break;
            }
            if level.flow == VisitorFlow::Back {
                // Stepping back requested: unmake the globally last made choice.
                let mut counter = Counter::default();
                let mut d = 0;
                loop {
                    let l = self.walk_level(&mut counter, d);
                    if !l.deeper || l.flow != VisitorFlow::Continue {
                        break;
                    }
                    d += 1;
                }

                if counter.count > 0 {
                    let mut unmaker = TargetUnmaker::new(counter.count);
                    let mut d = 0;
                    loop {
                        let l = self.walk_level(&mut unmaker, d);
                        if l.flow == VisitorFlow::Break || !l.deeper {
                            break;
                        }
                        d += 1;
                    }
                }
                return VisitorFlow::Back;
            }
            if !level.deeper {
                return VisitorFlow::Continue;
            }
            depth += 1;
        }
    }

    /// Reverts all made choices in the blueprint back to pending, preserving history.
    fn unmake_all(&mut self)
    where
        Self: Walkable<UnmakeAll> + Walkable<Counter> + Walkable<TargetUnmaker>,
    {
        let mut unmaker = UnmakeAll;
        self.walk_choices(&mut unmaker);
    }

    /// Re-confirms all unmade choices that have an existing selection history.
    fn confirm_all(&mut self)
    where
        Self: Walkable<ConfirmAll> + Walkable<Counter> + Walkable<TargetUnmaker>,
    {
        let mut confirmer = ConfirmAll;
        self.walk_choices(&mut confirmer);
    }
}
