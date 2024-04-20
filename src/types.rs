//! All interfaces for handling argument types.

use std::convert::Infallible;
use std::num::{ParseFloatError, ParseIntError};

/// The type returned when retrieving an argument.
pub type ArgResult<T> = Option<Result<T, <T as ArgumentType>::Error>>;

/// The type returned when retrieving an argument with a default value.
pub type DefaultedArgResult<T> = Result<T, <T as ArgumentType>::Error>;

/// A type that can be used as an argument.
/// Implemented for `bool`, `i64`, `u64`, `f64`,
/// `String`, and `Vec<T: ArgumentType>`.
///
/// You can implement this for your own types! It's essentially the same as
/// `FromStr`, with one crucial difference: you can specify a default value
/// (for when the argument wasn't provided) via
/// [`default_value`](ArgumentType::default_value).
///
/// If your type doesn't consume any values, such as `bool`, set `CONSUMES` to
/// false. Otherwise it defaults to true.
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

    /// Whether or not this type consumes any arguments.
    const CONSUMES: bool = true;

    /// Perform parsing on the value.
    ///
    /// If the argument doesn't take any input, `val` is None.
    #[allow(clippy::missing_errors_doc)]
    fn from_value(val: Option<&str>) -> ArgResult<Self>;

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

            fn from_value(val: Option<&str>) -> ArgResult<Self> {
                val.map(|val| val.parse())
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
    i8, ParseIntError;
    i16, ParseIntError;
    i32, ParseIntError;
    i64, ParseIntError;
    i128, ParseIntError;
    isize, ParseIntError;
    u8, ParseIntError;
    u16, ParseIntError;
    u32, ParseIntError;
    u64, ParseIntError;
    u128, ParseIntError;
    usize, ParseIntError;
    f32, ParseFloatError;
    f64, ParseFloatError;
    String, Infallible;
}

impl ArgumentType for bool {
    type Error = Infallible;

    const CONSUMES: bool = false;

    fn from_value(val: Option<&str>) -> ArgResult<Self> {
        Some(Ok(if let Some(val) = val {
            ["true", "1", "t"].contains(&val)
        } else {
            true
        }))
    }

    fn default_value() -> Option<Self> {
        Some(false)
    }
}

impl<T: ArgumentType> ArgumentType for Vec<T> {
    type Error = T::Error;

    fn from_value(val: Option<&str>) -> ArgResult<Self> {
        let bits = val?.split(',');
        let mut values = Vec::new();

        for bit in bits {
            values.push(match T::from_value(Some(bit))? {
                Ok(t) => t,
                Err(e) => return Some(Err(e)),
            });
        }

        Some(Ok(values))
    }
}
