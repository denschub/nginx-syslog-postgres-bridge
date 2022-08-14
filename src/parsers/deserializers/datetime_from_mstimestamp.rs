use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer};

pub fn datetime_from_mstimestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let string: &str = Deserialize::deserialize(deserializer)?;
    let mut parts = string.split('.');

    // [ToDo]: The following two blocks feel kinda verbose and copy-paste'y.
    // There probably is a way to write this more cleanly based on some code
    // around serde::de::Visitor, but... not now.
    let seconds: i64 = parts
        .next()
        .ok_or_else(|| serde::de::Error::custom("missing seconds part"))?
        .parse()
        .map_err(serde::de::Error::custom)?;

    let millis: u32 = parts
        .next()
        .ok_or_else(|| serde::de::Error::custom("missing milliseconds part"))?
        .parse()
        .map_err(serde::de::Error::custom)?;

    Ok(DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp(seconds, millis * 1_000_000),
        Utc,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    use chrono::NaiveDate;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "datetime_from_mstimestamp")]
        pub ts: DateTime<Utc>,
    }

    #[test]
    fn parses_timestamp_correctly() {
        let json = r#"{"ts": "1660341284.123"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();

        let expected_dt = DateTime::<Utc>::from_utc(
            NaiveDate::from_ymd(2022, 8, 12).and_hms_milli(21, 54, 44, 123),
            Utc,
        );

        assert_eq!(expected_dt, deserialized.ts);
    }

    #[test]
    fn is_err_if_milliseconds_are_missing() {
        let json = r#"{"ts": "1660341284"}"#;
        let deserialized = serde_json::from_str::<TestStruct>(json);
        assert!(deserialized.is_err());
    }

    #[test]
    fn is_err_if_seconds_are_missing() {
        let json = r#"{"ts": ".123"}"#;
        let deserialized = serde_json::from_str::<TestStruct>(json);
        assert!(deserialized.is_err());
    }

    #[test]
    fn is_err_for_junk() {
        let json = r#"{"ts": "hello world"}"#;
        let deserialized = serde_json::from_str::<TestStruct>(json);
        assert!(deserialized.is_err());
    }
}
