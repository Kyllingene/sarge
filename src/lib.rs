#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::float_cmp)]

pub mod prelude;

use std::env;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

#[cfg(feature = "macros")]
pub mod macros;

pub mod tag;
use tag::Full;

mod error;
pub use error::ArgParseError;

mod types;
pub use types::{ArgResult, ArgumentType};

#[cfg(test)]
mod test;

#[derive(Clone, Debug)]
#[allow(clippy::option_option)]
struct InternalArgument {
    tag: Full,
    consumes: bool,
    val: Option<Option<String>>,
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
    /// If there was no value given to the argument,
    /// returns `None`.
    ///
    /// For `String` and `bool`, this can never fail.
    pub fn get(self) -> Option<Result<T, T::Error>> {
        self.get_keep()
    }

    /// Retrieve the value of the argument.
    /// Does not consume the `ArgumentRef`.
    ///
    /// # Errors
    ///
    /// See [`get`](ArgumentRef::get) for possible errors.
    #[allow(clippy::missing_panics_doc)]
    pub fn get_keep(&self) -> ArgResult<T> {
        let args = self.parser.args.lock().unwrap();

        if let Some(val) = &args[self.i].val {
            T::from_value(val.as_deref())
        } else {
            T::default_value().map(Ok)
        }
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
        let arg = InternalArgument {
            tag,
            consumes: T::CONSUMES,
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
    /// See [`parse`](ArgumentParser::parse) for details.
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
    /// See [`parse`](ArgumentParser::parse) for details.
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
                arg.val = Some(Some(val));
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
    /// See [`parse`](ArgumentParser::parse) for details.
    #[allow(clippy::missing_panics_doc)]
    pub fn parse_cli(&self, args: &[String], reset: bool) -> Result<Vec<String>, ArgParseError> {
        let mut args = args.iter().peekable();
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
                    Some(right)
                } else {
                    None
                };

                let mut pre_args = self.args.lock().unwrap();
                let arg = pre_args
                    .iter_mut()
                    .find(|arg| arg.tag.matches_long(long))
                    .ok_or(ArgParseError::UnknownFlag(long.to_string()))?;

                let val = if arg.consumes {
                    if val.is_none() {
                        args.next().cloned()
                    } else {
                        val.map(str::to_string)
                    }
                } else {
                    None
                };

                arg.val = Some(val);
            } else if let Some(shorts) = arg.strip_prefix('-') {
                if shorts.is_empty() {
                    remainder.push(String::from("-"));
                } else {
                    let mut consumed = false;
                    let mut pre_args = self.args.lock().unwrap();
                    for short in shorts.chars() {
                        let arg = pre_args
                            .iter_mut()
                            .find(|arg| arg.tag.matches_short(short))
                            .ok_or(ArgParseError::UnknownFlag(short.to_string()))?;

                        if arg.consumes && consumed {
                            return Err(ArgParseError::ConsumedValue(shorts.to_string()));
                        }

                        let next = if arg.consumes {
                            consumed = true;
                            args.next().cloned()
                        } else {
                            None
                        };

                        arg.val = Some(next);
                    }
                }
            } else {
                remainder.push(arg.clone());
            }
        }

        Ok(remainder)
    }
}
