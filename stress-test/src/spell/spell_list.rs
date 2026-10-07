use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct SpellList {
    pub class: String,
    pub spells: Vec<String>,
}

impl SpellList {
    pub fn new(class: impl ToString, spells: Vec<String>) -> SpellList {
        SpellList { class: class.to_string(), spells }
    }
}
