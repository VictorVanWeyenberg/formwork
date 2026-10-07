use crate::effect::{Effect, SheetOperation};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Trait {
        title: String,
        description: String,
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
