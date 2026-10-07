use crate::choice::{SingleChoice, MultiChoice};
use crate::error::SchematicError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Slot<U, R, C> {
    Choose(C),
    FixedUnresolved(U),
    FixedResolved(R),
}

pub type SingleSlot<U, R> = Slot<U, R, SingleChoice<U>>;
pub type MultiSlot<U, R> = Slot<U, R, MultiChoice<U>>;

impl<U, R, C> Slot<U, R, C> {
    pub fn as_choice(&self) -> Option<&C> {
        match self {
            Slot::Choose(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_choice_mut(&mut self) -> Option<&mut C> {
        match self {
            Slot::Choose(c) => Some(c),
            _ => None,
        }
    }
}

impl<U, R> From<SingleChoice<U>> for SingleSlot<U, R> {
    fn from(choice: SingleChoice<U>) -> Self {
        Slot::Choose(choice)
    }
}

impl<U, R> From<MultiChoice<U>> for MultiSlot<U, R> {
    fn from(choice: MultiChoice<U>) -> Self {
        Slot::Choose(choice)
    }
}

impl<U, R> From<U> for SingleSlot<U, R> {
    fn from(value: U) -> Self {
        Slot::FixedUnresolved(value)
    }
}

impl<U, R> From<U> for MultiSlot<U, R> {
    fn from(value: U) -> Self {
        Slot::FixedUnresolved(value)
    }
}

impl<U, R> SingleSlot<U, R> {
    pub fn is_pending(&self) -> bool {
        match self {
            Slot::Choose(c) => !c.is_made(),
            _ => false,
        }
    }

    pub fn is_made(&self) -> bool {
        match self {
            Slot::Choose(c) => c.is_made(),
            _ => false,
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Slot::FixedUnresolved(_) | Slot::FixedResolved(_) => true,
            Slot::Choose(c) => c.is_made(),
        }
    }

    pub fn resolve_exact(self) -> Result<U, SchematicError> {
        match self {
            Slot::FixedUnresolved(u) => Ok(u),
            Slot::Choose(c) => c.resolve_exact(),
            Slot::FixedResolved(_) => Err(SchematicError::Custom("Slot already resolved".into())),
        }
    }
}

impl<U, R> MultiSlot<U, R> {
    pub fn is_pending(&self) -> bool {
        match self {
            Slot::Choose(c) => !c.is_made(),
            _ => false,
        }
    }

    pub fn is_made(&self) -> bool {
        match self {
            Slot::Choose(c) => c.is_made(),
            _ => false,
        }
    }

    pub fn is_complete(&self) -> bool {
        match self {
            Slot::FixedUnresolved(_) | Slot::FixedResolved(_) => true,
            Slot::Choose(c) => c.is_made(),
        }
    }

    pub fn resolve(self) -> Result<Vec<U>, SchematicError> {
        match self {
            Slot::FixedUnresolved(u) => Ok(vec![u]),
            Slot::Choose(c) => c.resolve(),
            Slot::FixedResolved(_) => Err(SchematicError::Custom("Slot already resolved".into())),
        }
    }
}
