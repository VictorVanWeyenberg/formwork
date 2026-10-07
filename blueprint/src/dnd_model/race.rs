use super::trait_def::{Trait, TraitDraft, TraitFinal};
use crate::blueprint_trait::{Blueprint, Level, Walkable};
use crate::choice::Choice;
use crate::error::BlueprintError;
use crate::node::Node;
use crate::slot::{Slot, SlotCollection};
use crate::visitor::{ChoiceCollectionVisitor, ChoiceVisitor, VisitorFlow};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

pub trait RaceMode {
    type TraitItem: Node;
    type BonusSkills: Node + Default;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaceDraft;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaceFinal;

impl RaceMode for RaceDraft {
    type TraitItem = Slot<Trait<TraitDraft>>;
    type BonusSkills = SlotCollection<String>;
}

impl RaceMode for RaceFinal {
    type TraitItem = Trait<TraitFinal>;
    type BonusSkills = Vec<String>;
}

pub trait PushTraitChoice<M: RaceMode> {
    fn push_choice(traits: &mut Vec<<M as RaceMode>::TraitItem>, choice: Choice<Trait<TraitDraft>>);
}

impl PushTraitChoice<RaceDraft> for RaceDraft {
    fn push_choice(traits: &mut Vec<Slot<Trait<TraitDraft>>>, choice: Choice<Trait<TraitDraft>>) {
        traits.push(Slot::Choose(choice));
    }
}

pub type RaceBlueprint = Race<RaceDraft>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[builder(mutators(
    pub fn with_trait(&mut self, trait_: impl Into<<M as RaceMode>::TraitItem>) {
        self.traits.push(trait_.into());
    }
    pub fn with_trait_choice(&mut self, choice: Choice<Trait<TraitDraft>>)
    where
        M: PushTraitChoice<M>,
    {
        M::push_choice(&mut self.traits, choice);
    }
))]
#[serde(bound = "")]
pub struct Race<M: RaceMode = RaceFinal> {
    #[builder(setter(into))]
    pub id: String,
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub description: String,
    #[builder(default, via_mutators)]
    pub traits: Vec<<M as RaceMode>::TraitItem>,
    #[builder(default)]
    pub bonus_skills: M::BonusSkills,
}

impl Race<RaceDraft> {
    pub fn draft<I, T>(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        traits: I,
        bonus_skills: impl Into<SlotCollection<String>>,
    ) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Slot<Trait<TraitDraft>>>,
    {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            traits: traits.into_iter().map(Into::into).collect(),
            bonus_skills: bonus_skills.into(),
        }
    }
}

impl<V> Walkable<V> for Race<RaceDraft>
where
    V: ChoiceVisitor<Trait<TraitDraft>> + ChoiceCollectionVisitor<String>,
    Trait<TraitDraft>: Walkable<V>,
{
    fn walk_level(&mut self, visitor: &mut V, depth: usize) -> Level {
        if depth == 0 {
            // --- Immediate choices at Race level ---
            if let SlotCollection::Choose(choice) = &mut self.bonus_skills {
                let flow = visitor.visit_choice_collection(choice);
                if flow != VisitorFlow::Continue {
                    return Level::from_flow(flow);
                }
            }
            for trait_slot in &mut self.traits {
                if let Slot::Choose(choice) = trait_slot {
                    let flow = visitor.visit_choice(choice);
                    if flow != VisitorFlow::Continue {
                        return Level::from_flow(flow);
                    }
                }
            }
            let has_children = self.traits.iter().any(|t| t.selected_value().is_some());
            Level::continue_level(has_children)
        } else {
            // --- Recurse into selected or fixed children at depth - 1 ---
            let mut level = Level::continue_level(false);
            for trait_slot in &mut self.traits {
                if let Some(child_trait) = trait_slot.selected_value_mut() {
                    level.combine(child_trait.walk_level(visitor, depth - 1));
                    if level.flow == VisitorFlow::Break {
                        return level;
                    }
                }
            }
            level
        }
    }
}

impl Blueprint for Race<RaceDraft> {
    type Final = Race<RaceFinal>;

    fn is_complete(&self) -> bool {
        self.bonus_skills.is_complete()
            && self.traits.iter().all(|slot| {
                slot.is_complete()
                    && slot
                        .selected_value()
                        .map(|t| t.is_complete())
                        .unwrap_or(false)
            })
    }

    fn resolve(self) -> Result<Self::Final, BlueprintError> {
        let bonus_skills = self.bonus_skills.resolve()?;
        let mut traits = Vec::with_capacity(self.traits.len());
        for slot in self.traits {
            let trait_draft = slot.resolve()?;
            let trait_final = trait_draft.resolve()?;
            traits.push(trait_final);
        }

        Ok(Race {
            id: self.id,
            name: self.name,
            description: self.description,
            traits,
            bonus_skills,
        })
    }
}

impl From<Race<RaceFinal>> for Race<RaceDraft> {
    fn from(value: Race<RaceFinal>) -> Self {
        Race {
            id: value.id,
            name: value.name,
            description: value.description,
            traits: value
                .traits
                .into_iter()
                .map(|t| Slot::Fixed(t.into()))
                .collect(),
            bonus_skills: SlotCollection::Fixed(value.bonus_skills),
        }
    }
}
