use std::fmt::{Display, Formatter};

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CastingRange {
    Sight,
    SelfRange,
    SelfAoE(Distance),
    SelfShaped(Distance, CastingRangeShape),
    Touch,
    Distance(Distance),
    Unlimited,
    Special,
}

impl Display for CastingRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CastingRange::Sight => write!(f, "Sight"),
            CastingRange::SelfRange => write!(f, "Self"),
            CastingRange::SelfAoE(distance) => write!(f, "Self ({} radius)", distance.adjective()),
            CastingRange::SelfShaped(distance, CastingRangeShape::Line) => write!(f, "Self ({} line)", distance.adjective()),
            CastingRange::SelfShaped(distance, CastingRangeShape::Cone) => write!(f, "Self ({} cone)", distance.adjective()),
            CastingRange::SelfShaped(distance, CastingRangeShape::Cube) => write!(f, "Self ({} cube)", distance.adjective()),
            CastingRange::SelfShaped(distance, CastingRangeShape::Sphere) => write!(f, "Self ({}-radius sphere)", distance.adjective()),
            CastingRange::SelfShaped(distance, CastingRangeShape::Cylinder) => write!(f, "Self ({} cylinder)", distance.adjective()),
            CastingRange::SelfShaped(distance, CastingRangeShape::Hemisphere) => write!(f, "Self ({}-radius hemisphere)", distance.adjective()),
            CastingRange::Touch => write!(f, "Touch"),
            CastingRange::Distance(distance) => write!(f, "{}", distance),
            CastingRange::Unlimited => write!(f, "Unlimited"),
            CastingRange::Special => write!(f, "Special"),
        }
    }
}

impl Serialize for CastingRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

