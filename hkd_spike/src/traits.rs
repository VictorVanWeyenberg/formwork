use crate::blueprint::blueprint;
use crate::blueprint::{Draft, Final, Mode};
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
#[derive(Clone, PartialEq, Serialize, Deserialize, TypedBuilder)]
#[serde(bound = "")]
#[builder(mutators(
    /// Final: `Effect`. Draft: `Effect` (fixed) or `Choice<Effect>` (pending).
    pub fn with_effect(&mut self, effect: impl Into<<M as Mode>::Effect>) {
        self.effects.push(effect.into())
    }
    pub fn with_permanent_effect(&mut self, operation: SheetOperation) {
        self.effects.push(Effect::new(operation).into())
    }
))]
pub struct Trait<M: Mode = Final> {
    #[builder(setter(into))]
    title: String,
    #[builder(setter(into))]
    description: String,
    #[builder(default, via_mutators)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    effects: Vec<<M as Mode>::Effect>,
}

impl<M: Mode> Trait<M> {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn effects(&self) -> &[M::Effect] {
        &self.effects
    }

    pub fn add_effect(&mut self, effect: impl Into<M::Effect>) {
        self.effects.push(effect.into());
    }
}

/// Domain vocabulary preserved: a blueprint is just the draft mode.
pub type TraitBlueprint = Trait<Draft>;
