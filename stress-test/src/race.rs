use crate::contentref::{RaceRef, Referable, SubraceRef};
use crate::traits::{Trait, TraitResolved, TraitUnresolved};
use schematic::schematic;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::fmt::{Display, Formatter};

/// A playable character race in D&D 5e (e.g. Dwarf, Elf, Human).
///
/// Models the top-level racial identity. Traits are modeled as a cascading collection slot
/// (`#[slot(collection, cascade)]`), allowing traits to contain choices that cascade downward.
#[schematic(builder, derives(Clone, Debug, PartialEq, Serialize, Deserialize))]
#[serde_as]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Race {
    /// Canonical content reference identifier for this race.
    #[serde_as(as = "DisplayFromStr")]
    id: RaceRef,
    /// Display name of the race.
    name: String,
    /// Flavor and rules description.
    description: String,
    /// Collection of racial traits granted by this race, cascading to child trait schematics.
    #[slot(collection, cascade)]
    pub(crate) traits: Vec<Trait>,
}

impl Referable for Race {
    type Ref = RaceRef;

    fn reference(&self) -> &Self::Ref {
        &self.id
    }
}

impl Race {
    pub fn new(id: RaceRef, name: String, description: String, traits: Vec<Trait>) -> Self {
        Self {
            id,
            name,
            description,
            traits,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn traits(&self) -> &[Trait] {
        &self.traits
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Subrace {
    #[serde_as(as = "DisplayFromStr")]
    id: SubraceRef,
    name: String,
    description: String,
    #[serde_as(as = "DisplayFromStr")]
    for_race: RaceRef,
    pub(crate) traits: Vec<Trait>,
}

impl Referable for Subrace {
    type Ref = SubraceRef;
    fn reference(&self) -> &Self::Ref {
        &self.id
    }
}

impl Subrace {
    pub fn new(
        id: SubraceRef,
        name: String,
        description: String,
        for_race: RaceRef,
        traits: Vec<Trait>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            for_race,
            traits,
        }
    }

    pub fn id(&self) -> &SubraceRef {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn for_race(&self) -> &RaceRef {
        &self.for_race
    }

    pub fn traits(&self) -> &[Trait] {
        &self.traits
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
        write!(
            f,
            "{}",
            match self {
                Size::Tiny => "Tiny",
                Size::Small => "Small",
                Size::Medium => "Medium",
                Size::Large => "Large",
                Size::Huge => "Huge",
                Size::Gargantuan => "Gargantuan",
            }
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Vision {
    Blindsight,
    Darkvision,
    Truesight,
}
