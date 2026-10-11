//! # D&D 5e Character Sheet Verification Suite
//!
//! This crate serves as a comprehensive, end-to-end verification suite modeling the complex domain
//! rules of a Dungeons & Dragons 5th Edition character sheet using [`schematic`].
//!
//! # Schematic Domain Modeling
//!
//! In D&D 5e, character creation involves deeply nested choices that cascade through multiple
//! layers of rules and features:
//!
//! 1. **Race ([`race::Race`])**:
//!    Grants racial traits (e.g., Dwarven Resilience, Tool Proficiency) modeled via
//!    `#[slot(collection, cascade)] traits: Vec<Trait>`.
//! 2. **Traits ([`traits::Trait`])**:
//!    Represent racial or class features that confer concrete mechanical effects modeled via
//!    `#[slot(collection, cascade)] effects: Vec<Effect>`.
//! 3. **Effects ([`effect::Effect`])**:
//!    Encapsulate discrete character modifications via `#[slot] operation: SheetOperation`,
//!    which may be a fixed bonus (e.g. +2 Constitution) or a pending decision (e.g. "Choose one tool
//!    proficiency from Brewer's Supplies, Mason's Tools, Smith's Tools").
//!
//! By utilizing `schematic`'s cascading slots, an unresolved character blueprint (`RaceSchematic`)
//! can be created with open choices at any depth, navigated via visitors, and resolved into a
//! watertight domain model (`Race`) once all choices are made.

#![allow(clippy::all)]

use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;

pub mod ability;
pub mod class;
pub mod condition;
pub mod contentref;
pub mod damage;
pub mod dice;
pub mod effect;
pub mod feature;
pub mod inventory;
pub mod language;
pub mod proficiency;
pub mod race;
pub mod resource;
pub mod rest;
pub mod roll;
pub mod sheet;
pub mod skill;
pub mod spell;
pub mod traits;
pub mod valueref;

pub use inventory::{armor, item, tool, wallet, weapon};

pub struct PlayableSheetReference {
    uuid: Uuid,
}

impl Default for PlayableSheetReference {
    fn default() -> Self {
        PlayableSheetReference {
            uuid: Uuid::new_v4(),
        }
    }
}

impl Display for PlayableSheetReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PSR_{}", self.uuid.as_hyphenated())
    }
}

impl FromStr for PlayableSheetReference {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("PSR_") {
            Err("String is not a playable sheet reference.".to_string())
        } else {
            Uuid::from_str(&s[5..])
                .map(PlayableSheetReference::new)
                .map_err(|err| err.to_string())
        }
    }
}

impl PlayableSheetReference {
    fn new(uuid: Uuid) -> Self {
        Self { uuid }
    }
}
