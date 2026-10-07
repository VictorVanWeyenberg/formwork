use crate::ability::Ability;
use crate::spell::{CastingFrequency, Spell};

#[derive(Debug, PartialEq)]
pub struct PermanentSpell {
    spell: Spell,
    ability: Option<Ability>,
    as_ritual: bool,
    frequency: Option<CastingFrequency>,
    expended: bool,
}

impl PermanentSpell {
    pub fn new(spell: Spell) -> Self {
        Self {
            spell,
            ability: None,
            as_ritual: false,
            frequency: None,
            expended: false,
        }
    }

    pub fn with_ability(mut self, ability: Ability) -> Self {
        self.ability = Some(ability);
        self
    }

    pub fn with_as_ritual(mut self) -> Self {
        self.as_ritual = true;
        self
    }

    pub fn with_frequency(mut self, frequency: CastingFrequency) -> Self {
        self.frequency = Some(frequency);
        self
    }

    pub fn expend(&mut self) {
        self.expended = true;
    }

    pub fn spell(&self) -> &Spell {
        &self.spell
    }
}