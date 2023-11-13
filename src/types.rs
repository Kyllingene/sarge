//! All interfaces for handling argument types.

use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::ArgumentValue;

/// All the types that `ArgumentParser` can
/// recognize. You can use [`String`][ArgumentValueType::String]
/// to parse (and therefore implement) your own type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArgumentValueType {
    Bool,
    String,
    I64,
    U64,
    Float,
}

/// A type that can be used as an argument.
/// Implemented for `bool`, `i64`, `u64`, `f64`,
/// and `String`.
///
/// You can implement this for your own types!
/// You would achieve this by returning
/// [`ArgumentValueType::String`] from `arg_type`,
/// and parsing an [`ArgumentValue::String`] into your
/// type in `from_value`.
///
/// An example can be found in `src/test/custom_type.rs`.
pub trait ArgumentType: Sized {
    /// What errors may occur during parsing.
    type Error: Default;

    /// The type of argument you would like to be given.
    /// Anything other than `String` will perform some
    /// parsing behind-the-scenes.
    fn arg_type() -> ArgumentValueType;

    /// Parse yourself from the argument type you chose in
    /// [`arg_type`](`ArgumentType::arg_type`).
    #[allow(clippy::missing_errors_doc)]
    fn from_value(val: ArgumentValue) -> Result<Self, Self::Error>;

    /// If no value was given, what the default should be, if any.
    /// This defaults to `None`.
    fn default_value() -> Option<Self> {
        None
    }
}

impl ArgumentType for bool {
    // TODO: should this be `Infallible`?
    type Error = ();

    fn arg_type() -> ArgumentValueType {
        ArgumentValueType::Bool
    }

    fn from_value(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::Bool(b) = val {
            Ok(b)
        } else {
            unreachable!("ArgumentType::from_value was given the wrong type");
        }
    }

    fn default_value() -> Option<Self> {
        Some(false)
    }
}

impl ArgumentType for String {
    type Error = ();

    fn arg_type() -> ArgumentValueType {
        ArgumentValueType::String
    }

    fn from_value(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::String(s) = val {
            Ok(s)
        } else {
            unreachable!("ArgumentType::from_value was given the wrong type");
        }
    }
}

impl ArgumentType for i64 {
    type Error = ();

    fn arg_type() -> ArgumentValueType {
        ArgumentValueType::I64
    }

    fn from_value(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::I64(i) = val {
            Ok(i)
        } else {
            unreachable!("ArgumentType::from_value was given the wrong type");
        }
    }
}

impl ArgumentType for u64 {
    type Error = ();

    fn arg_type() -> ArgumentValueType {
        ArgumentValueType::U64
    }

    fn from_value(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::U64(u) = val {
            Ok(u)
        } else {
            unreachable!("ArgumentType::from_value was given the wrong type");
        }
    }
}

impl ArgumentType for f64 {
    type Error = ();

    fn arg_type() -> ArgumentValueType {
        ArgumentValueType::Float
    }

    fn from_value(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::Float(f) = val {
            Ok(f)
        } else {
            unreachable!("ArgumentType::from_value was given the wrong type");
        }
    }
}

impl<T: ArgumentType> ArgumentType for Vec<T>
where
    T::Error: Debug + PartialEq + 'static,
{
    type Error = InvalidListError;

    fn arg_type() -> ArgumentValueType {
        ArgumentValueType::String
    }

    fn from_value(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::String(s) = val {
            let bits = s.split(',').map(|s| s.to_string());
            let mut values = Vec::new();

            for bit in bits {
                match T::arg_type() {
                    ArgumentValueType::Bool => values.push(
                        T::from_value(ArgumentValue::Bool(
                            bit.parse()
                                .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                        ))
                        .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    ),

                    ArgumentValueType::String => values.push(
                        T::from_value(ArgumentValue::String(bit))
                            .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    ),

                    ArgumentValueType::I64 => values.push(
                        T::from_value(ArgumentValue::I64(
                            bit.parse()
                                .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                        ))
                        .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    ),

                    ArgumentValueType::U64 => values.push(
                        T::from_value(ArgumentValue::Bool(
                            bit.parse()
                                .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                        ))
                        .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    ),

                    ArgumentValueType::Float => values.push(
                        T::from_value(ArgumentValue::Bool(
                            bit.parse()
                                .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                        ))
                        .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    ),
                }
            }

            Ok(values)
        } else {
            unreachable!("ArgumentType::from_value was given the wrong type");
        }
    }
}

#[derive(Debug, Default)]
pub enum InvalidListError {
    Err(Box<dyn Debug>),

    #[default]
    Other,
}

impl PartialEq for InvalidListError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (InvalidListError::Other, InvalidListError::Other)
        )
    }
}

impl Display for InvalidListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidListError::Err(e) => write!(f, "Invalid list in arguments: {e:?}"),
            InvalidListError::Other => write!(f, "Invalid list in arguments"),
        }
    }
}

impl Error for InvalidListError {}
