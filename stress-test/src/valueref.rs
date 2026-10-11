use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ValueRef {
    Sheet(SheetRef),
    Const(isize),
}

impl From<usize> for ValueRef {
    fn from(value: usize) -> Self {
        Self::Const(value as isize)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SheetRef {
    Level,
}
