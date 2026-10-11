use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Not;
use std::str::FromStr;

const SEPARATOR: &str = "/";

pub trait Referable {
    type Ref;
    fn reference(&self) -> &Self::Ref;
}

#[derive(Debug)]
pub enum RefError {
    ContainsSeparator(String),
    Regex(regex::Error),
    InvalidRef(String),
}

impl Display for RefError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RefError::ContainsSeparator(source) => {
                write!(f, "{source} contains the separator character `{SEPARATOR}`")
            }
            RefError::Regex(e) => write!(f, "{e}"),
            RefError::InvalidRef(e) => write!(f, "Invalid reference: `{e}`"),
        }
    }
}

impl From<regex::Error> for RefError {
    fn from(e: regex::Error) -> Self {
        RefError::Regex(e)
    }
}

impl Error for RefError {}

pub enum Ref {
    ArmorRef(ArmorRef),
    RaceRef(RaceRef),
    SpellRef(SpellRef),
    SubraceRef(SubraceRef),
    ToolRef(ToolRef),
    WeaponRef(WeaponRef),
}

#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct SourceRef(String);

impl Display for SourceRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SourceRef {
    type Err = RefError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl SourceRef {
    pub fn new(source: impl Into<String>) -> Result<Self, RefError> {
        let source = source.into();
        source
            .contains(SEPARATOR)
            .not()
            .then_some(Self(source))
            .ok_or(RefError::ContainsSeparator("SourceRef".to_string()))
    }
}

macro_rules! define_item_ref {
    ($prereq:ident, $name:ident, $tag:expr) => {
        #[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize, Deserialize)]
        pub struct $name($prereq, String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{SEPARATOR}{}{SEPARATOR}{}", self.0, $tag, self.1)
            }
        }

        impl std::str::FromStr for $name {
            type Err = RefError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let tag = regex::escape($tag);
                let pattern = format!("(?P<source>.+){SEPARATOR}{tag}{SEPARATOR}(?P<item>.+)");
                let regex = Regex::new(&pattern)?;
                let caps = regex
                    .captures(s)
                    .ok_or(RefError::InvalidRef(s.to_string()))?;
                Self::new($prereq::from_str(&caps["source"])?, &caps["item"])
            }
        }

        impl From<$name> for Ref {
            fn from(t: $name) -> Self {
                Ref::$name(t)
            }
        }

        impl $name {
            pub fn new(source: $prereq, item: impl Into<String>) -> Result<Self, RefError> {
                let item = item.into();
                item.contains(SEPARATOR)
                    .not()
                    .then_some(Self(source, item))
                    .ok_or(RefError::ContainsSeparator(stringify!($name).to_string()))
            }
        }
    };
}

define_item_ref!(SourceRef, ArmorRef, "armor");
define_item_ref!(SourceRef, RaceRef, "race");
define_item_ref!(SourceRef, SpellRef, "spell");
define_item_ref!(SourceRef, SubraceRef, "subrace");
define_item_ref!(SourceRef, ToolRef, "tool");
define_item_ref!(SourceRef, WeaponRef, "weapon");

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn source_ref_happy() {
        let tool = ToolRef::from_str("srd/tool/brewersupplies").expect("tool should be valid");
        assert_eq!("srd", tool.0.0);
        assert_eq!("brewersupplies", tool.1);
    }

    #[test]
    fn tool_one_extra_separator() {
        assert!(
            ToolRef::from_str("srd/tool/brewersupplies/oops")
                .is_err_and(|e| matches!(e, RefError::ContainsSeparator(_)))
        )
    }

    #[test]
    fn tool_marker_not_present() {
        assert!(
            ToolRef::from_str("srd//brewersupplies")
                .is_err_and(|e| matches!(e, RefError::InvalidRef(_)))
        )
    }

    #[test]
    fn my_cat_went_to_the_toilet() {
        assert!(
            ToolRef::from_str("ninkystinky").is_err_and(|e| matches!(e, RefError::InvalidRef(_)))
        )
    }
}
