use crate::effect::Effect;
use crate::effect::{EffectResolved, EffectUnresolved};
use schematic::schematic;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// A character trait or racial feature (e.g. Darkvision, Tool Proficiency).
///
/// Traits group one or more mechanical effects together under a common title and description.
/// The `effects` field is a cascading collection slot (`#[slot(collection, cascade)]`),
/// allowing child effects with choices to be resolved hierarchically.
#[schematic(builder, derives(Clone, Debug, Serialize, Deserialize, PartialEq))]
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trait {
    /// Human-readable title of the trait.
    title: String,
    /// Detailed description of the trait.
    description: String,
    /// Cascading collection of concrete effects granted by this trait.
    #[slot(collection, cascade)]
    effects: Vec<Effect>,
}

impl Trait {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}
