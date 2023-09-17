use std::{error::Error, fmt::Display};

/// A parsing error.
#[derive(Debug, Clone)]
pub enum ArgParseError {
    InvalidInteger(String),
    InvalidUnsignedInteger(String),
    InvalidFloat(String),
    InvalidList(String),
    UnknownFlag(String),
    UnexpectedArgument(String),
    MissingValue(String),
    ConsumedValue(String),
}

impl Display for ArgParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInteger(s) => write!(f, "Invalid integer: `{s}`"),
            Self::InvalidUnsignedInteger(s) => write!(f, "Invalid unsigned integer: `{s}`"),
            Self::InvalidFloat(s) => write!(f, "Invalid float: `{s}`"),
            Self::InvalidList(s) => write!(f, "Invalid list: `{s}`"),
            Self::UnknownFlag(s) => write!(f, "Unknown flag: `{s}`"),
            Self::UnexpectedArgument(s) => write!(f, "Unexpected argument: `{s}`"),
            Self::MissingValue(s) => write!(f, "Expected value for `{s}`"),
            Self::ConsumedValue(s) => write!(
                f,
                "Multiple arguments in `{s}` tried to consume the same value"
            ),
        }
    }
}

impl Error for ArgParseError {}
