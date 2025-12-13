#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::float_cmp)]

pub mod prelude;

use std::env;
use std::marker::PhantomData;
use std::ops::Deref;

#[cfg(feature = "macros")]
pub mod macros;

pub mod tag;
use tag::Full;

mod error;
pub use error::ArgParseError;

#[cfg(feature = "help")]
mod help;
#[cfg(feature = "help")]
use help::DocParams;

mod types;
pub use types::{ArgResult, ArgumentType, DefaultedArgResult};

#[cfg(test)]
mod test;

#[derive(Clone, Debug)]
#[allow(clippy::option_option)]
struct InternalArgument {
    tag: Full,
    consumes: bool,
    val: Option<Option<String>>,
}

/// The results of [`ArgumentReader::parse`]. Used both for retrieving
/// [`ArgumentRef`]s and for accessing the
/// [remainder](Arguments::remainder) of the input arguments.
///
/// `Arguments` implements `Deref<Target = [String]>`, so you can treat it
/// like a `&[String]`.
#[derive(Clone, Debug)]
pub struct Arguments {
    args: Vec<InternalArgument>,
    remainder: Vec<String>,
}

impl AsRef<[String]> for Arguments {
    fn as_ref(&self) -> &[String] {
        self.remainder.as_slice()
    }
}

// TODO: should there be AsRef AND Deref AND From?
impl Deref for Arguments {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        self.remainder.as_slice()
    }
}

impl From<Arguments> for Vec<String> {
    fn from(args: Arguments) -> Vec<String> {
        args.remainder
    }
}

impl Arguments {
    /// All the CLI arguments that didn't get parsed as part of an argument.
    ///
    /// `Arguments` implements `Deref<Target = [String]>`, so you can also just
    /// treat it like a `&[String]`. This is just to give you an explicit way
    /// to do so.
    pub fn remainder(&self) -> &[String] {
        self
    }

    pub(crate) fn get_arg(&self, i: usize) -> &InternalArgument {
        &self.args[i]
    }
}

/// An internal tag to an argument. Use this to retrieve the value of an
/// argument from an [`Arguments`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentRef<T: ArgumentType> {
    i: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<T: ArgumentType> ArgumentRef<T> {
    /// Retrieve the value of the argument from an [`Arguments`].
    ///
    /// # Errors
    ///
    /// If the argument type fails to parse,
    /// this will return that argument types error.
    /// If there was no value given to the argument,
    /// returns `None`.
    ///
    /// For `String` and `bool`, this can never fail.
    pub fn get(&self, args: &Arguments) -> ArgResult<T> {
        if let Some(val) = &args.get_arg(self.i).val {
            T::from_value(val.as_deref())
        } else {
            T::default_value().map(Ok)
        }
    }

    /// Retrieve the tag of the argument from an [`Arguments`].
    ///
    /// Note that this always returns a [`Full`] tag, even when the argument
    /// wasn't created with one.
    pub fn tag<'a>(&self, args: &'a Arguments) -> &'a Full {
        &args.get_arg(self.i).tag
    }
}

/// The structure that actually reads all your arguments.
///
/// Use [`ArgumentReader::add`] to register arguments and get [`ArgumentRef`]s.
/// Then, use <code>[ArgumentReader::parse]{_cli,_env,_provided}</code> to get
/// [`Arguments`], which contains the results of parsing. Finally, you can use
/// [`ArgumentRef::get`] to retrieve the values of your arguments.
#[derive(Debug, Clone, Default)]
#[allow(clippy::doc_markdown)]
pub struct ArgumentReader {
    args: Vec<InternalArgument>,

    /// Program-level documentation.
    ///
    /// Only available on feature `help`.
    pub doc: Option<String>,
}

impl ArgumentReader {
    /// Returns an empty [`ArgumentReader`].
    pub fn new() -> Self {
        Self { args: Vec::new(), doc: None }
    }

