use std::fmt::{Display, Formatter};
use crate::sheet::SheetError;
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;

#[derive(Deserialize, Serialize)]
pub struct Abilities {
    strength: AbilityScore,
    dexterity: AbilityScore,
    constitution: AbilityScore,
    intelligence: AbilityScore,
    wisdom: AbilityScore,
    charisma: AbilityScore,
}

impl Abilities {
    pub fn new(
        strength: usize,
        dexterity: usize,
        constitution: usize,
        intelligence: usize,
        wisdom: usize,
        charisma: usize,
    ) -> Result<Self, SheetError> {
        Ok(Self {
            strength: AbilityScore::new(strength)?,
            dexterity: AbilityScore::new(dexterity)?,
            constitution: AbilityScore::new(constitution)?,
            intelligence: AbilityScore::new(intelligence)?,
            wisdom: AbilityScore::new(wisdom)?,
            charisma: AbilityScore::new(charisma)?,
        })
    }

    pub fn score_for(&self, ability: &Ability) -> &AbilityScore {
        match ability {
            Ability::Strength => &self.strength,
            Ability::Dexterity => &self.dexterity,
            Ability::Constitution => &self.constitution,
            Ability::Intelligence => &self.intelligence,
            Ability::Wisdom => &self.wisdom,
            Ability::Charisma => &self.charisma,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Display for Ability {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Ability::Strength => write!(f, "Strength"),
            Ability::Dexterity => write!(f, "Dexterity"),
            Ability::Constitution => write!(f, "Constitution"),
            Ability::Intelligence => write!(f, "Intelligence"),
            Ability::Wisdom => write!(f, "Wisdom"),
            Ability::Charisma => write!(f, "Charisma"),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AbilityScoreIncrease {
    Racial {
        increase: isize,
        ability: Ability,
    },
    Class {
        increase: isize,
        ability: Ability,
        may_exceed: bool,
        level: usize,
    },
}

impl AbilityScoreIncrease {
    pub fn racial(increase: isize, ability: Ability) -> AbilityScoreIncrease {
        AbilityScoreIncrease::Racial { increase, ability }
    }

    pub fn ability(&self) -> &Ability {
        match self {
            AbilityScoreIncrease::Racial { ability, .. } => ability,
            AbilityScoreIncrease::Class { ability, .. } => ability,
        }
    }

    pub fn increase(&self) -> &isize {
        match self {
            AbilityScoreIncrease::Racial { increase, .. } => increase,
            AbilityScoreIncrease::Class { increase, .. } => increase,
        }
    }

    pub fn may_exceed(&self) -> bool {
        match self {
            AbilityScoreIncrease::Racial { .. } => false,
            AbilityScoreIncrease::Class { may_exceed, .. } => *may_exceed,
        }
    }

    pub fn level(&self) -> usize {
        match self {
            AbilityScoreIncrease::Racial { .. } => 0,
            AbilityScoreIncrease::Class { level, .. } => *level,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct AbilityScore {
    score: usize,
}

impl AbilityScore {
    pub fn new(score: usize) -> Result<AbilityScore, SheetError> {
        if score == 0 {
            Err(SheetError::new("Ability score cannot be zero."))
        } else if score > 30 {
            Err(SheetError::new("Ability score cannot exceed 20."))
        } else {
            Ok(AbilityScore { score })
        }
    }

    pub fn score(&self) -> usize {
        self.score
    }

    pub fn modifier(&self) -> isize {
        (self.score as isize) / 2 - 5
    }
}

impl AddAssign<&AbilityScoreIncrease> for AbilityScore {
    fn add_assign(&mut self, rhs: &AbilityScoreIncrease) {
        let score = self.score;
        let increase = *rhs.increase();
        let increased = score as isize + increase;
        if increased > 20 && !rhs.may_exceed() || increased < 0 || increased > 30 {
            return;
        }
        self.score = increased as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::ability::AbilityScore;

    #[test]
    fn modifier() {
        let mut modifier = -5isize;
        for score in 1..=30 {
            if score % 2 == 0 {
                modifier += 1;
            }
            assert_eq!(AbilityScore::new(score).expect("").modifier(), modifier);
        }
    }
}
