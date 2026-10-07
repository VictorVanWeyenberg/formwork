use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpellDuration {
    concentration: bool,
    #[serde_as(as = "DisplayFromStr")]
    duration: Duration,
}

impl SpellDuration {
    pub fn new(concentration: bool, duration: Duration) -> SpellDuration {
        SpellDuration { concentration, duration }
    }
    pub fn concentration(&self) -> &bool {
        &self.concentration
    }
    pub fn duration(&self) -> &Duration {
        &self.duration
    }
}

impl Display for SpellDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.concentration {
            let duration = format!("{}", self.duration).to_lowercase();
            write!(f, "Concentration, {}", duration)
        } else {
            write!(f, "{}", self.duration)
        }
    }
}

impl FromStr for SpellDuration {
    type Err = String;

    fn from_str(spell_duration_string: &str) -> Result<Self, Self::Err> {
        if let Some(captures) = Regex::new("^Concentration, (?<duration>.*)$")
            .unwrap()
            .captures(&spell_duration_string) {
            let duration = Duration::from_str(&captures["duration"])?;
            Ok(SpellDuration::new(true, duration))
        } else if let Some(captures) = Regex::new("^(?<duration>.*)$")
            .unwrap()
            .captures(&spell_duration_string) {
            let duration = Duration::from_str(&captures["duration"])?;
            Ok(SpellDuration::new(false, duration))
        } else {
            Err(format!("Unsupported spell duration: \"{}\".", spell_duration_string))
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Duration {
    Instantaneous,
    Minutes(usize),
    Hours(usize),
    Days(usize),
    Rounds(usize),
    UpToMinutes(usize),
    UpToHours(usize),
    UpToDays(usize),
    UpToRounds(usize),
    Special,
    Until(#[serde_as(as = "DisplayFromStr")] CastingDurationUntils),
}

impl Display for Duration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Duration::Instantaneous => write!(f, "Instantaneous"),
            Duration::Minutes(minutes) => {
                if *minutes == 1 {
                    write!(f, "{} minute", minutes)
                } else {
                    write!(f, "{} minutes", minutes)
                }
            }
            Duration::Hours(hours) => {
                if *hours == 1 {
                    write!(f, "{} hour", hours)
                } else {
                    write!(f, "{} hours", hours)
                }
            }
            Duration::Days(days) => {
                if *days == 1 {
                    write!(f, "{} day", days)
                } else {
                    write!(f, "{} days", days)
                }
            }
            Duration::Rounds(rounds) => {
                if *rounds == 1 {
                    write!(f, "{} round", rounds)
                } else {
                    write!(f, "{} rounds", rounds)
                }
            }
            Duration::UpToMinutes(minutes) => {
                if *minutes == 1 {
                    write!(f, "Up to {} minute", minutes)
                } else {
                    write!(f, "Up to {} minutes", minutes)
                }
            }
            Duration::UpToHours(hours) => {
                if *hours == 1 {
                    write!(f, "Up to {} hour", hours)
                } else {
                    write!(f, "Up to {} hours", hours)
                }
            }
            Duration::UpToDays(days) => {
                if *days == 1 {
                    write!(f, "Up to {} day", days)
                } else {
                    write!(f, "Up to {} days", days)
                }
            }
            Duration::UpToRounds(rounds) => {
                if *rounds == 1 {
                    write!(f, "Up to {} round", rounds)
                } else {
                    write!(f, "Up to {} rounds", rounds)
                }
            }
            Duration::Special => write!(f, "Special"),
            Duration::Until(until) => write!(f, "Until {}", until)
        }
    }
}

impl FromStr for Duration {
    type Err = String;

