use crate::contentref::WeaponRef;
use crate::damage::DamageDice;
use crate::dice::Dice;
use crate::inventory::item::Weight;
use crate::inventory::wallet::Cost;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::fmt::{Display, Formatter};

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Weapon {
    #[serde_as(as = "DisplayFromStr")]
    id: WeaponRef,
    name: String,
    description: String,
    cost: Cost,
    weight: Weight,
    category: WeaponCategory,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    damage: Option<DamageDice>,
    properties: Vec<WeaponProperty>,
}

impl Display for Weapon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Weapon {
    pub fn new(
        id: WeaponRef,
        name: impl ToString,
        description: impl ToString,
        cost: Cost,
        weight: Weight,
        category: WeaponCategory,
        damage: Option<DamageDice>,
        properties: Vec<WeaponProperty>,
    ) -> Weapon {
        Weapon {
            id,
            name: name.to_string(),
            description: description.to_string(),
            cost,
            weight,
            category,
            damage,
            properties,
        }
    }
    pub fn id(&self) -> &WeaponRef {
        &self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn cost(&self) -> &Cost {
        &self.cost
    }
    pub fn weight(&self) -> &Weight {
        &self.weight
    }
    pub fn category(&self) -> &WeaponCategory {
        &self.category
    }
    pub fn damage(&self) -> &Option<DamageDice> {
        &self.damage
    }
    pub fn properties(&self) -> &Vec<WeaponProperty> {
        &self.properties
    }
}

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Debug)]
pub struct WeaponCategory {
    simplicity: WeaponSimplicity,
    range: WeaponRange,
}

impl WeaponCategory {
    pub fn new(simplicity: WeaponSimplicity, range: WeaponRange) -> WeaponCategory {
        WeaponCategory { simplicity, range }
    }

    pub fn has_simplicity(&self, simplicity: &WeaponSimplicity) -> bool {
        self.simplicity == *simplicity
    }

    pub fn has_range(&self, weapon_range: &WeaponRange) -> bool {
        self.range == *weapon_range
    }

    pub fn simplicity(&self) -> &WeaponSimplicity {
        &self.simplicity
    }

    pub fn range(&self) -> &WeaponRange {
        &self.range
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub enum WeaponSimplicity {
    Simple,
    Martial,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub enum WeaponRange {
    Melee,
    Ranged,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum WeaponProperty {
    Ammunition(AttackRange),
    Finesse,
    Heavy,
    Light,
    Loading,
    Reach,
    Special(String),
    Thrown(AttackRange),
    TwoHanded,
    Versatile(Dice),
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub struct AttackRange {
    normal: usize,
    long: usize,
}

impl AttackRange {
    pub fn new(normal: usize, long: usize) -> AttackRange {
        AttackRange { normal, long }
    }

    pub fn normal(&self) -> &usize {
        &self.normal
    }
    pub fn long(&self) -> &usize {
        &self.long
    }
}
