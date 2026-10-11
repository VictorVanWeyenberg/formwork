use crate::ability::Ability;
use crate::condition::Condition;
use crate::damage::{DamageDice, DamageType};
use crate::inventory::weapon::Weapon;
use crate::skill::Skill;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub enum Roll {
    SkillCheck {
        skill: Skill,
        ability: Option<Ability>,
    },
    WeaponAttack {
        weapon: Weapon,
        ability: Ability,
        target: Uuid,
    },
    SavingThrow {
        ability: Ability,
        against: Option<SavingThrowAgainst>,
        cause: Uuid,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SavingThrowAgainst {
    DamageType(DamageType),
    Condition(Condition),
}

pub enum Rolled {
    SkillCheck(SkillCheck),
    WeaponAttack(WeaponAttack),
    SavingThrow(SavingThrow),
}

pub struct SkillCheck {
    skill: Skill,
    ability: Ability,
    advantage: VantageStatus,
    ability_modifier: isize,
    proficiency_bonus: Option<u8>,
    expertise: bool,
}

impl SkillCheck {
    pub fn new(
        skill: Skill,
        ability: Ability,
        advantage: VantageStatus,
        ability_modifier: isize,
        proficiency_bonus: Option<u8>,
        expertise: bool,
    ) -> Self {
        Self {
            skill,
            ability,
            advantage,
            ability_modifier,
            proficiency_bonus,
            expertise,
        }
    }
    pub fn skill(&self) -> &Skill {
        &self.skill
    }
    pub fn ability(&self) -> &Ability {
        &self.ability
    }
    pub fn advantage(&self) -> &VantageStatus {
        &self.advantage
    }
    pub fn ability_modifier(&self) -> &isize {
        &self.ability_modifier
    }
    pub fn proficiency_bonus(&self) -> &Option<u8> {
        &self.proficiency_bonus
    }
    pub fn expertise(&self) -> &bool {
        &self.expertise
    }
}

pub struct WeaponAttack {
    ability: Ability,
    weapon: Weapon,
    advantage_status: VantageStatus,
    ability_modifier: isize,
    proficiency_bonus: Option<u8>,
    on_hit: OnHit,
    target: Uuid,
}

impl WeaponAttack {
    pub fn new(
        ability: Ability,
        weapon: Weapon,
        advantage_status: VantageStatus,
        ability_modifier: isize,
        proficiency_bonus: Option<u8>,
        on_hit: OnHit,
        target: Uuid,
    ) -> Self {
        WeaponAttack {
            ability,
            weapon,
            advantage_status,
            ability_modifier,
            proficiency_bonus,
            on_hit,
            target,
        }
    }
    pub fn ability(&self) -> &Ability {
        &self.ability
    }
    pub fn weapon(&self) -> &Weapon {
        &self.weapon
    }
    pub fn advantage_status(&self) -> &VantageStatus {
        &self.advantage_status
    }
    pub fn ability_modifier(&self) -> &isize {
        &self.ability_modifier
    }
    pub fn proficiency_bonus(&self) -> &Option<u8> {
        &self.proficiency_bonus
    }
    pub fn on_hit(&self) -> &OnHit {
        &self.on_hit
    }
    pub fn target(&self) -> Uuid {
        self.target
    }
}

pub struct OnHit {
    damage_dice: Option<DamageDice>,
    weapon_bonus: usize,
    ability_modifier: isize,
}

impl OnHit {
    pub fn new(
        ability_modifier: isize,
        damage_dice: Option<DamageDice>,
        weapon_bonus: usize,
    ) -> Self {
        Self {
            ability_modifier,
            damage_dice,
            weapon_bonus,
        }
    }
    pub fn damage_dice(&self) -> &Option<DamageDice> {
        &self.damage_dice
    }
    pub fn weapon_bonus(&self) -> &usize {
        &self.weapon_bonus
    }
    pub fn ability_modifier(&self) -> &isize {
        &self.ability_modifier
    }
}

pub struct SavingThrow {
    ability: Ability,
    advantage: VantageStatus,
    ability_modifier: isize,
    proficiency_bonus: Option<u8>,
    cause: Uuid,
}

impl SavingThrow {
    pub fn new(
        ability: Ability,
        advantage: VantageStatus,
        ability_modifier: isize,
        proficiency_bonus: Option<u8>,
        cause: &Uuid,
    ) -> Self {
        Self {
            ability,
            advantage,
            ability_modifier,
            proficiency_bonus,
            cause: *cause,
        }
    }
    pub fn ability(&self) -> &Ability {
        &self.ability
    }
    pub fn advantage(&self) -> &VantageStatus {
        &self.advantage
    }
    pub fn ability_modifier(&self) -> &isize {
        &self.ability_modifier
    }
    pub fn proficiency_bonus(&self) -> &Option<u8> {
        &self.proficiency_bonus
    }
    pub fn targetor(&self) -> Uuid {
        self.cause
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VantageStatus {
    Disadvantage,
    None,
    Advantage,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RollDescription {
    SkillCheck(Skill),
    SavingThrow(SavingThrowKind),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SavingThrowKind {
    Ability(Ability),
    Against(SavingThrowAgainst),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VantageOnRoll {
    status: VantageStatus,
    roll: RollDescription,
}

impl VantageOnRoll {
    pub fn new(status: VantageStatus, roll: RollDescription) -> Self {
        Self { status, roll }
    }
}
