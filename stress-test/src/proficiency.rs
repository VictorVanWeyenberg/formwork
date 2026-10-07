use crate::ability::Ability;
use crate::contentref::{ArmorRef, ToolRef, WeaponRef};
use crate::inventory::armor::ArmorCategory;
use crate::inventory::weapon::{WeaponCategory, WeaponRange, WeaponSimplicity};
use crate::skill::Skill;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "proficiency_kind", content = "proficient_with")]
#[serde(rename_all = "camelCase")]
pub enum Proficiency {
    Armor(#[serde_as(as = "DisplayFromStr")] ArmorRef),
    ArmorCategory(ArmorCategory),
    Conditional {
        proficiency: Box<Proficiency>,
        condition: String,
    },
    SavingThrow(Ability),
    Skill {
        skill: Skill,
        expertise: bool,
    },
    Tool(#[serde_as(as = "DisplayFromStr")] ToolRef),
    Weapon(#[serde_as(as = "DisplayFromStr")] WeaponRef),
    WeaponCategory(WeaponCategory),
    WeaponRange(WeaponRange),
    WeaponSimplicity(WeaponSimplicity),
}

impl From<ArmorRef> for Proficiency {
    fn from(armor: ArmorRef) -> Self {
        Proficiency::Armor(armor)
    }
}

impl From<ToolRef> for Proficiency {
    fn from(tool: ToolRef) -> Self {
        Proficiency::Tool(tool)
    }
}

impl From<WeaponRef> for Proficiency {
    fn from(weapon: WeaponRef) -> Self {
        Proficiency::Weapon(weapon)
    }
}

impl From<ArmorCategory> for Proficiency {
    fn from(armor_category: ArmorCategory) -> Self {
        Proficiency::ArmorCategory(armor_category)
    }
}

impl From<Skill> for Proficiency {
    fn from(skill: Skill) -> Self {
        Proficiency::Skill {
            skill,
            expertise: false,
        }
    }
}

impl From<&Skill> for Proficiency {
    fn from(skill: &Skill) -> Self {
        Proficiency::Skill {
            skill: skill.clone(),
            expertise: false,
        }
    }
}

impl Proficiency {
    pub fn for_skill(&self, skill: &Skill) -> bool {
        match self {
            Proficiency::Skill {
                skill: proficient_skill,
                ..
            } => proficient_skill == skill,
            _ => false,
        }
    }

    pub fn expertise(&self, skill: &Skill) -> bool {
        match self {
            Proficiency::Skill {
                skill: proficient_skill,
                expertise,
            } => proficient_skill == skill && *expertise,
            _ => false,
        }
    }

    pub fn for_saving_throw(&self, ability: &Ability) -> bool {
        match self {
            Proficiency::SavingThrow(proficient_ability) => proficient_ability == ability,
            _ => false,
        }
    }
}
