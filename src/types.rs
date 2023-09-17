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
/// [`ArgumentValueType::String`] from `to_argtyp`,
/// and parsing a `String` into your type in
/// `from_argval`.
///
/// An example can be found in `src/test/custom_type.rs`.
pub trait ArgumentType: Sized {
    type Error: Default;

    fn to_argtyp() -> ArgumentValueType;
    fn from_argval(val: ArgumentValue) -> Result<Self, Self::Error>;
}

impl ArgumentType for bool {
    type Error = ();

    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::Bool
    }

    fn from_argval(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::Bool(b) = val {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl ArgumentType for String {
    type Error = ();

    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::String
    }

    fn from_argval(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::String(s) = val {
            Ok(s)
        } else {
            Err(())
        }
    }
}

impl ArgumentType for i64 {
    type Error = ();

    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::I64
    }

    fn from_argval(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::I64(i) = val {
            Ok(i)
        } else {
            Err(())
        }
    }
}

impl ArgumentType for u64 {
    type Error = ();

    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::U64
    }

    fn from_argval(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::U64(u) = val {
            Ok(u)
        } else {
            Err(())
        }
    }
}

impl ArgumentType for f64 {
    type Error = ();

    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::Float
    }

    fn from_argval(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::Float(f) = val {
            Ok(f)
        } else {
            Err(())
        }
    }
}

impl<T: ArgumentType> ArgumentType for Vec<T>
    where T::Error: Debug + PartialEq + 'static {
    type Error = InvalidListError;

    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::String
    }

    fn from_argval(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::String(s) = val {
            let mut bits = s.split(',').map(|s| s.to_string());
            let mut values = Vec::new();

            while let Some(bit) = bits.next() {
                match T::to_argtyp() {
                    ArgumentValueType::Bool => values.push(T::from_argval(ArgumentValue::Bool(
                        bit.parse().map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    )).map_err(|e| InvalidListError::Err(Box::new(e)))?),

                    ArgumentValueType::String => values.push(
                        T::from_argval(ArgumentValue::String(bit))
                            .map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    ),

                    ArgumentValueType::I64 => values.push(T::from_argval(ArgumentValue::I64(
                        bit.parse().map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    )).map_err(|e| InvalidListError::Err(Box::new(e)))?),

                    ArgumentValueType::U64 => values.push(T::from_argval(ArgumentValue::Bool(
                        bit.parse().map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    )).map_err(|e| InvalidListError::Err(Box::new(e)))?),

                    ArgumentValueType::Float => values.push(T::from_argval(ArgumentValue::Bool(
                        bit.parse().map_err(|e| InvalidListError::Err(Box::new(e)))?,
                    )).map_err(|e| InvalidListError::Err(Box::new(e)))?),
                }
            }

            Ok(values)
        } else {
            Err(InvalidListError::Other)
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
        match (self, other) {
            (InvalidListError::Other, InvalidListError::Other) => true,
            _ => false,
        }
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
