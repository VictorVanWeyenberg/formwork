use crate::dice::Dice;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub enum DamageType {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct DamageDice {
    dice: Dice,
    damage_type: DamageType,
}

impl DamageDice {
    pub fn new(dice: Dice, damage_type: DamageType) -> DamageDice {
        DamageDice { dice, damage_type }
    }
    pub fn dice(&self) -> &Dice {
        &self.dice
    }
    pub fn damage_type(&self) -> &DamageType {
        &self.damage_type
    }
}

#[allow(dead_code)]
pub struct Damage {
    points: usize,
    damage_type: DamageType,
}

impl Damage {
    pub fn new(points: usize, damage_type: DamageType) -> Self {
        Self {
            points,
            damage_type,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "damage_type")]
pub enum DamageModulation {
    Vulnerability(DamageType),
    Resistance(DamageType),
    Immunity(DamageType),
}
