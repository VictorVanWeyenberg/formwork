use serde::{Deserialize, Serialize};

/// Leaf domain type for D&D 5e traits: Effect.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Proficiency { item: String },
    AbilityBonus { ability: String, bonus: i32 },
    Darkvision { range_feet: u32 },
    Resistance { damage_type: String },
}

impl Effect {
    pub fn proficiency(item: impl Into<String>) -> Self {
        Self::Proficiency { item: item.into() }
    }

    pub fn ability_bonus(ability: impl Into<String>, bonus: i32) -> Self {
        Self::AbilityBonus {
            ability: ability.into(),
            bonus,
        }
    }

    pub fn darkvision(range_feet: u32) -> Self {
        Self::Darkvision { range_feet }
    }

    pub fn resistance(damage_type: impl Into<String>) -> Self {
        Self::Resistance {
            damage_type: damage_type.into(),
        }
    }
}
