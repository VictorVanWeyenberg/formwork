use crate::blueprint::blueprint;
use crate::effect::{Effect, SheetOperation};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// ONE definition. `Trait` means `Trait<Final>`; `Trait<Draft>` is the trait blueprint.
///
/// No `Debug` derive on purpose: `fifth_domain::traits::Trait` has none, and `Node` does not
/// require it, so the spike shows the design works without adding `Debug` everywhere.
///
/// The slot field is spelled `Vec<<M as Mode>::Effect>`: typed-builder 0.23's `via_mutators`
/// emits the field type in a context where the shorthand `M::Effect` does not resolve (E0220);
/// the fully qualified form does. That spelling in turn makes serde's bound inference see `M`
/// and demand `M: Serialize + Deserialize + Default`; `#[serde(bound = "")]` switches the
/// inference off — the `Node` bounds on the associated types are all the impls need.
#[blueprint]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Trait {
    title: String,
    description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    effects: Vec<Effect>,
}

impl Trait {
    pub fn new(title: String, description: String, effects: Vec<Effect>) -> Self {
        Self { title, description, effects, phantom: Default::default() }
    }
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn effects(&self) -> &[Effect] {
        &self.effects
    }

    pub fn add_effect(&mut self, effect: impl Into<Effect>) {
        self.effects.push(effect.into());
    }
}
