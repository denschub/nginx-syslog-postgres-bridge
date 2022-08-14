use serde::{Deserialize, Deserializer};

pub fn optional_normalized_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.trim();

    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "optional_normalized_string")]
        string: Option<String>,
    }

    #[test]
    fn is_none_for_empty_string() {
        let json = r#"{"string": ""}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert!(deserialized.string.is_none());
    }

    #[test]
    fn is_none_for_whitespace_string() {
        let json = r#"{"string": "  "}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert!(deserialized.string.is_none());
    }

    #[test]
    fn is_some_for_filled_string() {
        let json = r#"{"string": "example"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert!(deserialized.string.is_some());
    }

    #[test]
    fn returns_trimmed_string() {
        let json = r#"{"string": "  example  "}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!("example", deserialized.string.unwrap());
    }
}
