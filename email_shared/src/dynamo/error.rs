use serde::de::{Expected, Unexpected};
use thiserror::Error;

/// Alias for a Result with the error type `DeserializeError`.
pub type Result<T> = std::result::Result<T, DeserializeError>;

/// This type represents errors that can occur when serializing to or deserializing from DynamoDB.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DeserializeError {
    #[error("Custom({0})")]
    Custom(String),
    #[error("FieldDuplicated({0})")]
    FieldDuplicated(String),
    #[error("FieldMissing({0})")]
    FieldMissing(String),
    #[error("Parse({0})")]
    Parse(String),
    #[error("InvalidLength({0}, {1})")]
    InvalidLength(usize, String),
    #[error("InvalidType({expected}, {unexpected})")]
    InvalidType {
        expected: String,
        unexpected: String,
    },
    #[error("InvalidValue({expected}, {unexpected})")]
    InvalidValue {
        expected: String,
        unexpected: String,
    },
    #[error("UnknownField({0})")]
    UnknownField(String),
}

impl serde::de::Error for DeserializeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }

    fn duplicate_field(field: &'static str) -> Self {
        Self::FieldDuplicated(field.to_owned())
    }

    fn invalid_length(len: usize, exp: &dyn Expected) -> Self {
        Self::InvalidLength(len, exp.to_string())
    }

    fn invalid_type(unexp: Unexpected<'_>, exp: &dyn Expected) -> Self {
        Self::InvalidType {
            expected: exp.to_string(),
            unexpected: unexp.to_string(),
        }
    }

    fn invalid_value(unexp: Unexpected<'_>, exp: &dyn Expected) -> Self {
        Self::InvalidValue {
            expected: exp.to_string(),
            unexpected: unexp.to_string(),
        }
    }

    fn missing_field(field: &'static str) -> Self {
        Self::FieldMissing(field.to_owned())
    }

    fn unknown_field(field: &str, _expected: &'static [&'static str]) -> Self {
        Self::UnknownField(field.to_owned())
    }
}
