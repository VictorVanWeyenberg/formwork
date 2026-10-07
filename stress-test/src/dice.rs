use std::fmt::{Display, Formatter};
use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug)]
pub enum DiceKind {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
    C,
}

impl Display for DiceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            DiceKind::D4 => "d4",
            DiceKind::D6 => "d6",
            DiceKind::D8 => "d8",
            DiceKind::D10 => "d10",
            DiceKind::D12 => "d12",
            DiceKind::D20 => "d20",
            DiceKind::D100 => "d100",
            DiceKind::C => "",
        })
    }
}

impl FromStr for DiceKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "d4" {
            Ok(DiceKind::D4)
        } else if s == "d6" {
            Ok(DiceKind::D6)
        } else if s == "d8" {
            Ok(DiceKind::D8)
        } else if s == "d10" {
            Ok(DiceKind::D10)
        } else if s == "d12" {
            Ok(DiceKind::D12)
        } else if s == "d20" {
            Ok(DiceKind::D20)
        } else if s == "d100" {
            Ok(DiceKind::D100)
        } else if s == "" {
            Ok(DiceKind::C)
        } else {
            Err(format!("Unknown die {}.", s))
        }
    }
}

impl DiceKind {
    pub fn as_number(&self) -> Option<usize> {
        match self {
            DiceKind::D4 => Some(4),
            DiceKind::D6 => Some(6),
            DiceKind::D8 => Some(8),
            DiceKind::D10 => Some(10),
            DiceKind::D12 => Some(12),
            DiceKind::D20 => Some(20),
            DiceKind::D100 => Some(100),
            DiceKind::C => None,
        }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Dice {
    amount: usize,
    dice_type: DiceKind,
}

impl Display for Dice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.amount, self.dice_type)
    }
}

impl FromStr for Dice {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dice_regex = Regex::new("^(?<amount>\\d+)(?<dice_type>(d\\d+)?)$").expect("");
        if let Some(captures) = dice_regex.captures(s) {
            let amount = (&captures["amount"]).parse::<usize>().unwrap();
            let dice_type = DiceKind::from_str(&captures["dice_type"])?;
            Ok(Dice { amount, dice_type })
        } else {
            Err(format!("Unable to convert \"{}\" to dice.", s))
        }
    }
}

impl Serialize for Dice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for Dice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Dice::from_str(&String::deserialize(deserializer)?).expect(""))
    }
}

impl Dice {
    pub fn new(amount: usize, dice_type: DiceKind) -> Dice {
        Dice { amount, dice_type }
    }
    pub fn amount(&self) -> &usize {
        &self.amount
    }
    pub fn dice_type(&self) -> &DiceKind {
        &self.dice_type
    }
}