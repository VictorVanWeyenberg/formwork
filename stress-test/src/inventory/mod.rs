use crate::inventory::armor::Armor;
use crate::inventory::item::{Item, Weight};
use crate::inventory::tool::Tool;
use crate::inventory::wallet::Cost;
use crate::inventory::weapon::Weapon;
use serde::{Deserialize, Serialize};

pub mod armor;
pub mod item;
pub mod tool;
pub mod wallet;
pub mod weapon;

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Pack(Vec<Items>);

impl From<Vec<Items>> for Pack {
    fn from(value: Vec<Items>) -> Self {
        Pack(value)
    }
}

impl From<Items> for Pack {
    fn from(value: Items) -> Self {
        Pack(vec![value])
    }
}

impl Pack {
    pub fn new() -> Pack {
        Pack(vec![])
    }
    pub fn add(&mut self, other: Pack) {
        self.0.extend(other.0)
    }

    pub fn push(&mut self, item: Items) {
        self.0.push(item)
    }

    pub fn remove(&mut self, item: Items) -> Result<Items, String> {
        match self
            .0
            .iter()
            .position(|i| i == &item)
            .map(|index| self.0.remove(index))
        {
            None => Err(format!(
                "Unable to remove {} when you don't have any.",
                item.name(),
            )),
            Some(item) => Ok(item),
        }
    }

    pub fn amount(&self, item: &Items) -> usize {
        self.0.iter().filter(|i| *i == item).count()
    }

    pub fn contains(&self, predicate: impl Fn(&Items) -> bool) -> bool {
        self.0.iter().any(|item| predicate(item))
    }

    pub fn contains_amount(&self, predicate: impl Fn(&Items) -> bool, amount: usize) -> bool {
        self.0.iter().filter(|item| predicate(item)).count() == amount
    }

    pub fn drain(&mut self, pack: &mut Pack) {
        pack.0.append(&mut self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Items {
    Armor(Armor),
    Gear(Item),
    Tool(Tool),
    Weapon(Weapon),
}

impl Into<Items> for Armor {
    fn into(self) -> Items {
        Items::Armor(self)
    }
}

impl Into<Items> for Item {
    fn into(self) -> Items {
        Items::Gear(self)
    }
}

impl Into<Items> for Tool {
    fn into(self) -> Items {
        Items::Tool(self)
    }
}

impl Into<Items> for Weapon {
    fn into(self) -> Items {
        Items::Weapon(self)
    }
}

impl Items {
    pub fn name(&self) -> &String {
        match self {
            Items::Armor(armor) => armor.name(),
            Items::Gear(gear) => gear.name(),
            Items::Tool(tool) => tool.name(),
            Items::Weapon(weapon) => weapon.name(),
        }
    }

    pub fn description(&self) -> &String {
        match self {
            Items::Armor(armor) => armor.description(),
            Items::Gear(gear) => gear.description(),
            Items::Tool(tool) => tool.description(),
            Items::Weapon(weapon) => weapon.description(),
        }
    }

    pub fn cost(&self) -> &Cost {
        match self {
            Items::Armor(armor) => armor.cost(),
            Items::Gear(gear) => gear.cost(),
            Items::Tool(tool) => tool.cost(),
            Items::Weapon(weapon) => weapon.cost(),
        }
    }

    pub fn weight(&self) -> &Weight {
        match self {
            Items::Armor(armor) => armor.weight(),
            Items::Gear(gear) => gear.weight(),
            Items::Tool(tool) => tool.weight(),
            Items::Weapon(weapon) => weapon.weight(),
        }
    }
}

#[derive(Clone)]
pub enum Hand {
    Left,
    Right,
}
