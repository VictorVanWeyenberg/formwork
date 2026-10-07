use super::branch::{Branch, BranchUnresolved, BranchResolved};
use crate::schematic_trait::{Schematic, Level, Walkable};
use crate::choice::Choice;
use crate::error::SchematicError;
use crate::slot::Slot;
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

pub trait RootMode {
    type PrimaryBranch: Clone + std::fmt::Debug + PartialEq + Eq + serde::Serialize + serde::de::DeserializeOwned;
    type BranchItem: Clone + std::fmt::Debug + PartialEq + Eq + serde::Serialize + serde::de::DeserializeOwned;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootUnresolved;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootResolved;

impl RootMode for RootUnresolved {
    type PrimaryBranch = Slot<Branch<BranchUnresolved>>;
    type BranchItem = Slot<Branch<BranchUnresolved>>;
}

impl RootMode for RootResolved {
    type PrimaryBranch = Branch<BranchResolved>;
    type BranchItem = Branch<BranchResolved>;
}

pub trait PushBranchChoice<M: RootMode> {
    fn push_choice(
        branches: &mut Vec<<M as RootMode>::BranchItem>,
        choice: Choice<Branch<BranchUnresolved>>,
    );
}

impl PushBranchChoice<RootUnresolved> for RootUnresolved {
    fn push_choice(
        branches: &mut Vec<Slot<Branch<BranchUnresolved>>>,
        choice: Choice<Branch<BranchUnresolved>>,
    ) {
        branches.push(Slot::Choose(choice));
    }
}

pub type RootSchematic = Root<RootUnresolved>;

/// A Root node in the abstract hierarchy.
///
/// In unresolved mode (`RootUnresolved`), branches can be added individually as fixed values or choices:
/// ```
/// use schematic::abstract_model::{RootSchematic, BranchSchematic};
/// use schematic::choice::Choice;
///
/// let primary = BranchSchematic::builder().name("Primary").build();
/// let choice = Choice::new("Pick branch", vec![]);
/// let root = RootSchematic::builder()
///     .id("r1")
///     .primary_branch(primary)
///     .with_branch(BranchSchematic::builder().name("B1").build())
///     .with_branch_choice(choice)
///     .build();
/// assert_eq!(root.branches.len(), 2);
/// ```
///
/// In final mode (`RootResolved`), attempting to add a choice is rejected at compile time:
/// ```compile_fail
/// use schematic::abstract_model::{Root, RootResolved, Branch, BranchResolved};
/// use schematic::choice::Choice;
///
/// let primary = Branch::<BranchResolved>::builder().name("Primary").build();
/// let choice = Choice::new("Pick branch", vec![]);
/// let root = Root::<RootResolved>::builder()
///     .id("r1")
///     .primary_branch(primary)
///     .with_branch_choice(choice) // Compilation error: trait bound `RootResolved: PushBranchChoice<RootResolved>` is not satisfied!
///     .build();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[builder(mutators(
    pub fn with_branch(&mut self, branch: impl Into<<M as RootMode>::BranchItem>) {
        self.branches.push(branch.into());
    }
    pub fn with_branch_choice(&mut self, choice: Choice<Branch<BranchUnresolved>>)
    where
        M: PushBranchChoice<M>,
    {
        M::push_choice(&mut self.branches, choice);
    }
))]
#[serde(bound = "")]
pub struct Root<M: RootMode = RootResolved> {
    #[builder(setter(into))]
    pub id: String,
    #[builder(setter(into))]
    pub primary_branch: M::PrimaryBranch,
    #[builder(default, via_mutators)]
    pub branches: Vec<<M as RootMode>::BranchItem>,
}

impl Root<RootUnresolved> {
    pub fn unresolved(
        id: impl Into<String>,
        primary_branch: impl Into<Slot<Branch<BranchUnresolved>>>,
        branches: impl IntoIterator<Item = Slot<Branch<BranchUnresolved>>>,
    ) -> Self {
        Self {
            id: id.into(),
            primary_branch: primary_branch.into(),
            branches: branches.into_iter().collect(),
        }
    }
}

impl<V> Walkable<V> for Root<RootUnresolved>
where
    V: ChoiceVisitor<Branch<BranchUnresolved>>,
    Branch<BranchUnresolved>: Walkable<V>,
{
    fn walk_level(&mut self, visitor: &mut V, depth: usize) -> Level {
        if depth == 0 {
            // --- Immediate choices at Root level ---
            if let Slot::Choose(choice) = &mut self.primary_branch {
                let flow = visitor.visit_choice(choice);
                if flow != VisitorFlow::Continue {
                    return Level::from_flow(flow);
                }
            }
            for branch_slot in &mut self.branches {
                if let Slot::Choose(choice) = branch_slot {
                    let flow = visitor.visit_choice(choice);
                    if flow != VisitorFlow::Continue {
                        return Level::from_flow(flow);
                    }
                }
            }
            let has_children = !self.primary_branch.selected_values().is_empty()
                || self.branches.iter().any(|b| !b.selected_values().is_empty());
            Level::continue_level(has_children)
        } else {
            // --- Recurse into selected or fixed children at depth - 1 ---
            let mut level = Level::continue_level(false);
            for child in self.primary_branch.selected_values_mut() {
                level.combine(child.walk_level(visitor, depth - 1));
                if level.flow == VisitorFlow::Break {
                    return level;
                }
            }
            for branch_slot in &mut self.branches {
                for child in branch_slot.selected_values_mut() {
                    level.combine(child.walk_level(visitor, depth - 1));
                    if level.flow == VisitorFlow::Break {
                        return level;
                    }
                }
            }
            level
        }
    }
}

impl Schematic for Root<RootUnresolved> {
    type Resolved = Root<RootResolved>;

    fn is_complete(&self) -> bool {
        let primary_ok = self.primary_branch.is_complete()
            && self
                .primary_branch
                .selected_values()
                .iter().all(|b| b.is_complete());
                
        let branches_ok = self.branches.iter().all(|slot| {
            slot.is_complete()
                && slot
                    .selected_values()
                    .iter().all(|b| b.is_complete())
        });
        primary_ok && branches_ok
    }

    fn resolve(self) -> Result<Self::Resolved, SchematicError> {
        let primary_final = self.primary_branch.resolve_exact()?.resolve()?;

        let mut branches_final = Vec::with_capacity(self.branches.len());
        for slot in self.branches {
            for unresolved in slot.resolve()? {
            branches_final.push(unresolved.resolve()?);
            }
        }

        Ok(Root {
            id: self.id,
            primary_branch: primary_final,
            branches: branches_final,
        })
    }
}

// Convert from Resolved to Unresolved (lifting)
impl From<Root<RootResolved>> for Root<RootUnresolved> {
    fn from(value: Root<RootResolved>) -> Self {
        Root {
            id: value.id,
            primary_branch: Slot::fixed(value.primary_branch.into()),
            branches: value
                .branches
                .into_iter()
                .map(|b| Slot::fixed(b.into()))
                .collect(),
        }
    }
}
