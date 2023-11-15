#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]

pub mod custom;
pub mod prelude;

use std::env;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::num::{ParseFloatError, ParseIntError};
use std::sync::{Arc, Mutex};

pub mod tag;
use tag::Full;

mod error;
pub use error::ArgParseError;

mod types;
use types::{ArgumentType, ArgumentValueType};

#[cfg(test)]
mod test;

/// A value passed to an argument. This will always
/// match the value returned from [`ArgumentType::arg_type`].
///
/// When implementing your own types, you
/// almost always want [`ArgumentValue::String`].
#[derive(Clone, Debug, PartialEq)]
pub enum ArgumentValue {
    Bool(bool),
    String(String),
    I64(i64),
    U64(u64),
    Float(f64),
}

impl ArgumentValue {
    pub fn typ(&self) -> ArgumentValueType {
        match self {
            Self::Bool(_) => ArgumentValueType::Bool,
            Self::String(_) => ArgumentValueType::String,
            Self::I64(_) => ArgumentValueType::I64,
            Self::U64(_) => ArgumentValueType::U64,
            Self::Float(_) => ArgumentValueType::Float,
        }
    }
}

#[derive(Clone, Debug)]
struct InternalArgument {
    tag: Full,
    typ: ArgumentValueType,
    val: Option<ArgumentValue>,
}

/// A reference to an argument. Use this to
/// retrieve the value of an argument.
pub struct ArgumentRef<'a, T: ArgumentType> {
    parser: &'a ArgumentParser,
    i: usize,
    _marker: PhantomData<T>,
}

impl<'a, T: ArgumentType> ArgumentRef<'a, T> {
    /// Retrieve the value of the argument.
    /// Consumes the `ArgumentRef`.
    ///
    /// # Errors
    ///
    /// If the argument type fails to parse,
    /// this will return that argument types error.
    ///
    /// For `String` and `bool`, this can never fail.
    pub fn get(self) -> Result<T, T::Error> {
        self.get_keep()
    }

    /// Retrieve the value of the argument.
    /// Does not consume the `ArgumentRef`.
    ///
    /// # Errors
    ///
    /// See [`get`] for possible errors.
    #[allow(clippy::missing_panics_doc)]
    pub fn get_keep(&self) -> Result<T, T::Error> {
        let args = self.parser.args.lock().unwrap();

        let Some(val) = args[self.i].clone().val else {
            return if let Some(default) = T::default_value() {
                Ok(default)
            } else {
                Err(T::Error::default())
            };
        };

        T::from_value(val)
    }
}

/// The structure that actually parses all your
/// arguments. Use [`ArgumentParser::add`] to
/// register arguments and get [`ArgumentRef`]s.
///
/// Internally, the parser is a shambling heap of
/// `Arc<Mutex<HeapAllocatedValue>>`. This is to
/// enable the current API in a thread-safe manner.
/// In the future, there may be a single-threaded
/// feature that disables the extra synchronization
/// primitives, at the risk of possible memory
/// unsafety.
#[derive(Debug, Clone, Default)]
pub struct ArgumentParser {
    args: Arc<Mutex<Vec<InternalArgument>>>,
    binary: Arc<Mutex<Option<String>>>,
}

impl ArgumentParser {
    /// Returns an empty [`ArgumentParser`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an argument to the parser.
    #[allow(clippy::missing_panics_doc)]
    pub fn add<T: ArgumentType>(&self, tag: Full) -> ArgumentRef<T> {
        let typ = T::arg_type();
        let arg = InternalArgument {
            tag,
            typ,
            val: None,
        };

        let mut args = self.args.lock().unwrap();
        let i = args.len();
        args.push(arg);

        ArgumentRef {
            parser: self,
            i,
            _marker: PhantomData,
        }
    }

    /// Retrieves the binary, if any.
    #[allow(clippy::missing_panics_doc)]
    pub fn binary(&self) -> Option<String> {
        self.binary.lock().unwrap().clone()
    }

    /// Parse arguments from `std::env::{args,vars}`.
    ///
    /// # Errors
    ///
    /// If any arguments fail to parse their values, this
    /// will forward that error. Otherwise, see
    /// [`ArgParseError`] for a list of all possible errors.
    pub fn parse(&self) -> Result<Vec<String>, ArgParseError> {
        self.parse_provided(env::args().collect::<Vec<_>>().as_slice(), env::vars())
    }

    /// Parse from the provided environment variables and CLI arguments.
    ///
    /// # Errors
    ///
    /// See [`parse`] for details.
    pub fn parse_provided<I: Iterator<Item = (String, String)>>(
        &self,
        cli: &[String],
        env: I,
    ) -> Result<Vec<String>, ArgParseError> {
        self.parse_env(env, true)?;
        self.parse_cli(cli, false)
    }

    /// Parse the provided arguments as if they were environment variables.
    ///
    /// If `reset == true`, clears the values of all arguments beforehand.
    /// You probably want to leave this at `false`, unless you're re-using
    /// your parser.
    ///
    /// # Errors
    ///
    /// See [`parse`] for details.
    #[allow(clippy::missing_panics_doc)]
    pub fn parse_env<I: Iterator<Item = (String, String)>>(
        &self,
        args: I,
        reset: bool,
    ) -> Result<(), ArgParseError> {
        let mut env_args = self.args.lock().unwrap();

        if reset {
            for arg in env_args.iter_mut() {
                arg.val = None;
            }
        }

        let mut env_args: Vec<_> = env_args
            .iter_mut()
            .filter(|arg| arg.tag.has_env())
            .collect();

        if env_args.is_empty() {
            return Ok(());
        }

        for (key, val) in args {
            let key_ref = &key;
            if let Some(arg) = env_args
                .iter_mut()
                .find(|arg| arg.tag.env.as_ref().is_some_and(|env| env == key_ref))
            {
                Self::parse_arg(arg, Some(val), key)?;
            }
        }

        Ok(())
    }

