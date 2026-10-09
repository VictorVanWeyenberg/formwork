use crate::contentref::{RaceRef, Referable, SubraceRef};
use crate::traits::{Trait, TraitUnresolved, TraitResolved};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Display, Formatter};
use schematic::schematic;

#[schematic(builder, derives(Clone, PartialEq, Serialize, Deserialize))]
#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct Race {
    #[serde_as(as = "DisplayFromStr")]
    id: RaceRef,
    name: String,
    description: String,
    #[slot(collection, cascade)]
    pub (crate) traits: Vec<Trait>,
}

impl Referable for Race {
    type Ref = RaceRef;

    fn reference(&self) -> &Self::Ref {
        &self.id
    }
}

impl Race {
    pub fn new(id: RaceRef, name: String, description: String, traits: Vec<Trait>) -> Self {
        Self { id, name, description, traits }
    }
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Subrace {
    #[serde_as(as = "DisplayFromStr")]
    id: SubraceRef,
        name: String,
        description: String,
    #[serde_as(as = "DisplayFromStr")]
        for_race: RaceRef,
        pub (crate) traits: Vec<Trait>,
}

impl Referable for Subrace {
    type Ref = SubraceRef;
    fn reference(&self) -> &Self::Ref {
        &self.id
    }
}

impl Subrace {
    pub fn new(id: SubraceRef, name: String, description: String, for_race: RaceRef, traits: Vec<Trait>) -> Self {
        Self { id, name, description, for_race, traits }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Size {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Size::Tiny => "Tiny",
            Size::Small => "Small",
            Size::Medium => "Medium",
            Size::Large => "Large",
            Size::Huge => "Huge",
            Size::Gargantuan => "Gargantuan",
        })
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Speed(usize);

impl From<usize> for Speed {
    fn from(speed: usize) -> Self {
        Self(speed)
    }
}

impl Display for Speed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Vision {
    Blindsight,
    Darkvision,
    Truesight,
}