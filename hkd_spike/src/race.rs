use crate::traits::TraitDraft;
use crate::traits::Trait;
use serde::{Deserialize, Serialize};
use spike_derive::blueprint;

/// ONE definition. `Race` means `Race<Final>`; `Race<Draft>` replaces the `RaceBlueprint` struct.
#[blueprint(derive(Clone, PartialEq))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Race {
    /// `String` stands in for `RaceRef`.
    id: String,
    name: String,
    description: String,
    #[slots_transitive]
    traits: Vec<Trait>,
}

impl Race {
    pub fn new(id: String, name: String, description: String, traits: Vec<Trait>) -> Self {
        Self { id, name, description, traits }
    }
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn traits(&self) -> &[Trait] {
        &self.traits
    }
}
