use serde::{Deserialize, Serialize};

/// Plain domain leaf type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Leaf {
    pub name: String,
    pub value: i32,
}

impl Leaf {
    pub fn new(name: impl Into<String>, value: i32) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}
