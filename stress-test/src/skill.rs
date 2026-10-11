use crate::ability::Ability;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Skill {
    Acrobatics,
    AnimalHandling,
    Arcana,
    Athletics,
    Deception,
    History,
    Insight,
    Intimidation,
    Investigation,
    Medicine,
    Nature,
    Perception,
    Performance,
    Persuasion,
    Religion,
    SleightOfHand,
    Stealth,
    Survival,
}

impl Skill {
    pub fn ability(&self) -> Ability {
        match self {
            Skill::Athletics => Ability::Strength,
            Skill::Acrobatics | Skill::SleightOfHand | Skill::Stealth => Ability::Dexterity,
            Skill::Arcana | Skill::History | Skill::Investigation | Skill::Religion => {
                Ability::Intelligence
            }
            Skill::AnimalHandling
            | Skill::Insight
            | Skill::Medicine
            | Skill::Nature
            | Skill::Perception
            | Skill::Survival => Ability::Wisdom,
            Skill::Deception | Skill::Intimidation | Skill::Performance | Skill::Persuasion => {
                Ability::Charisma
            }
        }
    }
}

impl Display for Skill {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Skill::Acrobatics => "Acrobatics",
                Skill::AnimalHandling => "Animal Handling",
                Skill::Arcana => "Arcana",
                Skill::Athletics => "Athletics",
                Skill::Deception => "Deception",
                Skill::History => "History",
                Skill::Insight => "Insight",
                Skill::Intimidation => "Intimidation",
                Skill::Investigation => "Investigation",
                Skill::Medicine => "Medicine",
                Skill::Nature => "Nature",
                Skill::Perception => "Perception",
                Skill::Performance => "Performance",
                Skill::Persuasion => "Persuasion",
                Skill::Religion => "Religion",
                Skill::SleightOfHand => "Sleight of hand",
                Skill::Stealth => "Stealth",
                Skill::Survival => "Survival",
            }
        )
    }
}
