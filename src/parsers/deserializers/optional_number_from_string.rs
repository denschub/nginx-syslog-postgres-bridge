use std::str::FromStr;

use serde::{Deserialize, Deserializer};

// This is heavily "inspired" from serde-aux' implementation at
// https://github.com/vityafx/serde-aux/blob/c6f8482f51da7f187ecea62931c8f38edcf355c9/src/field_attributes.rs#L208
// I've left out the Null option, because I know that the field always will be
// there in my case, and it will always be a string, even if it's empty.
pub fn optional_number_from_string<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: std::fmt::Display,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Contents<'a, T> {
        Str(&'a str),
        FromStr(T),
    }

    match Contents::<T>::deserialize(deserializer)? {
        Contents::Str(s) => match s {
            "" => Ok(None),
            _ => T::from_str(s).map(Some).map_err(serde::de::Error::custom),
        },
        Contents::FromStr(i) => Ok(Some(i)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestStructInt {
        #[serde(deserialize_with = "optional_number_from_string")]
        v: Option<i32>,
    }

    #[derive(Debug, Deserialize)]
    struct TestStructFloat {
        #[serde(deserialize_with = "optional_number_from_string")]
        v: Option<f32>,
    }

    #[test]
    fn parses_int_correctly() {
        let json = r#"{"v": "123"}"#;
        let deserialized: TestStructInt = serde_json::from_str(json).unwrap();
        assert_eq!(123, deserialized.v.unwrap());
    }

    #[test]
    fn parses_float_correctly() {
        let json = r#"{"v": "123.45"}"#;
        let deserialized: TestStructFloat = serde_json::from_str(json).unwrap();
        assert_eq!(123.45, deserialized.v.unwrap());
    }

    #[test]
    fn is_err_for_junk() {
        let json = r#"{"v": "aaa"}"#;
        let deserialized = serde_json::from_str::<TestStructFloat>(json);
        assert!(deserialized.is_err());
    }
}
