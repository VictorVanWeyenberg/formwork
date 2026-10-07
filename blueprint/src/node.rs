use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// Marker trait representing what every value and draft in a blueprint slot must support.
pub trait Node: Clone + Debug + PartialEq + Eq + Serialize + DeserializeOwned {}

impl<T: Clone + Debug + PartialEq + Eq + Serialize + DeserializeOwned> Node for T {}
