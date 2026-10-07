use super::trait_def::{Trait, TraitDraft, TraitFinal};
use crate::blueprint_trait::{Blueprint, Level, Walkable};
use crate::choice::Choice;
use crate::error::BlueprintError;
use crate::slot::Slot;
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

pub trait RaceMode {
    type TraitItem: Clone + std::fmt::Debug + PartialEq + Eq + serde::Serialize + serde::de::DeserializeOwned;
    type BonusSkills: Clone + std::fmt::Debug + PartialEq + Eq + serde::Serialize + serde::de::DeserializeOwned + Default;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaceDraft;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaceFinal;

impl RaceMode for RaceDraft {
    type TraitItem = Slot<Trait<TraitDraft>>;
    type BonusSkills = Vec<Slot<String>>;
}

impl RaceMode for RaceFinal {
    type TraitItem = Trait<TraitFinal>;
    type BonusSkills = Vec<String>;
}

pub type RaceBlueprint = Race<RaceDraft>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[builder(mutators(
    pub fn with_trait(&mut self, trait_: impl Into<<M as RaceMode>::TraitItem>) {
        self.traits.push(trait_.into());
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
        bonus_skills: Vec<Slot<String>>,
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
            bonus_skills,
        }
    }

    pub fn add_trait_choice(&mut self, choice: Choice<Trait<TraitDraft>>) {
        self.traits.push(Slot::Choose(choice));
    }
}

impl<V> Walkable<V> for Race<RaceDraft>
where
    V: ChoiceVisitor<Trait<TraitDraft>> + ChoiceVisitor<String>,
    Trait<TraitDraft>: Walkable<V>,
{
    fn walk_level(&mut self, visitor: &mut V, depth: usize) -> Level {
        if depth == 0 {
            for skill_slot in &mut self.bonus_skills {
                if let Slot::Choose(choice) = skill_slot {
                    let flow = visitor.visit_choice(choice);
                    if flow != VisitorFlow::Continue {
                        return Level::from_flow(flow);
                    }
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
            
            for trait_slot in &mut self.traits {
                if let Some(child_trait) = trait_slot.as_fixed_mut() {
                    let flow = child_trait.walk_choices(visitor);
                    if flow == VisitorFlow::Break {
                        return Level::from_flow(flow);
                    }
                }
                if let Some(choice) = trait_slot.as_choice_mut() {
                    for child_trait in choice.selected_values_mut() {
                        let flow = child_trait.walk_choices(visitor);
                        if flow == VisitorFlow::Break {
                            return Level::from_flow(flow);
                        }
                    }
                }
            }
            Level::continue_level(false)
        } else {
            Level::continue_level(false)
        }
    }
}

impl Blueprint for Race<RaceDraft> {
    type Final = Race<RaceFinal>;

    fn is_complete(&self) -> bool {
        self.bonus_skills.iter().all(|s| s.is_complete())
            && self.traits.iter().all(|slot| {
                slot.is_complete()
                    && slot.selected_values().iter().all(|t| t.is_complete())
            })
    }

    fn resolve(self) -> Result<Self::Final, BlueprintError> {
        let mut bonus_skills = Vec::with_capacity(self.bonus_skills.len());
        for skill in self.bonus_skills {
            bonus_skills.extend(skill.resolve()?);
        }
        let mut traits = Vec::with_capacity(self.traits.len());
        for slot in self.traits {
            for trait_draft in slot.resolve()? {
                traits.push(trait_draft.resolve()?);
            }
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
                .map(|t| Slot::fixed(t.into()))
                .collect(),
            bonus_skills: value.bonus_skills.into_iter().map(Slot::fixed).collect(),
        }
    }
}
