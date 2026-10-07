use super::leaf::Leaf;
use crate::blueprint_trait::{Blueprint, Level, Walkable};
use crate::choice::Choice;
use crate::error::BlueprintError;
use crate::node::Node;
use crate::slot::{Slot, SlotCollection};
use crate::visitor::{ChoiceCollectionVisitor, ChoiceVisitor, VisitorFlow};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

pub trait BranchMode {
    type LeafItem: Node;
    type Tags: Node + Default;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchDraft;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchFinal;

impl BranchMode for BranchDraft {
    type LeafItem = Slot<Leaf>;
    type Tags = SlotCollection<String>;
}

impl BranchMode for BranchFinal {
    type LeafItem = Leaf;
    type Tags = Vec<String>;
}

pub trait PushLeafChoice<M: BranchMode> {
    fn push_choice(leaves: &mut Vec<<M as BranchMode>::LeafItem>, choice: Choice<Leaf>);
}

impl PushLeafChoice<BranchDraft> for BranchDraft {
    fn push_choice(leaves: &mut Vec<Slot<Leaf>>, choice: Choice<Leaf>) {
        leaves.push(Slot::Choose(choice));
    }
}

pub type BranchBlueprint = Branch<BranchDraft>;

/// A Branch node in the abstract hierarchy.
///
/// In draft mode (`BranchDraft`), leaves can be added individually as fixed values or choices:
/// ```
/// use blueprint::abstract_model::{BranchBlueprint, Leaf};
/// use blueprint::choice::Choice;
///
/// let choice = Choice::new("Pick leaf", vec![]);
/// let branch = BranchBlueprint::builder()
///     .name("Branch")
///     .with_leaf(Leaf::new("L1", 1))
///     .with_leaf_choice(choice)
///     .build();
/// assert_eq!(branch.leaves.len(), 2);
/// ```
///
/// In final mode (`BranchFinal`), attempting to add a choice is rejected at compile time:
/// ```compile_fail
/// use blueprint::abstract_model::{Branch, BranchFinal};
/// use blueprint::choice::Choice;
///
/// let choice = Choice::new("Pick leaf", vec![]);
/// let branch = Branch::<BranchFinal>::builder()
///     .name("Branch")
///     .with_leaf_choice(choice) // Compilation error: trait bound `BranchFinal: PushLeafChoice<BranchFinal>` is not satisfied!
///     .build();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[builder(mutators(
    pub fn with_leaf(&mut self, leaf: impl Into<<M as BranchMode>::LeafItem>) {
        self.leaves.push(leaf.into());
    }
    pub fn with_leaf_choice(&mut self, choice: Choice<Leaf>)
    where
        M: PushLeafChoice<M>,
    {
        M::push_choice(&mut self.leaves, choice);
    }
))]
#[serde(bound = "")]
pub struct Branch<M: BranchMode = BranchFinal> {
    #[builder(setter(into))]
    pub name: String,
    #[builder(default, via_mutators)]
    pub leaves: Vec<<M as BranchMode>::LeafItem>,
    #[builder(default)]
    pub tags: M::Tags,
}

impl Branch<BranchDraft> {
    /// Convenience helper to create a draft branch with leaves and tags.
    pub fn draft(
        name: impl Into<String>,
        leaves: impl IntoIterator<Item = Slot<Leaf>>,
        tags: impl Into<SlotCollection<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            leaves: leaves.into_iter().collect(),
            tags: tags.into(),
        }
    }
}

impl<V> Walkable<V> for Branch<BranchDraft>
where
    V: ChoiceVisitor<Leaf> + ChoiceCollectionVisitor<String>,
{
    fn walk_level(&mut self, visitor: &mut V, depth: usize) -> Level {
        if depth == 0 {
            for leaf_slot in &mut self.leaves {
                if let Slot::Choose(choice) = leaf_slot {
                    if visitor.visit_choice(choice) == VisitorFlow::Break {
                        return Level::break_level();
                    }
                }
            }
            if let SlotCollection::Choose(choice) = &mut self.tags {
                if visitor.visit_choice_collection(choice) == VisitorFlow::Break {
                    return Level::break_level();
                }
            }
            Level::continue_level(false)
        } else {
            Level::continue_level(false)
        }
    }
}

impl Blueprint for Branch<BranchDraft> {
    type Final = Branch<BranchFinal>;

    fn is_complete(&self) -> bool {
        self.leaves.iter().all(|s| s.is_complete()) && self.tags.is_complete()
    }

    fn resolve(self) -> Result<Self::Final, BlueprintError> {
        let mut leaves = Vec::with_capacity(self.leaves.len());
        for slot in self.leaves {
            leaves.push(slot.resolve()?);
        }
        let tags = self.tags.resolve()?;
        Ok(Branch {
            name: self.name,
            leaves,
            tags,
        })
    }
}

// Convert from Final to Draft (lifting)
impl From<Branch<BranchFinal>> for Branch<BranchDraft> {
    fn from(value: Branch<BranchFinal>) -> Self {
        Branch {
            name: value.name,
            leaves: value.leaves.into_iter().map(Slot::Fixed).collect(),
            tags: SlotCollection::Fixed(value.tags),
        }
    }
}