    /// Parses the provided arguments as if they were from the CLI.
    ///
    /// If `reset == true`, clears the values of all arguments beforehand.
    /// You probably want to leave this at `false`, unless you're re-using
    /// your parser.
    ///
    /// # Errors
    ///
    /// See [`parse`] for details.
    #[allow(clippy::missing_panics_doc)]
    pub fn parse_cli(&self, args: &[String], reset: bool) -> Result<Vec<String>, ArgParseError> {
        let mut args = args.iter();
        *self.binary.lock().unwrap() = args.next().cloned();

        let mut remainder = Vec::new();

        if reset {
            for arg in self.args.lock().unwrap().iter_mut() {
                arg.val = None;
            }
        }

        while let Some(arg) = args.next() {
            if let Some(mut long) = arg.strip_prefix("--") {
                let val = if let Some((left, right)) = long.split_once('=') {
                    long = left;
                    Some(right.to_string())
                } else {
                    args.next().cloned()
                };

                let mut pre_args = self.args.lock().unwrap();
                let arg = pre_args
                    .iter_mut()
                    .find(|arg| arg.tag.matches_long(long))
                    .ok_or(ArgParseError::UnknownFlag(long.to_string()))?;

                Self::parse_arg(arg, val, long.to_string())?;
            } else if let Some(short) = arg.strip_prefix('-') {
                if short.is_empty() {
                    remainder.push(String::from("-"));
                } else {
                    let mut consumed = false;
                    let mut pre_args = self.args.lock().unwrap();
                    for short in short.chars() {
                        let arg = pre_args
                            .iter_mut()
                            .find(|arg| arg.tag.matches_short(short))
                            .ok_or(ArgParseError::UnknownFlag(short.to_string()))?;

                        match arg.typ {
                            ArgumentValueType::Bool => arg.val = Some(ArgumentValue::Bool(true)),
                            ArgumentValueType::I64 => {
                                if consumed {
                                    return Err(ArgParseError::ConsumedValue(short.to_string()));
                                }

                                consumed = true;
                                arg.val = Some(ArgumentValue::I64(
                                    args.next()
                                        .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                        .parse()
                                        .map_err(|e: ParseIntError| {
                                            ArgParseError::InvalidInteger(e.to_string())
                                        })?,
                                ));
                            }
                            ArgumentValueType::U64 => {
                                if consumed {
                                    return Err(ArgParseError::ConsumedValue(short.to_string()));
                                }

                                consumed = true;
                                arg.val = Some(ArgumentValue::U64(
                                    args.next()
                                        .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                        .parse()
                                        .map_err(|e: ParseIntError| {
                                            ArgParseError::InvalidUnsignedInteger(e.to_string())
                                        })?,
                                ));
                            }
                            ArgumentValueType::Float => {
                                if consumed {
                                    return Err(ArgParseError::ConsumedValue(short.to_string()));
                                }

                                consumed = true;
                                arg.val = Some(ArgumentValue::Float(
                                    args.next()
                                        .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                        .parse()
                                        .map_err(|e: ParseFloatError| {
                                            ArgParseError::InvalidInteger(e.to_string())
                                        })?,
                                ));
                            }
                            ArgumentValueType::String => {
                                if consumed {
                                    return Err(ArgParseError::ConsumedValue(short.to_string()));
                                }

                                consumed = true;
                                arg.val = Some(ArgumentValue::String(
                                    args.next()
                                        .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                        .clone(),
                                ));
                            }
                        }
                    }
                }
            } else {
                remainder.push(arg.clone());
            }
        }

        Ok(remainder)
    }

    fn parse_arg<S: AsRef<str>>(
        arg: &mut InternalArgument,
        val: Option<S>,
        name: String,
    ) -> Result<(), ArgParseError> {
        match arg.typ {
            ArgumentValueType::Bool => {
                if let Some(val) = val {
                    let val = val.as_ref().trim();
                    if val == "0" || val == "false" {
                        arg.val = Some(ArgumentValue::Bool(false));
                    } else {
                        arg.val = Some(ArgumentValue::Bool(true));
                    }
                } else {
                    arg.val = Some(ArgumentValue::Bool(true));
                }
            }
            ArgumentValueType::I64 => {
                arg.val = Some(ArgumentValue::I64(
                    val.ok_or(ArgParseError::MissingValue(name))?
                        .as_ref()
                        .parse()
                        .map_err(|e: ParseIntError| ArgParseError::InvalidInteger(e.to_string()))?,
                ));
            }
            ArgumentValueType::U64 => {
                arg.val = Some(ArgumentValue::U64(
                    val.ok_or(ArgParseError::MissingValue(name))?
                        .as_ref()
                        .parse()
                        .map_err(|e: ParseIntError| {
                            ArgParseError::InvalidUnsignedInteger(e.to_string())
                        })?,
                ));
            }
            ArgumentValueType::Float => {
                arg.val = Some(ArgumentValue::Float(
                    val.ok_or(ArgParseError::MissingValue(name))?
                        .as_ref()
                        .parse()
                        .map_err(|e: ParseFloatError| ArgParseError::InvalidFloat(e.to_string()))?,
                ));
            }
            ArgumentValueType::String => {
                arg.val = Some(ArgumentValue::String(
                    val.ok_or(ArgParseError::MissingValue(name))?
                        .as_ref()
                        .to_string(),
                ));
            }
        }

        Ok(())
    }
}
