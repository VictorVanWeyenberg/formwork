use std::fmt::{Display, Formatter};
use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
pub enum CastingTime {
    Minutes(usize),
    Hours(usize),
    Action,
    BonusAction,
    Reaction(String),
}

impl Display for CastingTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CastingTime::Minutes(minutes) => {
                if minutes > &1 {
                    write!(f, "{} minutes", minutes)
                } else {
                    write!(f, "{} minute", minutes)
                }
            }
            CastingTime::Hours(hours) => {
                if hours > &1 {
                    write!(f, "{} hours", hours)
                } else {
                    write!(f, "{} hour", hours)
                }
            }
            CastingTime::Action => write!(f, "1 action"),
            CastingTime::BonusAction => write!(f, "1 bonus action"),
            CastingTime::Reaction(which) => write!(f, "1 reaction, {}", which),
        }
    }
}

impl FromStr for CastingTime {
    type Err = String;
    fn from_str(time_string: &str) -> Result<Self, Self::Err> {
        let hours_re = Regex::new("^(?<hours>\\d+) (hour|hours)$").unwrap();
        let minutes_re = Regex::new("^(?<minutes>\\d+) (minute|minutes)$").unwrap();
        let one_reaction_re = Regex::new("^1 reaction, (?<which>.+)$").unwrap();

        let one_reaction_captures = one_reaction_re.captures(&time_string);
        let hours_captures = hours_re.captures(&time_string);
        let minutes_captures = minutes_re.captures(&time_string);
        if time_string == "1 action" {
            Ok(CastingTime::Action)
        } else if time_string == "1 bonus action" {
            Ok(CastingTime::BonusAction)
        } else if one_reaction_captures.is_some() {
            let which = (&one_reaction_captures.unwrap()["which"]).to_string();
            Ok(CastingTime::Reaction(which))
        } else if hours_captures.is_some() {
            match (&hours_captures.unwrap()["hours"]).parse::<usize>() {
                Ok(hours) => Ok(CastingTime::Hours(hours)),
                Err(_) => Err("Hours regex matches but cannot parse number of hours.".to_string())
            }
        } else if minutes_captures.is_some() {
            match (&minutes_captures.unwrap()["minutes"]).parse::<usize>() {
                Ok(minutes) => Ok(CastingTime::Minutes(minutes)),
                Err(_) => Err("Minutes regex matches but cannot parse number of minutes.".to_string())
            }
        } else {
            Err(format!("Unsupported casting time: \"{}\".", time_string))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::spell::CastingTime;

    #[test]
    fn action() {
        let time = CastingTime::Action;
        let value = serde_json::to_value(&time)
            .expect("Unexpected error.");
        assert_eq!(Value::String("1 action".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error.");
        assert_eq!(time, deserialized);
    }

    #[test]
    fn action_string() {
        let time = CastingTime::Action;
        let value = serde_json::to_value(&time)
            .expect("Unexpected error 1.")
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!("1 action".to_string(), value);
    }

    #[test]
    fn bonus_action() {
        let time = CastingTime::BonusAction;
        let json = serde_json::to_value(&time)
            .expect("Unexpected error.");
        assert_eq!("1 bonus action", json);
        let deserialized = serde_json::from_value(json)
            .expect("Unexpected error.");
        assert_eq!(time, deserialized);
    }

    #[test]
    fn reaction() {
        let which = "which you take when running tests".to_string();
        let time = CastingTime::Reaction(which.clone());
        let json = serde_json::to_value(&time)
            .expect("Unexpected error.");
        assert_eq!(format!("1 reaction, {}", which), json);
        let deserialized = serde_json::from_value(json)
            .expect("Unexpected error.");
        assert_eq!(time, deserialized);
    }

    #[test]
    fn hours() {
        let hours = 3;
        let time = CastingTime::Hours(hours);
        let json = serde_json::to_value(&time)
            .expect("Unexpected error.");
        assert_eq!(format!("{} hours", hours), json);
        let deserialized = serde_json::from_value(json)
            .expect("Unexpected error.");
        assert_eq!(time, deserialized);
    }

    #[test]
    fn minutes() {
        let minutes = 3;
        let time = CastingTime::Minutes(minutes);
        let json = serde_json::to_value(&time)
            .expect("Unexpected error.");
        assert_eq!(format!("{} minutes", minutes), json);
        let deserialized = serde_json::from_value(json)
            .expect("Unexpected error.");
        assert_eq!(time, deserialized);
    }
}