    /// Returns help for all the arguments.
    ///
    /// Only available on feature `help`.
    ///
    /// # Panics
    ///
    /// If the name of the executable could not be found, panics.
    #[cfg(feature = "help")]
    pub fn help(&self) -> String {
        let exe = option_env!("CARGO_BIN_NAME")
            .map(String::from)
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|s| s.file_stem().map(|s| s.to_string_lossy().into_owned()))
            })
            .expect("failed to get executable");

        let mut out = String::new();
        out.push_str(&exe);
        out.push_str(" [options...] <arguments...>\n");

        if let Some(doc) = &self.doc {
            out.push_str(doc);
            out.push_str("\n\n");
        }

        let mut params = DocParams::default();
        for arg in &self.args {
            help::update_params(&mut params, &arg.tag);
        }

        for arg in &self.args {
            out.push_str(&help::render_argument(&arg.tag, params));
            out.push('\n');
        }

        out
    }

    /// Prints help for all the arguments.
    ///
    /// Only available on feature `help`.
    ///
    /// # Panics
    ///
    /// If the name of the executable could not be found, panics.
    #[cfg(feature = "help")]
    pub fn print_help(&self) {
        print!("{}", self.help());
    }

    /// Adds an argument to the parser.
    pub fn add<T: ArgumentType>(&mut self, tag: Full) -> ArgumentRef<T> {
        let arg = InternalArgument {
            tag,
            consumes: T::CONSUMES,
            val: None,
        };

        let i = self.args.len();
        self.args.push(arg);

        ArgumentRef {
            i,
            _marker: PhantomData,
        }
    }

    /// Parse arguments from `std::env::{args,vars}`.
    ///
    /// # Errors
    ///
    /// If any arguments fail to parse their values, this
    /// will forward that error. Otherwise, see
    /// [`ArgParseError`] for a list of all possible errors.
    pub fn parse(self) -> Result<Arguments, ArgParseError> {
        self.parse_provided(env::args(), env::vars())
    }

    /// Parse from the provided environment variables and CLI arguments.
    ///
    /// # Errors
    ///
    /// If any arguments fail to parse their values, this
    /// will forward that error. Otherwise, see
    /// [`ArgParseError`] for a list of all possible errors.
    pub fn parse_provided<
        A: AsRef<str>,
        IA: IntoIterator<Item = A>,
        K: AsRef<str>,
        V: AsRef<str>,
        IE: IntoIterator<Item = (K, V)>,
    >(
        mut self,
        cli: IA,
        env: IE,
    ) -> Result<Arguments, ArgParseError> {
        self.parse_env(env);
        self.parse_cli(cli)
    }

    /// Parse the provided arguments as environment variables.
    fn parse_env<K: AsRef<str>, V: AsRef<str>, I: IntoIterator<Item = (K, V)>>(&mut self, args: I) {
        let mut env_args: Vec<_> = self
            .args
            .iter_mut()
            .filter(|arg| arg.tag.has_env())
            .collect();

        if !env_args.is_empty() {
            for (key, val) in args {
                let key_ref = key.as_ref();
                let val = val.as_ref();
                if let Some(arg) = env_args
                    .iter_mut()
                    .find(|arg| arg.tag.env.as_ref().is_some_and(|env| env == key_ref))
                {
                    arg.val = Some(Some(val.to_string()));
                }
            }
        }
    }

    /// Parses the provided arguments as if they were from the CLI.
    ///
    /// If `reset == true`, clears the values of all arguments beforehand.
    /// You probably want to leave this at `false`, unless you're re-using
    /// your parser.
    ///
    /// # Errors
    ///
    /// See [`parse`](ArgumentReader::parse) for details.
    fn parse_cli<A: AsRef<str>, IA: IntoIterator<Item = A>>(
        mut self,
        args: IA,
    ) -> Result<Arguments, ArgParseError> {
        fn tostring<S: AsRef<str>>(arg: S) -> String {
            <S as AsRef<str>>::as_ref(&arg).to_string()
        }

        let mut args = args.into_iter().peekable();
        let mut remainder = Vec::new();

        while let Some(arg) = args.next() {
            let arg = arg.as_ref();
            if let Some(mut long) = arg.strip_prefix("--") {
                let val = if let Some((left, right)) = long.split_once('=') {
                    long = left;
                    Some(right)
                } else {
                    None
                };

                let arg = self
                    .args
                    .iter_mut()
                    .find(|arg| arg.tag.matches_long(long))
                    .ok_or(ArgParseError::UnknownFlag(long.to_string()))?;

                let val = if arg.consumes {
                    if val.is_none() {
                        args.next().map(tostring)
                    } else {
                        val.map(tostring)
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
                    for short in shorts.chars() {
                        let arg = self
                            .args
                            .iter_mut()
                            .find(|arg| arg.tag.matches_short(short))
                            .ok_or(ArgParseError::UnknownFlag(short.to_string()))?;

                        if arg.consumes && consumed {
                            return Err(ArgParseError::ConsumedValue(shorts.to_string()));
                        }

                        let next = if arg.consumes {
                            consumed = true;
                            args.next().map(|arg| arg.as_ref().to_string())
                        } else {
                            None
                        };

                        arg.val = Some(next);
                    }
                }
            } else {
                remainder.push(arg.to_string());
            }
        }

        Ok(Arguments {
            args: self.args,
            remainder,
        })
    }
}