impl<'de> Deserialize<'de> for CastingRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let range_string = String::deserialize(deserializer)?;
        if range_string == "Sight" {
            Ok(CastingRange::Sight)
        } else if range_string == "Self" {
            Ok(CastingRange::SelfRange)
        } else if range_string == "Touch" {
            Ok(CastingRange::Touch)
        } else if range_string == "Unlimited" {
            Ok(CastingRange::Unlimited)
        } else if range_string == "Special" {
            Ok(CastingRange::Special)
        } else {
            let self_range_re = Regex::new("^Self \\((?<distance>\\d+)-(?<unit>foot|mile)(-radius)? (?<shape>radius|line|cone|sphere|cube|hemisphere|cylinder)\\)$").unwrap();
            let distance_re = Regex::new("^(?<distance>\\d+) (?<unit>foot|feet|mile|miles)$").unwrap();
            let self_range_captures = self_range_re.captures(&range_string);
            let distance_captures = distance_re.captures(&range_string);
            if let Some(captures) = self_range_captures {
                match (&captures["distance"]).parse::<usize>() {
                    Ok(distance) => {
                        let unit = &captures["unit"];
                        let shape = &captures["shape"];
                        let shape = match shape {
                            "radius" => return if unit == "feet" || unit == "foot" {
                                Ok(CastingRange::SelfAoE(Distance::Feet(distance)))
                            } else if unit == "mile" || unit == "miles" {
                                Ok(CastingRange::SelfAoE(Distance::Miles(distance)))
                            } else {
                                Err(Error::custom(format!("Unsupported unit: \"{}\".", unit)))
                            },
                            "line" => CastingRangeShape::Line,
                            "cone" => CastingRangeShape::Cone,
                            "sphere" => CastingRangeShape::Sphere,
                            "cube" => CastingRangeShape::Cube,
                            "hemisphere" => CastingRangeShape::Hemisphere,
                            "cylinder" => CastingRangeShape::Cylinder,
                            _ => return Err(Error::custom(format!("Unsupported shape: \"{}\".", shape)))
                        };
                        if unit == "feet" || unit == "foot" {
                            Ok(CastingRange::SelfShaped(Distance::Feet(distance), shape))
                        } else if unit == "mile" || unit == "miles" {
                            Ok(CastingRange::SelfShaped(Distance::Miles(distance), shape))
                        } else {
                            Err(Error::custom(format!("Unsupported unit: \"{}\".", unit)))
                        }
                    }
                    Err(_) => Err(Error::custom("Distance regex matches but cannot parse distance."))
                }
            } else if let Some(captures) = distance_captures {
                match (&captures["distance"]).parse::<usize>() {
                    Ok(distance) => {
                        let unit = &captures["unit"];
                        if unit == "feet" || unit == "foot" {
                            Ok(CastingRange::Distance(Distance::Feet(distance)))
                        } else if unit == "mile" || unit == "miles" {
                            Ok(CastingRange::Distance(Distance::Miles(distance)))
                        } else {
                            Err(Error::custom(format!("Unsupported unit: \"{}\".", unit)))
                        }
                    }
                    Err(_) => Err(Error::custom("Distance regex matches but cannot parse distance."))
                }
            } else {
                Err(Error::custom(format!("Unsupported casting range: \"{}\".", range_string)))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Distance {
    Feet(usize),
    Miles(usize),
}

impl Distance {
    fn adjective(&self) -> String {
        match self {
            Distance::Feet(feet) => format!("{}-foot", feet),
            Distance::Miles(miles) => format!("{}-mile", miles)
        }
    }
}

impl Display for Distance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Distance::Feet(feet) => if *feet == 1 {
                write!(f, "{} foot", feet)
            } else {
                write!(f, "{} feet", feet)
            },
            Distance::Miles(miles) => if *miles == 1 {
                write!(f, "{} mile", miles)
            } else {
                write!(f, "{} miles", miles)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum CastingRangeShape {
    Line,
    Cone,
    Cube,
    Sphere,
    Cylinder,
    Hemisphere,
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::spell::{CastingRange, CastingRangeShape, Distance};

    #[test]
    fn one_mile() {
        let range = CastingRange::Distance(Distance::Miles(1));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("1 mile".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn ten_feet() {
        let range = CastingRange::Distance(Distance::Feet(10));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("10 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn hundred_feet() {
        let range = CastingRange::Distance(Distance::Feet(100));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("100 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn hundred_twenty_feet() {
        let range = CastingRange::Distance(Distance::Feet(120));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("120 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn gundred_fifty_feet() {
        let range = CastingRange::Distance(Distance::Feet(150));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("150 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn thirty_feet() {
        let range = CastingRange::Distance(Distance::Feet(30));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("30 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn three_hundred_feet() {
        let range = CastingRange::Distance(Distance::Feet(300));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("300 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn five_feet() {
        let range = CastingRange::Distance(Distance::Feet(5));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("5 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn five_hundred_feet() {
        let range = CastingRange::Distance(Distance::Feet(500));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("500 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn five_hundred_miles() {
        let range = CastingRange::Distance(Distance::Miles(500));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("500 miles".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn sixty_feet() {
        let range = CastingRange::Distance(Distance::Feet(60));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("60 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn ninety_feet() {
        let range = CastingRange::Distance(Distance::Feet(90));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("90 feet".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_range() {
        let range = CastingRange::SelfRange;
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_10_foot_radius() {
        let range = CastingRange::SelfAoE(Distance::Feet(10));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (10-foot radius)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_10_foot_radius_hemisphere() {
        let range = CastingRange::SelfShaped(Distance::Feet(10), CastingRangeShape::Hemisphere);
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (10-foot-radius hemisphere)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_10_foot_radius_sphere() {
        let range = CastingRange::SelfShaped(Distance::Feet(10), CastingRangeShape::Sphere);
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (10-foot-radius sphere)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_100_foot_line() {
        let range = CastingRange::SelfShaped(Distance::Feet(100), CastingRangeShape::Line);
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (100-foot line)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_15_foot_cone() {
        let range = CastingRange::SelfShaped(Distance::Feet(15), CastingRangeShape::Cone);
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (15-foot cone)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_15_foot_cube() {
        let range = CastingRange::SelfShaped(Distance::Feet(15), CastingRangeShape::Cube);
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (15-foot cube)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_15_foot_radius() {
        let range = CastingRange::SelfAoE(Distance::Feet(15));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (15-foot radius)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_30_foot_cone() {
        let range = CastingRange::SelfShaped(Distance::Feet(30), CastingRangeShape::Cone);
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (30-foot cone)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_30_foot_radius() {
        let range = CastingRange::SelfAoE(Distance::Feet(30));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (30-foot radius)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_5_foot_radius() {
        let range = CastingRange::SelfAoE(Distance::Feet(5));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (5-foot radius)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_5_mile_radius() {
        let range = CastingRange::SelfAoE(Distance::Miles(5));
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (5-mile radius)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_60_foot_cone() {
        let range = CastingRange::SelfShaped(Distance::Feet(60), CastingRangeShape::Cone);
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (60-foot cone)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn self_60_foot_line() {
        let range = CastingRange::SelfShaped(Distance::Feet(60), CastingRangeShape::Line);
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Self (60-foot line)".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn sight() {
        let range = CastingRange::Sight;
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Sight".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn special() {
        let range = CastingRange::Special;
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Special".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn touch() {
        let range = CastingRange::Touch;
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Touch".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
    #[test]
    fn unlimited() {
        let range = CastingRange::Unlimited;
        let value = serde_json::to_value(&range)
            .expect("Unexpected error while serializing.");
        assert_eq!(Value::String("Unlimited".to_string()), value);
        let deserialized = serde_json::from_value(value)
            .expect("Unexpected error while deserializing.");
        assert_eq!(range, deserialized);
    }
}