use crate::blueprint::Slot;
use crate::effect::Effect;
use crate::traits::Trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// What every value that can sit in a chooseable position must support.
pub trait Node: Clone + PartialEq + Serialize + DeserializeOwned {}
impl<T: Clone + PartialEq + Serialize + DeserializeOwned> Node for T {}

/// A type family: for every kind of domain value that may sit in a list position, the mode says
/// how that position is represented. Adding a kind (Feature, Item, …) adds one line here and one
/// line to each mode below.
pub trait Mode: Copy + Debug + PartialEq + 'static {
    type Effect: Node + From<Effect>;
    type Trait: Node + From<Trait<Final>>;
}

/// The finished domain: a position *is* the value.
///
/// Deliberately *not* `Serialize`/`Deserialize`: S0 item 1 checks that serde does not require
/// `M: Serialize` when `M` only appears as `M::Kind`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Final;

/// The blueprint: a position is a value or a pending choice.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Draft;

impl Mode for Final {
    type Effect = Effect;
    type Trait = Trait<Final>;
}

impl Mode for Draft {
    type Effect = Slot<Effect>;
    type Trait = Slot<Trait<Draft>>;
}
