use crate::ability::{Abilities, AbilityScoreIncrease};
use crate::class::ClassDto;
use crate::proficiency::Proficiency;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use crate::race::Race;

#[allow(dead_code)]
pub struct Sheet {
    permanent: PermanentSheet,
    playable: PlayableSheet,
}

#[allow(dead_code)]
pub struct PermanentSheet {
    abilities: Abilities,
    race: Race,
    class: ClassDto,
}

#[allow(dead_code)]
pub struct PlayableSheet {
    ability_score_increase: HashMap<String, AbilityScoreIncrease>,
    proficiencies: HashMap<String, Proficiency>,
}

pub struct SheetError {
    details: String,
}

impl Debug for SheetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl SheetError {
    pub fn new(details: impl ToString) -> Self {
        Self { details: details.to_string() }
    }
}
