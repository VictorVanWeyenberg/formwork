use crate::blueprint::{Blueprint, SlotOne};
use crate::effect::Effect;
use crate::traits::Trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

pub trait Mode {
    type Draft: Clone + PartialEq + Serialize + DeserializeOwned + Blueprint<Final = Self::Final>;
    type Final: Clone + PartialEq + Serialize + DeserializeOwned;
}

/// What every value that can sit in a chooseable position must support.
pub trait Node: Clone + PartialEq + Serialize + DeserializeOwned {}
impl<T: Clone + PartialEq + Serialize + DeserializeOwned> Node for T {}