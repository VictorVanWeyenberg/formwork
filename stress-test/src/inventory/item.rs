use std::hash::Hash;

use regex::Regex;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::inventory::wallet::Cost;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Weight {
    weight: usize,
    denominator: Option<usize>,
}

impl From<usize> for Weight {
    fn from(weight: usize) -> Self {
        Weight {
            weight,
            denominator: None,
        }
    }
}

impl From<(usize, usize)> for Weight {
    fn from((weight, denominator): (usize, usize)) -> Self {
        if denominator == 1 {
            Weight {
                weight,
                denominator: None,
            }
        } else {
            Weight {
                weight,
                denominator: Some(denominator),
            }
        }
    }
}

impl Serialize for Weight {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let weight = self.weight;
        if let Some(denominator) = self.denominator {
            if denominator == 1 {
                serializer.serialize_str(&format!("{weight} lb."))
            } else {
                serializer.serialize_str(&format!("{weight}/{denominator} lb."))
            }
        } else {
            serializer.serialize_str(&format!("{weight} lb."))
        }
    }
}

impl<'de> Deserialize<'de> for Weight {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let weight_regex = Regex::new("^(?<weight>\\d+)(/(?<denominator>\\d+))?\\slb\\.$").unwrap();
        let weight_str = String::deserialize(deserializer).unwrap();
        let (weight, denominator) = if let Some(captures) = weight_regex.captures(&weight_str) {
            let weight = (&captures["weight"]).parse::<usize>().unwrap();
            let denominator = captures
                .name("denominator")
                .map(|m| m.as_str())
                .map(|s| s.parse::<usize>().unwrap());
            (weight, denominator)
        } else {
            return Err(Error::custom("Unable to parse weight."));
        };
        if let Some(denominator) = denominator {
            Ok((weight, denominator).into())
        } else {
            Ok(weight.into())
        }
    }
}

impl Weight {
    pub fn new(weight: usize) -> Self {
        Self {
            weight,
            denominator: None,
        }
    }

    pub fn fractional(weight: usize, denominator: usize) -> Self {
        Self {
            weight,
            denominator: Some(denominator),
        }
    }

    pub fn weight(&self) -> &usize {
        &self.weight
    }

    pub fn denominator(&self) -> &Option<usize> {
        &self.denominator
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Item {
    name: String,
    description: String,
    cost: Cost,
    weight: Weight,
}

impl Item {
    pub fn new(name: String, description: String, cost: Cost, weight: Weight) -> Item {
        Item {
            name,
            description,
            cost,
            weight,
        }
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
}
