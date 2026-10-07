use super::effect::Effect;
use crate::blueprint_trait::{Blueprint, Level, Walkable};
use crate::choice::Choice;
use crate::error::BlueprintError;
use crate::slot::Slot;
use crate::visitor::{ChoiceVisitor, VisitorFlow};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

pub trait TraitMode {
    type EffectItem: Clone + std::fmt::Debug + PartialEq + Eq + serde::Serialize + serde::de::DeserializeOwned;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitDraft;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitFinal;

impl TraitMode for TraitDraft {
    type EffectItem = Slot<Effect>;
}

impl TraitMode for TraitFinal {
    type EffectItem = Effect;
}

pub trait PushEffectChoice<M: TraitMode> {
    fn push_choice(effects: &mut Vec<<M as TraitMode>::EffectItem>, choice: Choice<Effect>);
}

impl PushEffectChoice<TraitDraft> for TraitDraft {
    fn push_choice(effects: &mut Vec<Slot<Effect>>, choice: Choice<Effect>) {
        effects.push(Slot::Choose(choice));
    }
}

pub type TraitBlueprint = Trait<TraitDraft>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TypedBuilder)]
#[builder(mutators(
    pub fn with_effect(&mut self, effect: impl Into<<M as TraitMode>::EffectItem>) {
        self.effects.push(effect.into());
    }
    pub fn with_effect_choice(&mut self, choice: Choice<Effect>)
    where
        M: PushEffectChoice<M>,
    {
        M::push_choice(&mut self.effects, choice);
    }
))]
#[serde(bound = "")]
pub struct Trait<M: TraitMode = TraitFinal> {
    #[builder(setter(into))]
    pub title: String,
    #[builder(setter(into))]
    pub description: String,
    #[builder(default, via_mutators)]
    pub effects: Vec<<M as TraitMode>::EffectItem>,
}

impl Trait<TraitDraft> {
    pub fn draft<I, E>(
        title: impl Into<String>,
        description: impl Into<String>,
        effects: I,
    ) -> Self
    where
        I: IntoIterator<Item = E>,
        E: Into<Slot<Effect>>,
    {
        Self {
            title: title.into(),
            description: description.into(),
            effects: effects.into_iter().map(Into::into).collect(),
        }
    }
}

impl<V> Walkable<V> for Trait<TraitDraft>
where
    V: ChoiceVisitor<Effect>,
{
    fn walk_level(&mut self, visitor: &mut V, depth: usize) -> Level {
        if depth == 0 {
            for effect_slot in &mut self.effects {
                if let Slot::Choose(choice) = effect_slot {
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

impl Blueprint for Trait<TraitDraft> {
    type Final = Trait<TraitFinal>;

    fn is_complete(&self) -> bool {
        self.effects.iter().all(|s| s.is_complete())
    }

    fn resolve(self) -> Result<Self::Final, BlueprintError> {
        let mut effects = Vec::with_capacity(self.effects.len());
        for slot in self.effects {
            effects.push(slot.resolve()?);
        }
        Ok(Trait {
            title: self.title,
            description: self.description,
            effects,
        })
    }
}

impl From<Trait<TraitFinal>> for Trait<TraitDraft> {
    fn from(value: Trait<TraitFinal>) -> Self {
        Trait {
            title: value.title,
            description: value.description,
            effects: value.effects.into_iter().map(Slot::Fixed).collect(),
        }
    }
}
