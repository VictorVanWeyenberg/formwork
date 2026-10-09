use crate::ability::AbilityScoreIncrease;
use crate::condition::Condition;
use crate::damage::DamageModulation;
use crate::proficiency::Proficiency;
use crate::race::{Size, Speed, Vision};
use crate::roll::VantageOnRoll;
use serde::{Deserialize, Serialize};
use schematic::schematic;
use crate::valueref::ValueRef;

#[derive(Serialize, Deserialize)]
pub enum EffectOperation {
    AddEffect(Vec<Effect>),
    RemoveEffect(EffectId),
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct EffectId(String);

impl From<&str> for EffectId {
    fn from(id: &str) -> Self {
        Self(String::from(id))
    }
}

impl EffectId {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[schematic(builder, derives(Clone, Serialize, Deserialize, PartialEq))]
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Effect {
    #[serde(skip_serializing_if = "EffectId::is_empty")]
    #[serde(default)]
    id: EffectId,
    #[serde(flatten)]
    #[slot]
    operation: SheetOperation,
}

impl Effect {
    pub fn new(operation: SheetOperation) -> Self {
        Self {
            id: EffectId::default(),
            operation,
        }
    }

    pub fn with_id(id: EffectId, operation: SheetOperation) -> Self {
        Self {
            id,
            operation,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "operation", content = "argument")]
#[serde(rename_all = "camelCase")]
pub enum SheetOperation {
    Proficiency(Proficiency),
    Condition(Condition),
    AbilityScoreIncrease(AbilityScoreIncrease),
    RaceSize(Size),
    RaceWalkingSpeed(Speed),
    Vision(Vision),
    VantageOnRoll(VantageOnRoll),
    IgnoreRule(Rule),
    DamageModulation(DamageModulation),
    ModifyMaxHitPoint(ValueRef)
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Rule {
    /// When heavy armor has a strength requirement, and the wearer's strength score is below the required value, their strength is reduced by 10 feet.
    HeavyArmorSpeedPenalty,
}
