use crate::effect::Effect;
use schematic::schematic;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[schematic(builder, derives(Clone, Serialize, Deserialize, PartialEq))]
#[serde_as]
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Trait {
    title: String,
    description: String,
    #[slot(collection)]
    effects: Vec<Effect>,
}

impl Trait {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn add_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}
