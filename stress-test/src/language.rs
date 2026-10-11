use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Language {
    // Standard
    Common,
    Dwarvish,
    Elvish,
    Giant,
    Gnomish,
    Goblin,
    Halfling,
    Orc,
    // Exotic
    Abyssal,
    Celestial,
    DeepSpeech,
    Draconic,
    Infernal,
    Primordial,
    Sylvan,
    Undercommon,
}

impl Language {
    pub fn script(&self) -> Option<Script> {
        match self {
            Language::Common | Language::Halfling => Some(Script::Common),
            Language::Dwarvish
            | Language::Giant
            | Language::Gnomish
            | Language::Goblin
            | Language::Orc
            | Language::Primordial => Some(Script::Dwarvish),
            Language::Elvish | Language::Sylvan | Language::Undercommon => Some(Script::Elvish),
            Language::Abyssal | Language::Infernal => Some(Script::Infernal),
            Language::Celestial => Some(Script::Celestial),
            Language::DeepSpeech => None,
            Language::Draconic => Some(Script::Draconic),
        }
    }

    pub fn rareness(&self) -> LanguageRareness {
        match self {
            Language::Common
            | Language::Dwarvish
            | Language::Elvish
            | Language::Giant
            | Language::Gnomish
            | Language::Goblin
            | Language::Halfling
            | Language::Orc => LanguageRareness::Standard,
            Language::Abyssal
            | Language::Celestial
            | Language::DeepSpeech
            | Language::Draconic
            | Language::Infernal
            | Language::Primordial
            | Language::Sylvan
            | Language::Undercommon => LanguageRareness::Exotic,
        }
    }
}

pub enum Script {
    Common,
    Dwarvish,
    Elvish,
    Infernal,
    Celestial,
    Draconic,
}

pub enum LanguageRareness {
    Standard,
    Exotic,
}
