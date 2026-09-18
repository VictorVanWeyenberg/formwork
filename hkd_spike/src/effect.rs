//! Leaf domain — stand-ins for `fifth_domain::{effect, proficiency, race, roll, ...}`, using the
//! T6 representation: externally tagged enums (serde default) and a flattened `SheetOperation`.

use crate::blueprint::{Blueprint, ChoiceError, ChoicePrompt};
use serde::{Deserialize, Serialize};
use std::ops::Not;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EffectId(String);

impl EffectId {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for EffectId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    #[serde(default, skip_serializing_if = "EffectId::is_empty")]
    id: EffectId,
    #[serde(flatten)]
    operation: SheetOperation,
}

impl Effect {
    pub fn new(operation: SheetOperation) -> Self {
        Self { id: EffectId::default(), operation }
    }

    pub fn with_id(id: impl Into<EffectId>, operation: SheetOperation) -> Self {
        Self { id: id.into(), operation }
    }

    pub fn operation(&self) -> &SheetOperation {
        &self.operation
    }
}

/// Leaves are already final.
impl Blueprint for Effect {
    type Final = Effect;

    fn resolve(self) -> Result<Effect, ChoiceError> {
        Ok(self)
    }

    fn visible_choices(&self) -> Vec<&dyn ChoicePrompt> {
        Vec::new()
    }

    fn next_open_choice_mut(&mut self) -> Option<&mut dyn ChoicePrompt> {
        None
    }
}

/// Every variant carries data: a requirement for `#[serde(flatten)]` on an externally tagged enum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SheetOperation {
    Proficiency(Proficiency),
    AbilityScoreIncrease(AbilityScoreIncrease),
    RaceSize(Size),
    RaceWalkingSpeed(Speed),
    Vision(Vision),
    IgnoreRule(Rule),
    DamageModulation(DamageModulation),
    ModifyMaxHitPoint(ValueRef),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Proficiency {
    /// `String` stands in for `ToolRef` (`DisplayFromStr`).
    Tool(String),
    Weapon(String),
    SavingThrow(Ability),
    Skill {
        skill: Skill,
        #[serde(default, skip_serializing_if = "Not::not")]
        expertise: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Ability {
    pub const ALL: [Ability; 6] = [
        Ability::Strength,
        Ability::Dexterity,
        Ability::Constitution,
        Ability::Intelligence,
        Ability::Wisdom,
        Ability::Charisma,
    ];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Skill {
    Perception,
    Stealth,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AbilityScoreIncrease {
    Racial { increase: isize, ability: Ability },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Size {
    Small,
    Medium,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Speed(pub usize);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Vision {
    Darkvision,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Rule {
    HeavyArmorSpeedPenalty,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DamageModulation {
    Resistance(DamageType),
    Immunity(DamageType),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DamageType {
    Acid,
    Fire,
    Poison,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ValueRef {
    Sheet(SheetRef),
    Const(isize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SheetRef {
    Level,
}
