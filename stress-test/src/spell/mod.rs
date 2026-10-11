#![allow(clippy::all)]

pub use casting_duration::*;
pub use casting_range::*;
pub use casting_time::*;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::hash::Hash;

use crate::rest::Rest;

mod casting_duration;
mod casting_range;
mod casting_time;
mod permanent_spell;
mod spell_list;

use crate::contentref::SpellRef;
pub use permanent_spell::*;
pub use spell_list::*;

#[serde_as]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Spell {
    #[serde_as(as = "DisplayFromStr")]
    id: SpellRef,
    name: String,
    description: String,
    level: SpellLevel,
    components: CastingComponents,
    #[serde_as(as = "DisplayFromStr")]
    time: CastingTime,
    range: CastingRange,
    #[serde_as(as = "DisplayFromStr")]
    duration: SpellDuration,
    school: MageSchool,
    ritual: bool,
}

impl Spell {
    pub fn new(
        id: SpellRef,
        name: String,
        description: String,
        level: SpellLevel,
        components: CastingComponents,
        time: CastingTime,
        range: CastingRange,
        duration: SpellDuration,
        school: MageSchool,
        ritual: bool,
    ) -> Spell {
        Spell {
            id,
            name,
            description,
            level,
            components,
            time,
            range,
            duration,
            school,
            ritual,
        }
    }
    pub fn id(&self) -> &SpellRef {
        &self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn description(&self) -> &String {
        &self.description
    }
    pub fn level(&self) -> &SpellLevel {
        &self.level
    }
    pub fn components(&self) -> &CastingComponents {
        &self.components
    }
    pub fn casting_time(&self) -> &CastingTime {
        &self.time
    }
    pub fn range(&self) -> &CastingRange {
        &self.range
    }
    pub fn duration(&self) -> &SpellDuration {
        &self.duration
    }
    pub fn school(&self) -> &MageSchool {
        &self.school
    }
    pub fn ritual(&self) -> &bool {
        &self.ritual
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum SpellLevel {
    #[serde(rename = "cantrip")]
    Cantrip,
    #[serde(untagged)]
    Level(usize),
}

impl SpellLevel {
    pub fn new(level: usize) -> Result<SpellLevel, String> {
        if level == 0 {
            Ok(SpellLevel::Cantrip)
        } else if level < 10 {
            Ok(SpellLevel::Level(level))
        } else {
            Err("Spell level must be 0 - 9.".to_string())
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            SpellLevel::Cantrip => 0,
            SpellLevel::Level(level) => *level,
        }
    }
}

impl PartialOrd for SpellLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.as_usize().cmp(&other.as_usize()))
    }
}

#[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq)]
pub enum CastingFrequency {
    OncePer(Rest),
}

#[derive(Clone, Debug)]
pub struct SpellSlot {
    level: SpellLevel,
    expended: bool,
}

impl SpellSlot {
    pub fn new(level: SpellLevel) -> Self {
        SpellSlot {
            level,
            expended: false,
        }
    }

    pub fn level(&self) -> &SpellLevel {
        &self.level
    }

    pub fn expend(&mut self) {
        if self.expended {
            panic!("Trying to expend an already expended spell slot.")
        }
        self.expended = true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CastingComponents {
    verbal: bool,
    somatic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    material: Option<String>,
}

impl CastingComponents {
    pub fn new(verbal: bool, somatic: bool, material: Option<String>) -> Self {
        Self {
            verbal,
            somatic,
            material,
        }
    }

    pub fn verbal(&self) -> bool {
        self.verbal
    }

    pub fn somatic(&self) -> bool {
        self.somatic
    }

    pub fn material(&self) -> Option<&str> {
        self.material.as_deref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MageSchool {
    Abjuration,
    Conjuration,
    Divination,
    Enchantment,
    Evocation,
    Illusion,
    Necromancy,
    Transmutation,
}
