use std::{error::Error, fmt::Display};

/// An error that occurred while parsing arguments,
/// either CLI, environment variables, or provided.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub enum ArgParseError {
    /// A flag was encountered that wasn't registered.
    UnknownFlag(String),
    /// A flag expected an accompanying value, but none was given.
    MissingValue(String),
    /// Multiple short flags in a cluster (e.g. `-abc`) tried to consume the
    /// same value (e.g. `-abc only_one_value`).
    ConsumedValue(String),
}

impl Display for ArgParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownFlag(s) => write!(f, "Unknown flag: `{s}`"),
            Self::MissingValue(s) => write!(f, "Expected value for `{s}`"),
            Self::ConsumedValue(s) => write!(
                f,
                "Multiple arguments in `{s}` tried to consume the same value"
            ),
        }
    }
}

impl Error for ArgParseError {}
