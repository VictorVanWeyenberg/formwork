#![allow(dead_code)]

use serde::{Serialize, Deserialize};

trait Mode {
    type Effects: Clone + Serialize + for<'schematic_a> Deserialize<'schematic_a> + PartialEq;
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Resolved;

impl Mode for Resolved {
    type Effects = String;
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Trait<M: Mode = Resolved> {
    effects: M::Effects,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Subrace {
    traits: Vec<Trait>,
}

fn main() {}