    fn from_str(duration_string: &str) -> Result<Self, Self::Err> {
        if duration_string == "Special" {
            Ok(Duration::Special)
        } else if duration_string == "Instantaneous" {
            Ok(Duration::Instantaneous)
        } else if let Some(captures) = Regex::new("^(?<minutes>\\d+) (minute|minutes)$")
            .unwrap()
            .captures(&duration_string) {
            match (&captures["minutes"]).parse::<usize>() {
                Ok(minutes) => Ok(Duration::Minutes(minutes)),
                Err(_) => Err("Minutes regex matches but unable to parse minutes.".to_string())
            }
        } else if let Some(captures) = Regex::new("^(?<hours>\\d+) (hour|hours)$")
            .unwrap()
            .captures(&duration_string) {
            match (&captures["hours"]).parse::<usize>() {
                Ok(hours) => Ok(Duration::Hours(hours)),
                Err(_) => Err("Hours regex matches but unable to parse hours.".to_string())
            }
        } else if let Some(captures) = Regex::new("^(?<days>\\d+) (day|days)$")
            .unwrap()
            .captures(&duration_string) {
            match (&captures["days"]).parse::<usize>() {
                Ok(days) => Ok(Duration::Days(days)),
                Err(_) => Err("Days regex matches but unable to parse days.".to_string())
            }
        } else if let Some(captures) = Regex::new("^(?<rounds>\\d+) (round|rounds)$")
            .unwrap()
            .captures(&duration_string) {
            match (&captures["rounds"]).parse::<usize>() {
                Ok(rounds) => Ok(Duration::Rounds(rounds)),
                Err(_) => Err("Rounds regex matches but unable to parse rounds.".to_string())
            }
        } else if let Some(captures) = Regex::new("^(Up|up) to (?<minutes>\\d+) (minute|minutes)$")
            .unwrap()
            .captures(&duration_string) {
            match (&captures["minutes"]).parse::<usize>() {
                Ok(minutes) => Ok(Duration::UpToMinutes(minutes)),
                Err(_) => Err("UpToMinutes regex matches but unable to parse minutes.".to_string())
            }
        } else if let Some(captures) = Regex::new("^(Up|up) to (?<hours>\\d+) (hour|hours)$")
            .unwrap()
            .captures(&duration_string) {
            match (&captures["hours"]).parse::<usize>() {
                Ok(hours) => Ok(Duration::UpToHours(hours)),
                Err(_) => Err("UpToHours regex matches but unable to parse hours.".to_string())
            }
        } else if let Some(captures) = Regex::new("^(Up|up) to (?<days>\\d+) (day|days)$")
            .unwrap()
            .captures(&duration_string) {
            match (&captures["days"]).parse::<usize>() {
                Ok(days) => Ok(Duration::UpToDays(days)),
                Err(_) => Err("UpToDays regex matches but unable to parse days.".to_string())
            }
        } else if let Some(captures) = Regex::new("^(Up|up) to (?<rounds>\\d+) (round|rounds)$")
            .unwrap()
            .captures(&duration_string) {
            match (&captures["rounds"]).parse::<usize>() {
                Ok(rounds) => Ok(Duration::UpToRounds(rounds)),
                Err(_) => Err("UpToRounds regex matches but unable to parse rounds.".to_string())
            }
        } else if let Some(captures) = Regex::new("^(Until|until) (?<untils>(( or )?(dispelled|triggered))+)$")
            .unwrap()
            .captures(&duration_string) {
            captures["untils"].split(" or ")
                .map(|until| CastingDurationUntil::from_str(until))
                .collect::<Result<Vec<CastingDurationUntil>, _>>()
                .map(CastingDurationUntils::from)
                .map(Duration::Until)
        } else {
            Err(format!("Unsupported duration: \"{}\".", duration_string))
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CastingDurationUntils {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    pub untils: Vec<CastingDurationUntil>,
}

impl From<Vec<CastingDurationUntil>> for CastingDurationUntils {
    fn from(untils: Vec<CastingDurationUntil>) -> Self {
        CastingDurationUntils { untils }
    }
}

impl From<CastingDurationUntil> for CastingDurationUntils {
    fn from(value: CastingDurationUntil) -> Self {
        vec![value].into()
    }
}

impl Display for CastingDurationUntils {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.untils.iter()
            .map(|until| format!("{}", until))
            .collect::<Vec<String>>()
            .join(" or "))
    }
}

impl FromStr for CastingDurationUntils {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(" or ")
            .map(|until| CastingDurationUntil::from_str(until))
            .collect::<Result<Vec<_>, _>>()
            .map(|untils| CastingDurationUntils { untils })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CastingDurationUntil {
    #[serde(rename = "dispelled")]
    Dispelled,
    #[serde(rename = "triggered")]
    Triggered,
}

impl Display for CastingDurationUntil {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CastingDurationUntil::Dispelled => write!(f, "dispelled"),
            CastingDurationUntil::Triggered => write!(f, "triggered"),
        }
    }
}

impl FromStr for CastingDurationUntil {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dispelled" => Ok(CastingDurationUntil::Dispelled),
            "triggered" => Ok(CastingDurationUntil::Triggered),
            _ => Err(format!("Unrecognized casting duration until \"{}\".", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::spell::{CastingDurationUntil, Duration, SpellDuration};

    #[test]
    fn one_day() {
        let duration = SpellDuration::new(false, Duration::Days(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("1 day".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn one_hour() {
        let duration = SpellDuration::new(false, Duration::Hours(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("1 hour".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn one_minute() {
        let duration = SpellDuration::new(false, Duration::Minutes(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("1 minute".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn one_round() {
        let duration = SpellDuration::new(false, Duration::Rounds(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("1 round".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn ten_days() {
        let duration = SpellDuration::new(false, Duration::Days(10));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("10 days".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn ten_minutes() {
        let duration = SpellDuration::new(false, Duration::Minutes(10));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("10 minutes".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn twenty_four_hours() {
        let duration = SpellDuration::new(false, Duration::Hours(24));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("24 hours".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn thirty_days() {
        let duration = SpellDuration::new(false, Duration::Days(30));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("30 days".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn seven_days() {
        let duration = SpellDuration::new(false, Duration::Days(7));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("7 days".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn eight_hours() {
        let duration = SpellDuration::new(false, Duration::Hours(8));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("8 hours".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_1_day() {
        let duration = SpellDuration::new(true, Duration::UpToDays(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 1 day".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_1_hour() {
        let duration = SpellDuration::new(true, Duration::UpToHours(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 1 hour".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_1_minute() {
        let duration = SpellDuration::new(true, Duration::UpToMinutes(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 1 minute".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_1_round() {
        let duration = SpellDuration::new(true, Duration::UpToRounds(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 1 round".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_10_minutes() {
        let duration = SpellDuration::new(true, Duration::UpToMinutes(10));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 10 minutes".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_2_hours() {
        let duration = SpellDuration::new(true, Duration::UpToHours(2));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 2 hours".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_24_hours() {
        let duration = SpellDuration::new(true, Duration::UpToHours(24));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 24 hours".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_6_rounds() {
        let duration = SpellDuration::new(true, Duration::UpToRounds(6));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 6 rounds".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn concentration_up_to_8_hours() {
        let duration = SpellDuration::new(true, Duration::UpToHours(8));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Concentration, up to 8 hours".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn instantaneous() {
        let duration = SpellDuration::new(false, Duration::Instantaneous);
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Instantaneous".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn special() {
        let duration = SpellDuration::new(false, Duration::Special);
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Special".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn up_to_1_hour() {
        let duration = SpellDuration::new(false, Duration::UpToHours(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Up to 1 hour".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn up_to_1_minute() {
        let duration = SpellDuration::new(false, Duration::UpToMinutes(1));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Up to 1 minute".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn up_to_8_hours() {
        let duration = SpellDuration::new(false, Duration::UpToHours(8));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Up to 8 hours".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn until_dispelled() {
        let duration = SpellDuration::new(false, Duration::Until(CastingDurationUntil::Dispelled.into()));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Until dispelled".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn until_triggered() {
        let duration = SpellDuration::new(false, Duration::Until(CastingDurationUntil::Triggered.into()));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Until triggered".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
    #[test]
    fn until_dispelled_or_triggered() {
        let duration = SpellDuration::new(false, Duration::Until(vec![CastingDurationUntil::Dispelled, CastingDurationUntil::Triggered].into()));
        let json = serde_json::to_value(&duration).expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Until dispelled or triggered".to_string()), json);
        let deserialized = serde_json::from_value(json).expect("Unexpected error while deserializing.");
        assert_eq!(duration, deserialized)
    }
}