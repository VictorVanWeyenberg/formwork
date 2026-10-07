use super::leaf::Leaf;
use crate::schematic_trait::{Schematic, Level, Walkable};
use crate::choice::Choice;
use crate::error::SchematicError;
use crate::slot::{Slot};
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

pub trait BranchMode {
    type LeafItem: Clone + std::fmt::Debug + PartialEq + Eq + serde::Serialize + serde::de::DeserializeOwned;
    type Tags: Clone + std::fmt::Debug + PartialEq + Eq + serde::Serialize + serde::de::DeserializeOwned + Default;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchUnresolved;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchResolved;

impl BranchMode for BranchUnresolved {
    type LeafItem = Slot<Leaf>;
    type Tags = Vec<Slot<String>>;
}

impl BranchMode for BranchResolved {
    type LeafItem = Leaf;
    type Tags = Vec<String>;
}

pub trait PushLeafChoice<M: BranchMode> {
    fn push_choice(leaves: &mut Vec<<M as BranchMode>::LeafItem>, choice: Choice<Leaf>);
}

impl PushLeafChoice<BranchUnresolved> for BranchUnresolved {
    fn push_choice(leaves: &mut Vec<Slot<Leaf>>, choice: Choice<Leaf>) {
        leaves.push(Slot::Choose(choice));
    }
}

pub type BranchSchematic = Branch<BranchUnresolved>;

/// A Branch node in the abstract hierarchy.
///
/// In unresolved mode (`BranchUnresolved`), leaves can be added individually as fixed values or choices:
/// ```
/// use schematic::abstract_model::{BranchSchematic, Leaf};
/// use schematic::choice::Choice;
///
/// let choice = Choice::new("Pick leaf", vec![]);
/// let branch = BranchSchematic::builder()
///     .name("Branch")
///     .with_leaf(Leaf::new("L1", 1))
///     .with_leaf_choice(choice)
///     .build();
/// assert_eq!(branch.leaves.len(), 2);
/// ```
///
/// In final mode (`BranchResolved`), attempting to add a choice is rejected at compile time:
/// ```compile_fail
/// use schematic::abstract_model::{Branch, BranchResolved};
/// use schematic::choice::Choice;
///
/// let choice = Choice::new("Pick leaf", vec![]);
/// let branch = Branch::<BranchResolved>::builder()
///     .name("Branch")
///     .with_leaf_choice(choice) // Compilation error: trait bound `BranchResolved: PushLeafChoice<BranchResolved>` is not satisfied!
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
pub struct Branch<M: BranchMode = BranchResolved> {
    #[builder(setter(into))]
    pub name: String,
    #[builder(default, via_mutators)]
    pub leaves: Vec<<M as BranchMode>::LeafItem>,
    #[builder(default)]
    pub tags: M::Tags,
}

impl Branch<BranchUnresolved> {
    /// Convenience helper to create a unresolved branch with leaves and tags.
    pub fn unresolved(
        name: impl Into<String>,
        leaves: impl IntoIterator<Item = Slot<Leaf>>,
        tags: Vec<Slot<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            leaves: leaves.into_iter().collect(),
            tags,
        }
    }
}

impl<V> Walkable<V> for Branch<BranchUnresolved>
where
    V: ChoiceVisitor<Leaf> + ChoiceVisitor<String>,
{
    fn walk_level(&mut self, visitor: &mut V, depth: usize) -> Level {
        if depth == 0 {
            for leaf_slot in &mut self.leaves {
                if let Slot::Choose(choice) = leaf_slot {
                    let flow = visitor.visit_choice(choice);
                    if flow != VisitorFlow::Continue {
                        return Level::from_flow(flow);
                    }
                }
            }
            for tag_slot in &mut self.tags {
                if let Slot::Choose(choice) = tag_slot {
                    let flow = visitor.visit_choice(choice);
                    if flow != VisitorFlow::Continue {
                        return Level::from_flow(flow);
                    }
                }
            }
            Level::continue_level(false)
        } else {
            Level::continue_level(false)
        }
    }
}

impl Schematic for Branch<BranchUnresolved> {
    type Resolved = Branch<BranchResolved>;

    fn is_complete(&self) -> bool {
        self.leaves.iter().all(|s| s.is_complete()) && self.tags.iter().all(|s| s.is_complete())
    }

    fn resolve(self) -> Result<Self::Resolved, SchematicError> {
        let mut leaves = Vec::with_capacity(self.leaves.len());
        for slot in self.leaves {
            leaves.extend(slot.resolve()?);
        }
        let mut tags = Vec::with_capacity(self.tags.len());
        for tag in self.tags {
            tags.extend(tag.resolve()?);
        }
        Ok(Branch {
            name: self.name,
            leaves,
            tags,
        })
    }
}

// Convert from Resolved to Unresolved (lifting)
impl From<Branch<BranchResolved>> for Branch<BranchUnresolved> {
    fn from(value: Branch<BranchResolved>) -> Self {
        Branch {
            name: value.name,
            leaves: value.leaves.into_iter().map(Slot::fixed).collect(),
            tags: value.tags.into_iter().map(Slot::fixed).collect(),
        }
    }
}
