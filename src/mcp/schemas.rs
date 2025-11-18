use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// Custom deserializer that accepts both integers and string representations of integers
fn deserialize_optional_usize<'de, D>(deserializer: D) -> Result<Option<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct OptionalUsizeVisitor;

    impl<'de> Visitor<'de> for OptionalUsizeVisitor {
        type Value = Option<usize>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer, a string representation of an integer, or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            // Deserialize the inner value, which can be an integer or string
            deserializer.deserialize_any(UsizeVisitor).map(Some)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value as usize))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value < 0 {
                return Err(E::custom(format!("negative integer not allowed: {}", value)));
            }
            Ok(Some(value as usize))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .parse::<usize>()
                .map(Some)
                .map_err(|_| E::custom(format!("invalid integer string: {}", value)))
        }
    }

    struct UsizeVisitor;

    impl<'de> Visitor<'de> for UsizeVisitor {
        type Value = usize;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an integer or a string representation of an integer")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as usize)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value < 0 {
                return Err(E::custom(format!("negative integer not allowed: {}", value)));
            }
            Ok(value as usize)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value
                .parse::<usize>()
                .map_err(|_| E::custom(format!("invalid integer string: {}", value)))
        }
    }

    deserializer.deserialize_option(OptionalUsizeVisitor)
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CargoArgs {
    pub args: Vec<String>,
    pub working_directory: String,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_usize")]
    pub from: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_usize")]
    pub to: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadParams {
    pub path: String,
    pub working_directory: String,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_usize")]
    pub from: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_usize")]
    pub to: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DescribeParams {
    pub path: String,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchParams {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TestsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ProjectParams {
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocsParams {
    pub path: String,
    pub working_directory: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefactorRenameParams {
    pub symbol: String,
    pub to: String,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefactorExtractParams {
    pub file: String,
    pub working_directory: String,
    pub from: usize,
    pub to: usize,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefactorMoveParams {
    pub symbol: String,
    pub to: String,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RefactorSignatureParams {
    pub function: String,
    pub new_signature: String,
    pub working_directory: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ClippyArgs {
    pub args: Vec<String>,
    pub working_directory: String,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_usize")]
    pub from: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none", deserialize_with = "deserialize_optional_usize")]
    pub to: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_read_params_deserialize_integer() {
        let json = r#"{"path": "test.rs", "working_directory": ".", "from": 10, "to": 20}"#;
        let params: ReadParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.from, Some(10));
        assert_eq!(params.to, Some(20));
    }

    #[test]
    fn test_read_params_deserialize_string() {
        let json = r#"{"path": "test.rs", "working_directory": ".", "from": "10", "to": "20"}"#;
        let params: ReadParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.from, Some(10));
        assert_eq!(params.to, Some(20));
    }

    #[test]
    fn test_read_params_deserialize_null() {
        let json = r#"{"path": "test.rs", "working_directory": "."}"#;
        let params: ReadParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.from, None);
        assert_eq!(params.to, None);
    }

    #[test]
    fn test_read_params_deserialize_mixed() {
        let json = r#"{"path": "test.rs", "working_directory": ".", "from": 10, "to": "20"}"#;
        let params: ReadParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.from, Some(10));
        assert_eq!(params.to, Some(20));
    }

    #[test]
    fn test_cargo_args_deserialize_string() {
        let json = r#"{"args": ["build"], "working_directory": ".", "from": "1", "to": "50"}"#;
        let params: CargoArgs = serde_json::from_str(json).unwrap();
        assert_eq!(params.from, Some(1));
        assert_eq!(params.to, Some(50));
    }

    #[test]
    fn test_clippy_args_deserialize_string() {
        let json = r#"{"args": [], "working_directory": ".", "from": "100", "to": "200"}"#;
        let params: ClippyArgs = serde_json::from_str(json).unwrap();
        assert_eq!(params.from, Some(100));
        assert_eq!(params.to, Some(200));
    }
}
