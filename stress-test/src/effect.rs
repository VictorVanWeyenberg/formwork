use crate::ability::AbilityScoreIncrease;
use crate::condition::Condition;
use crate::damage::DamageModulation;
use crate::proficiency::Proficiency;
use crate::race::{Size, Speed, Vision};
use crate::roll::VantageOnRoll;
use crate::valueref::ValueRef;
use schematic::schematic;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum EffectOperation {
    AddEffect(Vec<Effect>),
    RemoveEffect(EffectId),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
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

/// A discrete mechanical modification applied to a character sheet.
///
/// `Effect` serves as the leaf node in the schematic tree. Its `operation` field is a single
/// slot (`#[slot]`), allowing it to hold either a fixed resolved operation (such as +2 Constitution)
/// or a choice (such as "Choose one tool proficiency from Brewer's Supplies, Mason's Tools, Smith's Tools").
#[schematic(builder, derives(Clone, Debug, Serialize, Deserialize, PartialEq))]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Effect {
    /// Optional identifier for tracking or removing effects.
    #[serde(skip_serializing_if = "EffectId::is_empty")]
    #[serde(default)]
    id: EffectId,
    /// The character modification operation, stored in a single slot.
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
        Self { id, operation }
    }

    pub fn id(&self) -> &EffectId {
        &self.id
    }

    pub fn operation(&self) -> &SheetOperation {
        &self.operation
    }
}

/// The specific character sheet operation performed by an [`Effect`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    ModifyMaxHitPoint(ValueRef),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Rule {
    /// When heavy armor has a strength requirement, and the wearer's strength score is below the required value, their strength is reduced by 10 feet.
    HeavyArmorSpeedPenalty,
}
