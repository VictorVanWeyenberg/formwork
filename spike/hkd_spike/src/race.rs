use crate::traits::TraitDraft;
use crate::traits::Trait;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use spike_derive::blueprint;

/// ONE definition. `Race` means `Race<Final>`; `Race<Draft>` replaces the `RaceBlueprint` struct.
#[blueprint(derive(Clone, PartialEq))]
#[derive(Clone, PartialEq, Serialize, Deserialize, TypedBuilder)]
#[serde(bound = "")]
#[builder(mutators(
    pub fn with_trait(&mut self, trait_: impl Into<<M as RaceMode>::Traits>) {
        self.traits.push(trait_.into())
    }
))]
pub struct Race {
    #[builder(setter(into))]
    id: String,
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into))]
    description: String,
    #[slots_transitive]
    #[builder(default, via_mutators)]
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
