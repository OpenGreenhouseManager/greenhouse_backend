use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::{fmt, num::NonZeroU32, str::FromStr};

#[derive(Deserialize, Serialize, Debug)]
pub struct PromQuery {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub sub_property: Option<String>,
    #[serde(default)]
    pub step: Step,
}

/// Resolution of a Prometheus range query.
///
/// (De)serializes as a Prometheus duration string, e.g. `"15s"`, `"5m"`, `"1h"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Seconds(NonZeroU32),
    Minutes(NonZeroU32),
    Hours(NonZeroU32),
    Days(NonZeroU32),
    Weeks(NonZeroU32),
}

impl Default for Step {
    fn default() -> Self {
        Step::Seconds(NonZeroU32::new(15).unwrap())
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Step::Seconds(n) => write!(f, "{n}s"),
            Step::Minutes(n) => write!(f, "{n}m"),
            Step::Hours(n) => write!(f, "{n}h"),
            Step::Days(n) => write!(f, "{n}d"),
            Step::Weeks(n) => write!(f, "{n}w"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseStepError(String);

impl fmt::Display for ParseStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid step '{}', expected a positive integer followed by s, m, h, d or w (e.g. \"15s\")",
            self.0
        )
    }
}

impl std::error::Error for ParseStepError {}

impl FromStr for Step {
    type Err = ParseStepError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || ParseStepError(s.to_string());
        let unit_at = s.len().checked_sub(1).ok_or_else(err)?;
        let (amount, unit) = s.split_at_checked(unit_at).ok_or_else(err)?;
        let amount = amount.parse::<NonZeroU32>().map_err(|_| err())?;

        match unit {
            "s" => Ok(Step::Seconds(amount)),
            "m" => Ok(Step::Minutes(amount)),
            "h" => Ok(Step::Hours(amount)),
            "d" => Ok(Step::Days(amount)),
            "w" => Ok(Step::Weeks(amount)),
            _ => Err(err()),
        }
    }
}

impl Serialize for Step {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Step {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_displays_round_trip() {
        for s in ["15s", "1m", "5m", "2h", "1d", "1w"] {
            assert_eq!(s.parse::<Step>().unwrap().to_string(), s);
        }
    }

    #[test]
    fn rejects_invalid_steps() {
        for s in ["", "s", "15", "0s", "-5m", "1.5h", "15ms", "1h30m", "5 m"] {
            assert!(s.parse::<Step>().is_err(), "{s} should be rejected");
        }
    }

    #[test]
    fn default_is_15_seconds() {
        assert_eq!(Step::default().to_string(), "15s");
    }

    #[test]
    fn serde_uses_duration_string() {
        assert_eq!(serde_json::to_string(&Step::default()).unwrap(), "\"15s\"");
        assert_eq!(
            serde_json::from_str::<Step>("\"5m\"").unwrap(),
            Step::Minutes(NonZeroU32::new(5).unwrap())
        );
        assert!(serde_json::from_str::<Step>("\"5x\"").is_err());
    }

    #[test]
    fn missing_step_falls_back_to_default() {
        let query: PromQuery = serde_json::from_str(
            r#"{"start":"2026-01-01T00:00:00Z","end":"2026-01-02T00:00:00Z","sub_property":null}"#,
        )
        .unwrap();
        assert_eq!(query.step, Step::default());
    }
}
