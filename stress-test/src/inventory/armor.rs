use crate::ability::Ability;
use crate::contentref::ArmorRef;
use crate::inventory::item::Weight;
use crate::inventory::wallet::Cost;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Armor {
    #[serde_as(as = "DisplayFromStr")]
    id: ArmorRef,
    name: String,
    description: String,
    cost: Cost,
    weight: Weight,
    category: ArmorCategory,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    strength: Option<usize>,
    stealth_disadvantage: bool,
    armor_class_requirements: ArmorClassRequirements,
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct ArmorClassRequirements {
    armor_class: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    add_modifier: Option<Ability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    limit_modifier: Option<usize>,
}

impl ArmorClassRequirements {
    pub fn new(
        armor_class: usize,
        add_modifier: Option<Ability>,
        limit_modifier: Option<usize>,
    ) -> ArmorClassRequirements {
        ArmorClassRequirements {
            armor_class,
            add_modifier,
            limit_modifier,
        }
    }

    pub fn base_armor_class(&self) -> &usize {
        &self.armor_class
    }
    pub fn add_modifier(&self) -> &Option<Ability> {
        &self.add_modifier
    }
    pub fn limit_modifier(&self) -> &Option<usize> {
        &self.limit_modifier
    }
}

impl Armor {
    pub fn new(
        id: ArmorRef,
        name: String,
        description: String,
        cost: Cost,
        weight: Weight,
        category: ArmorCategory,
        strength: Option<usize>,
        stealth_disadvantage: bool,
        armor_class_requirements: ArmorClassRequirements,
    ) -> Armor {
        Armor {
            id,
            name,
            description,
            cost,
            weight,
            category,
            strength,
            stealth_disadvantage,
            armor_class_requirements,
        }
    }
    pub fn id(&self) -> &ArmorRef {
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
    pub fn category(&self) -> &ArmorCategory {
        &self.category
    }
    pub fn is_shield(&self) -> bool {
        self.category == ArmorCategory::Shield
    }
    pub fn strength(&self) -> &Option<usize> {
        &self.strength
    }
    pub fn stealth_disadvantage(&self) -> &bool {
        &self.stealth_disadvantage
    }
    pub fn armor_class_requirements(&self) -> &ArmorClassRequirements {
        &self.armor_class_requirements
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum ArmorCategory {
    Light,
    Medium,
    Heavy,
    Shield,
}
