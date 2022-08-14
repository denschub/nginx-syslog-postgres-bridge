use serde::{Deserialize, Deserializer};

pub fn optional_normalized_ip<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let full_ip: String = Deserialize::deserialize(deserializer)?;
    if full_ip.is_empty() {
        Ok(None)
    } else if let Some(stripped) = full_ip.strip_prefix("::ffff:") {
        Ok(Some(stripped.to_owned()))
    } else {
        Ok(Some(full_ip))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "optional_normalized_ip")]
        ip: Option<String>,
    }

    #[test]
    fn is_none_for_empty_string() {
        let json = r#"{"ip": ""}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert!(deserialized.ip.is_none());
    }

    #[test]
    fn removes_prefix_from_ipv4_in_ipv6() {
        let json = r#"{"ip": "::ffff:127.0.0.1"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!("127.0.0.1", deserialized.ip.unwrap());
    }

    #[test]
    fn leaves_ipv4_untouched() {
        let json = r#"{"ip": "127.0.0.1"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!("127.0.0.1", deserialized.ip.unwrap());
    }

    #[test]
    fn leaves_ipv6_untouched() {
        let json = r#"{"ip": "2001:db8::ffff"}"#;
        let deserialized: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!("2001:db8::ffff", deserialized.ip.unwrap());
    }
}
