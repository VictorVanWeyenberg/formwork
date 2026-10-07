use super::branch::{Branch, BranchDraft, BranchFinal};
use crate::blueprint_trait::{Blueprint, Level, Walkable};
use crate::choice::Choice;
use crate::error::BlueprintError;
use crate::node::Node;
use crate::slot::Slot;
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

pub trait RootMode {
    type PrimaryBranch: Node;
    type BranchItem: Node;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootDraft;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootFinal;

impl RootMode for RootDraft {
    type PrimaryBranch = Slot<Branch<BranchDraft>>;
    type BranchItem = Slot<Branch<BranchDraft>>;
}

impl RootMode for RootFinal {
    type PrimaryBranch = Branch<BranchFinal>;
    type BranchItem = Branch<BranchFinal>;
}

pub trait PushBranchChoice<M: RootMode> {
    fn push_choice(
        branches: &mut Vec<<M as RootMode>::BranchItem>,
        choice: Choice<Branch<BranchDraft>>,
    );
}

impl PushBranchChoice<RootDraft> for RootDraft {
    fn push_choice(
        branches: &mut Vec<Slot<Branch<BranchDraft>>>,
        choice: Choice<Branch<BranchDraft>>,
    ) {
        branches.push(Slot::Choose(choice));
    }
}

pub type RootBlueprint = Root<RootDraft>;

/// A Root node in the abstract hierarchy.
///
/// In draft mode (`RootDraft`), branches can be added individually as fixed values or choices:
/// ```
/// use blueprint::abstract_model::{RootBlueprint, BranchBlueprint};
/// use blueprint::choice::Choice;
///
/// let primary = BranchBlueprint::builder().name("Primary").build();
/// let choice = Choice::new("Pick branch", vec![]);
/// let root = RootBlueprint::builder()
///     .id("r1")
///     .primary_branch(primary)
///     .with_branch(BranchBlueprint::builder().name("B1").build())
///     .with_branch_choice(choice)
///     .build();
/// assert_eq!(root.branches.len(), 2);
/// ```
///
/// In final mode (`RootFinal`), attempting to add a choice is rejected at compile time:
/// ```compile_fail
/// use blueprint::abstract_model::{Root, RootFinal, Branch, BranchFinal};
/// use blueprint::choice::Choice;
///
/// let primary = Branch::<BranchFinal>::builder().name("Primary").build();
/// let choice = Choice::new("Pick branch", vec![]);
/// let root = Root::<RootFinal>::builder()
///     .id("r1")
///     .primary_branch(primary)
///     .with_branch_choice(choice) // Compilation error: trait bound `RootFinal: PushBranchChoice<RootFinal>` is not satisfied!
///     .build();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[builder(mutators(
    pub fn with_branch(&mut self, branch: impl Into<<M as RootMode>::BranchItem>) {
        self.branches.push(branch.into());
    }
    pub fn with_branch_choice(&mut self, choice: Choice<Branch<BranchDraft>>)
    where
        M: PushBranchChoice<M>,
    {
        M::push_choice(&mut self.branches, choice);
    }
))]
#[serde(bound = "")]
pub struct Root<M: RootMode = RootFinal> {
    #[builder(setter(into))]
    pub id: String,
    #[builder(setter(into))]
    pub primary_branch: M::PrimaryBranch,
    #[builder(default, via_mutators)]
    pub branches: Vec<<M as RootMode>::BranchItem>,
}

impl Root<RootDraft> {
    pub fn draft(
        id: impl Into<String>,
        primary_branch: impl Into<Slot<Branch<BranchDraft>>>,
        branches: impl IntoIterator<Item = Slot<Branch<BranchDraft>>>,
    ) -> Self {
        Self {
            id: id.into(),
            primary_branch: primary_branch.into(),
            branches: branches.into_iter().collect(),
        }
    }
}

impl<V> Walkable<V> for Root<RootDraft>
where
    V: ChoiceVisitor<Branch<BranchDraft>>,
    Branch<BranchDraft>: Walkable<V>,
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
            let has_children = self.primary_branch.selected_value().is_some()
                || self.branches.iter().any(|b| b.selected_value().is_some());
            Level::continue_level(has_children)
        } else {
            // --- Recurse into selected or fixed children at depth - 1 ---
            let mut level = Level::continue_level(false);
            if let Some(child) = self.primary_branch.selected_value_mut() {
                level.combine(child.walk_level(visitor, depth - 1));
                if level.flow == VisitorFlow::Break {
                    return level;
                }
            }
            for branch_slot in &mut self.branches {
                if let Some(child) = branch_slot.selected_value_mut() {
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

impl Blueprint for Root<RootDraft> {
    type Final = Root<RootFinal>;

    fn is_complete(&self) -> bool {
        let primary_ok = self.primary_branch.is_complete()
            && self
                .primary_branch
                .selected_value()
                .map(|b| b.is_complete())
                .unwrap_or(false);
        let branches_ok = self.branches.iter().all(|slot| {
            slot.is_complete()
                && slot
                    .selected_value()
                    .map(|b| b.is_complete())
                    .unwrap_or(false)
        });
        primary_ok && branches_ok
    }

    fn resolve(self) -> Result<Self::Final, BlueprintError> {
        let primary_draft = self.primary_branch.resolve()?;
        let primary_final = primary_draft.resolve()?;

        let mut branches_final = Vec::with_capacity(self.branches.len());
        for slot in self.branches {
            let draft = slot.resolve()?;
            let final_b = draft.resolve()?;
            branches_final.push(final_b);
        }

        Ok(Root {
            id: self.id,
            primary_branch: primary_final,
            branches: branches_final,
        })
    }
}

// Convert from Final to Draft (lifting)
impl From<Root<RootFinal>> for Root<RootDraft> {
    fn from(value: Root<RootFinal>) -> Self {
        Root {
            id: value.id,
            primary_branch: Slot::Fixed(value.primary_branch.into()),
            branches: value
                .branches
                .into_iter()
                .map(|b| Slot::Fixed(b.into()))
                .collect(),
        }
    }
}
