#![doc = include_str!("../README.md")]

pub mod prelude;
pub mod custom;

use std::{
    env,
    fmt::Debug,
    marker::PhantomData,
    num::{ParseFloatError, ParseIntError},
    sync::{Arc, Mutex},
};

pub mod tag;
use tag::Tag;

mod error;
pub use error::ArgParseError;

mod types;
use types::{ArgumentType, ArgumentValueType};

#[cfg(test)]
mod test;

/// This is an implementation detail, but due to the [`ArgumentType`] needing
/// to be public, this has to also be public.
#[doc(hidden)]
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

#[derive(Clone, Debug, PartialEq)]
struct InternalArgument {
    tag: Tag,
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
    pub fn get(self) -> Result<T, T::Error> {
        self.get_keep()
    }

    /// Retrieve the value of the argument.
    /// Does not consume the `ArgumentRef`.
    pub fn get_keep(&self) -> Result<T, T::Error> {
        // let arg = self.parser.args.lock().unwrap()[self.i].clone();
        let args = self.parser.args.lock().unwrap();
        T::from_argval(args[self.i].clone().val.ok_or_else(T::Error::default)?)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ArgumentParser {
    args: Arc<Mutex<Vec<InternalArgument>>>,
    binary: Arc<Mutex<Option<String>>>,
}

impl ArgumentParser {
    /// Returns an empty ArgumentParser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an argument to the parser.
    pub fn add<T: ArgumentType>(&self, tag: Tag) -> ArgumentRef<T> {
        let typ = T::to_argtyp();
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
    pub fn binary(&self) -> Option<String> {
        self.binary.lock().unwrap().clone()
    }

    /// Parse arguments from std::env::args.
    pub fn parse(&self) -> Result<Vec<String>, ArgParseError> {
        self.parse_args(env::args().collect::<Vec<_>>().as_slice())
    }

    /// Parses the provided arguments.
    pub fn parse_args(&self, args: &[String]) -> Result<Vec<String>, ArgParseError> {
        let mut args = args.iter();
        *self.binary.lock().unwrap() = args.next().cloned();

        let mut remainder = Vec::new();

        self.args.lock().unwrap().iter_mut().for_each(|arg| {
            arg.val = match arg.typ {
                ArgumentValueType::Bool => Some(ArgumentValue::Bool(false)),
                _ => None,
            };
        });

        while let Some(arg) = args.next() {
            if let Some(mut long) = arg.strip_prefix("--") {
                let val = if let Some((left, right)) = arg.split_once('=') {
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

                match arg.typ {
                    ArgumentValueType::Bool => arg.val = Some(ArgumentValue::Bool(true)),
                    ArgumentValueType::I64 => {
                        arg.val = Some(ArgumentValue::I64(
                            val.ok_or(ArgParseError::MissingValue(long.to_string()))?
                                .parse()
                                .map_err(|e: ParseIntError| {
                                    ArgParseError::InvalidInteger(e.to_string())
                                })?,
                        ))
                    }
                    ArgumentValueType::U64 => {
                        arg.val = Some(ArgumentValue::U64(
                            val.ok_or(ArgParseError::MissingValue(long.to_string()))?
                                .parse()
                                .map_err(|e: ParseIntError| {
                                    ArgParseError::InvalidUnsignedInteger(e.to_string())
                                })?,
                        ))
                    }
                    ArgumentValueType::Float => {
                        arg.val = Some(ArgumentValue::Float(
                            val.ok_or(ArgParseError::MissingValue(long.to_string()))?
                                .parse()
                                .map_err(|e: ParseFloatError| {
                                    ArgParseError::InvalidFloat(e.to_string())
                                })?,
                        ))
                    }
                    ArgumentValueType::String => {
                        arg.val = Some(ArgumentValue::String(
                            val.ok_or(ArgParseError::MissingValue(long.to_string()))?
                                .clone(),
                        ))
                    }
                }
            } else if let Some(short) = arg.strip_prefix('-') {
                if short.is_empty() {
                    remainder.push(String::from("-"));
                } else if short.len() == 1 {
                    let short = short.chars().next().unwrap();
                    let mut pre_args = self.args.lock().unwrap();
                    let arg = pre_args
                        .iter_mut()
                        .find(|arg| arg.tag.matches_short(short))
                        .ok_or(ArgParseError::UnknownFlag(short.to_string()))?;

                    match arg.typ {
                        ArgumentValueType::Bool => arg.val = Some(ArgumentValue::Bool(true)),
                        ArgumentValueType::I64 => {
                            arg.val = Some(ArgumentValue::I64(
                                args.next()
                                    .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                    .parse()
                                    .map_err(|e: ParseIntError| {
                                        ArgParseError::InvalidInteger(e.to_string())
                                    })?,
                            ))
                        }
                        ArgumentValueType::U64 => {
                            arg.val = Some(ArgumentValue::U64(
                                args.next()
                                    .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                    .parse()
                                    .map_err(|e: ParseIntError| {
                                        ArgParseError::InvalidUnsignedInteger(e.to_string())
                                    })?,
                            ))
                        }
                        ArgumentValueType::Float => {
                            arg.val = Some(ArgumentValue::Float(
                                args.next()
                                    .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                    .parse()
                                    .map_err(|e: ParseFloatError| {
                                        ArgParseError::InvalidFloat(e.to_string())
                                    })?,
                            ))
                        }
                        ArgumentValueType::String => {
                            arg.val = Some(ArgumentValue::String(
                                args.next()
                                    .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                    .clone(),
                            ))
                        }
                    }
                } else {
                    let mut consumed = false;
                    for short in short.chars() {
                        let mut pre_args = self.args.lock().unwrap();
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
                                ))
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
                                ))
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
                                ))
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
                                ))
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
}
