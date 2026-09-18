use crate::blueprint::{Blueprint, Draft, Final, Mode};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// ONE definition. `Race` means `Race<Final>`; `Race<Draft>` replaces the `RaceBlueprint` struct.
#[derive(Clone, PartialEq, Serialize, Deserialize, TypedBuilder, Blueprint)]
#[serde(bound = "")]
#[builder(mutators(
    /// Final: `Trait`. Draft: `Trait` (lifted), `Trait<Draft>` (fixed, may hold choices) or
    /// `Choice<Trait<Draft>>` (pending).
    pub fn with_trait(&mut self, trait_: impl Into<<M as Mode>::Trait>) {
        self.traits.push(trait_.into())
    }
))]
pub struct Race<M: Mode = Final> {
    /// `String` stands in for `RaceRef`.
    #[builder(setter(into))]
    id: String,
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into))]
    description: String,
    #[builder(default, via_mutators)]
    traits: Vec<<M as Mode>::Trait>,
}

impl<M: Mode> Race<M> {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn traits(&self) -> &[M::Trait] {
        &self.traits
    }
}

pub type RaceBlueprint = Race<Draft>;
