//! All interfaces for handling argument types.

use std::error::Error;
use std::fmt::{Debug, Display};
use std::num::{ParseIntError, ParseFloatError};
use std::convert::Infallible;

/// A type that can be used as an argument.
/// Implemented for `bool`, `i64`, `u64`, `f64`,
/// `String`, and `Vec<T: ArgumentType>`.
///
/// You can implement this for your own types! It's essentially the same as
/// [`FromStr`], with one crucial difference: you can specify a default value
/// (for when the argument wasn't provided) via [`default_value`].
///
/// The reason this isn't just `FromStr + Default` is because types like `i32`
/// implement `Default`, but few people want 0 whenever their user doesn't pass
/// in a value. In other words, `Default` has different semantics from
/// `default_value`.
///
/// An example can be found in `src/test/custom_type.rs`.
pub trait ArgumentType: Sized {
    /// A parsing error.
    type Error;

    /// Perform parsing on the value.
    #[allow(clippy::missing_errors_doc)]
    fn from_value(val: &str) -> Result<Self, Self::Error>;

    /// If no value was given, what the default should be, if any.
    /// This defaults to `None`.
    fn default_value() -> Option<Self> {
        None
    }
}

macro_rules! impl_intrinsics {
    ( $( $typ:ty, $err:ty $( => $default:block )? );+ $(;)? ) => {
        $(
        impl ArgumentType for $typ {
            type Error = $err;

            fn from_value(val: &str) -> Result<Self, Self::Error> {
                val.parse()
            }

            $(
            fn default_value() -> Option<Self> {
                $default
            }
            )?
        }
        )+
    };
}

impl_intrinsics! {
    i64, ParseIntError;
    u64, ParseIntError;
    f64, ParseFloatError;
    String, Infallible;
}

impl ArgumentType for bool {
    type Error = Infallible;

    fn from_value(val: &str) -> Result<Self, Self::Error> {
        Ok(if ["true", "1", "t"].contains(&val) {
            true
        } else {
            false
        })
    }

    fn default_value() -> Option<Self> {
        Some(false)
    }
}

impl<T: ArgumentType> ArgumentType for Vec<T> {
    type Error = T::Error;

    fn from_value(val: &str) -> Result<Self, Self::Error> {
        let bits = val.split(',');
        let mut values = Vec::new();

        for bit in bits {
            values.push(T::from_value(bit)?);
        }

        Ok(values)
    }
}

