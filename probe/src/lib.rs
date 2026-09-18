//! Bisect: which derive cannot cope with `Vec<M::Kind>`?

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub trait Mode: Copy + std::fmt::Debug + PartialEq + 'static {
    type Effect: Clone + PartialEq + Serialize + DeserializeOwned;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Final;
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Draft;

impl Mode for Final {
    type Effect = String;
}
impl Mode for Draft {
    type Effect = Vec<String>;
}

pub mod std_only {
    use super::*;

    #[derive(Clone, PartialEq)]
    pub struct A<M: Mode = Final> {
        pub title: String,
        pub effects: Vec<M::Effect>,
    }
}

pub mod serde_only {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub struct B<M: Mode = Final> {
        pub title: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub effects: Vec<M::Effect>,
    }
}

pub mod builder_plain_setter {
    use super::*;
    use typed_builder::TypedBuilder;

    #[derive(TypedBuilder)]
    pub struct C<M: Mode = Final> {
        #[builder(setter(into))]
        pub title: String,
        pub effects: Vec<M::Effect>,
    }
}

pub mod builder_default_only {
    use super::*;
    use typed_builder::TypedBuilder;

    #[derive(TypedBuilder)]
    pub struct D<M: Mode = Final> {
        #[builder(setter(into))]
        pub title: String,
        #[builder(default)]
        pub effects: Vec<M::Effect>,
    }
}

#[cfg(feature = "typed_builder_e0220_repro")]
pub mod builder_via_mutators {
    use super::*;
    use typed_builder::TypedBuilder;

    #[derive(TypedBuilder)]
    #[builder(mutators(
        pub fn with_effect(&mut self, effect: impl Into<M::Effect>) {
            self.effects.push(effect.into())
        }
    ))]
    pub struct E<M: Mode = Final> {
        #[builder(setter(into))]
        pub title: String,
        #[builder(default, via_mutators)]
        pub effects: Vec<M::Effect>,
    }
}

#[cfg(feature = "typed_builder_e0220_repro")]
pub mod builder_via_mutators_no_default_param {
    use super::*;
    use typed_builder::TypedBuilder;

    #[derive(TypedBuilder)]
    #[builder(mutators(
        pub fn with_effect(&mut self, effect: impl Into<M::Effect>) {
            self.effects.push(effect.into())
        }
    ))]
    pub struct F<M: Mode> {
        #[builder(setter(into))]
        pub title: String,
        #[builder(default, via_mutators)]
        pub effects: Vec<M::Effect>,
    }
}

#[cfg(feature = "typed_builder_e0220_repro")]
pub mod g_via_mutators_init {
    use super::*;
    use typed_builder::TypedBuilder;

    #[derive(TypedBuilder)]
    #[builder(mutators(
        pub fn with_effect(&mut self, effect: impl Into<M::Effect>) {
            self.effects.push(effect.into())
        }
    ))]
    pub struct G<M: Mode = Final> {
        #[builder(setter(into))]
        pub title: String,
        #[builder(via_mutators(init = Vec::new()))]
        pub effects: Vec<M::Effect>,
    }
}

#[cfg(feature = "typed_builder_e0220_repro")]
pub mod h_via_mutators_without_mutator {
    use super::*;
    use typed_builder::TypedBuilder;

    #[derive(TypedBuilder)]
    pub struct H<M: Mode = Final> {
        #[builder(setter(into))]
        pub title: String,
        #[builder(default, via_mutators)]
        pub effects: Vec<M::Effect>,
    }
}

pub mod j_mutator_over_plain_vec {
    use super::*;
    use typed_builder::TypedBuilder;

    #[derive(TypedBuilder)]
    #[builder(mutators(
        pub fn with_tag(&mut self, tag: impl Into<String>) {
            self.tags.push(tag.into())
        }
    ))]
    pub struct J<M: Mode = Final> {
        #[builder(setter(into))]
        pub title: String,
        pub effects: Vec<M::Effect>,
        #[builder(default, via_mutators)]
        pub tags: Vec<String>,
    }
}

pub mod k_fully_qualified_projection {
    use super::*;
    use typed_builder::TypedBuilder;

    #[derive(TypedBuilder)]
    #[builder(mutators(
        pub fn with_effect(&mut self, effect: impl Into<<M as Mode>::Effect>) {
            self.effects.push(effect.into())
        }
    ))]
    pub struct K<M: Mode = Final> {
        #[builder(setter(into))]
        pub title: String,
        #[builder(default, via_mutators)]
        pub effects: Vec<<M as Mode>::Effect>,
    }

    /// The builder must also be *usable* in both modes, with `M` inferred from the return type.
    pub fn final_k() -> K {
        K::builder().title("t").with_effect("x").build()
    }

    pub fn draft_k() -> K<Draft> {
        K::builder().title("t").with_effect(vec!["x".to_string()]).build()
    }
}
