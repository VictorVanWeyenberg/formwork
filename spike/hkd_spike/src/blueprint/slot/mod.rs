use crate::blueprint::{Blueprint, Cardinality, Choice, ChoiceError, ChoicePrompt};
use serde::{Deserialize, Serialize};

mod one;
mod many;
mod one_t;
mod many_t;

pub use one::SlotOne;
pub use many::SlotMany;
pub use one_t::SlotOneTransitive;
pub use many_t::SlotManyTransitive;