#![doc(hidden)]

pub mod const_exprs;

#[macro_export]
#[doc(hidden)]
macro_rules! __parse_arg {
    ( err => $args:expr, $name:ident, ) => {
        let $name = $name.get(&$args);
    };

    ( ok => $args:expr, $name:ident, ) => {
        let $name = $name.get(&$args).map(|a| a.ok()).flatten();
    };

    ( => $args:expr, $name:ident, ) => {
        let $name = $name
            .get(&$args)
            .expect("Tried to unwrap argument that wasn't passed")
            .expect("Tried to unwrap argument that failed to parse");
    };

    ( err => $args:expr, $name:ident, $default:expr ) => {
        let $name = $name
            .get(&$args)
            .unwrap_or_else(|| ::std::result::Result::Ok($default));
    };

    ( ok => $args:expr, $name:ident, $default:expr ) => {
        compile_error!(concat!(
            "cannot have default value for argument `#ok ",
            stringify!($name),
            " = ",
            stringify!($default),
            "`, only on `#err` or unwrapped arguments",
        ))
    };

    ( => $args:expr, $name:ident, $default:expr ) => {
        let $name = $name
            .get(&$args)
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_else(|| $default);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __arg_typ {
    ( $name:ident, err, $typ:ty, ) => {
        $crate::ArgResult<$typ>
    };

    ( $name:ident, err, $typ:ty, $default:expr ) => {
        $crate::DefaultedArgResult<$typ>
    };

    ( $name:ident, ok, $typ:ty, ) => {
        ::std::option::Option<$typ>
    };

    ( $name:ident, ok, $typ:ty, $default:expr ) => {
        compile_error!(concat!(
            "cannot have default value for argument `#ok ",
            stringify!($name),
            " = ",
            stringify!($default),
            "`, only on `#err` or unwrapped arguments",
        ))
    };

    ( $name:ident, $typ:ty, $( $default:expr )? ) => {
        $typ
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __var_tag {
    ( $long:ident $( $doc:literal )* ) => {{
        let mut s = ::std::string::String::new();
        $(
            s.push_str($doc);
            s.push('\n');
        )*
        s.pop();
        $crate::tag::long($crate::__replace!(::std::stringify!($long), '_', '-'))
            .doc(s)
    }};
    ( $short:literal $long:ident $( $doc:literal )* ) => {{
        let mut s = ::std::string::String::new();
        $(
            s.push_str($doc);
            s.push('\n');
        )*
        s.pop();
        $crate::tag::both($short, $crate::__replace!(::std::stringify!($long), '_', '-'))
            .doc(s)
    }};
    ( $long:ident $env:ident $( $doc:literal )* ) => {{
        let mut s = ::std::string::String::new();
        $(
            s.push_str($doc);
            s.push('\n');
        )*
        s.pop();
        $crate::tag::long(
            $crate::__replace!(::std::stringify!($long), '_', '-')
        )
            .env(::std::stringify!($env))
            .doc(s)
    }};
    ( $short:literal $long:ident $env:ident $( $doc:literal )* ) => {{
        let mut s = ::std::string::String::new();
        $(
            s.push_str($doc);
            s.push('\n');
        )*
        s.pop();
        $crate::tag::both($short, $crate::__replace!(::std::stringify!($long), '_', '-'))
            .env(::std::stringify!($env))
            .doc(s)
    }};
}

/// A macro to quickly define your CLI interface with struct-like syntax.
///
/// The syntax looks like this:
///
/// ```plain
/// sarge! {
///     StructName,
///     [fields...]
/// }
/// ```
///
/// Each field has the following form:
/// ```plain
///     [> DOCS...]
///     [#MARKER] [SHORT_FORM] [@ENV_FORM] long_form: type [= DEFAULT],
/// ```
///
/// # Documentation
///
/// You may specify documentation to apply to arguments. On feature `help`,
/// this will also be specified in `print_help`. Example:
///
/// ```plain
///     > "Documentation for argument"
///     (rest of argument)
/// ```
///
/// Whether or not you add documentation, the basic form of the argument will
/// still be provided in the help message.
///
/// # Wrapper markers
///
/// You can specify the type of each argument by prepending a "wrapper" marker,
/// like so:
///
/// ```plain
///     #ok name: type,
/// ```
///
/// There are three kinds of wrappers: `#err`, `#ok`, and none at all. `#err`
/// wraps the type in `Option<Result<T, _>>`: if the argument wasn't passed at
/// all, it's `None`; if it failed to parse, it's `Some(Err(_))`.
///
/// `#ok` wraps it in `Option<T>`: if the argument wasn't passed, or failed to
/// parse, it's `None`.
///
/// No wrapper means that if the argument wasn't passed, or failed to parse,
/// trying to parse your arguments will panic. It gives basic error messages,
/// but this should still be avoided if possible. It is, however, safe to use
/// this marker on `bool` arguments, since they will default to `false`.
///
/// # Short forms
///
/// To specify a short form for your argument, place a character literal after
/// your wrapper marker (if any), like so:
///
/// ```plain
///     #ok 'a' name: type,
/// ```
///
/// Note that you cannot specify an argument that has a short form but no long
/// form. This is for simplicity's sake, and is rarely needed anyways. The
/// long form of your argument is derived from the fields name, with any
/// underscores replaced by dashes at compile-time.
///
/// # Environment variables
///
/// To specify an environment variable form, place the name preceded by an `@`
/// symbol after your short form (if any), like so:
///
/// ```plain
///     #ok 'a' @ENV_FORM name: type,
/// ```
///
/// The name will not be altered in any way, so make sure it's unique and won't
/// clash with any other common environment variables.
///
/// # Defaults
///
/// Sometimes, when an argument is left unspecified, you wish to automatically
/// fill it with a default. This may be done with the following syntax:
///
/// ```plain
///     page: u32 = 1,
/// ```
///
/// This example shows an infallible default, i.e. even if the argument is
/// passed but fails to parse, it will be defaulted. You may instead desire to
/// place the default on an `#err` argument, in which case it will become
/// `Result<T, _>`.
///
/// You may not place defaults on `#ok` arguments.
///
/// # Example
///
/// ```
/// # use sarge::prelude::*;
/// sarge! {
///     // This is the name of our struct.
///     > "This is documentation for our command."
///     Args,
///
///     // These are our arguments. Each will have a long variant matching the
///     // field name one-to-one, with one exception: all underscores are
///     // replaced by dashes at compile-time.
///     //
///     // The hashtags denote the arg 'wrapper'. No wrapper means it will be
///     // unwrapped; if the argument wasn't passed, or it failed to parse, this
///     // will panic. Thankfully, `bool` arguments are immune to both, and
///     // `String` arguments are immune to the latter.
///
///     > "Hello, World!"
///     first: bool, // true if `--first` is passed, false otherwise
///
///     // If you want a short variant (e.g. '-s'), you can specify one with a char
///     // literal before the name (but after the wrapper, if any):
///     's' second: String,
///
///     // You can also specify an environment variable counterpart. If an argument
///     // has values for both an environment variable and a CLI argument, the CLI
///     // argument takes precedence.
///     @ENV_VAR env_var: i32,
///
///     // `#err` makes the argument an `Option<Result<T, _>>`.
///     #err foo: f32,
///
///     // `#ok` makes the argument an `Option<T>`, discarding any parsing errors.
///     #ok bar: f64,
///
///     // Here's almost every feature in one argument:
///     // an `Option<Result<T, _>>` that can be set via `-b`, `--baz`, or `BAZ=`.
///     #err 'b' @BAZ baz: Vec<u64>,
///
///     // An argument with a default value:
///     qux: String = "foobar".into(),
/// }
///
/// fn main() {
///     let args = [
///         "test",           // Usually the name of the executable.
///         "--first",
///         "-s", "Hello, World!",
///         "--bar=badnum",   // The syntax `--arg=val` is valid for long tags.
///         "foobar",         // This value isn't part of an argument.
///         "--baz", "1,2,3", // Remember this value...
///     ];
///
///     let env = [
///         ("ENV_VAR", "42"),
///         ("BAZ", "4,5,6"), // ...and this one.
///     ];
///
///     // Normally, you would use `::parse()` here. However, since this gets run
///     // as a test, we'll manually pass the arguments along.
///     let (args, remainder) = Args::parse_provided(args, env)
///         .expect("Failed to parse arguments");
///
///     assert_eq!(remainder, vec!["test", "foobar"]);
///
///     assert!(args.first);
///     assert_eq!(args.second, "Hello, World!");
///     assert_eq!(args.env_var, 42);
///     assert_eq!(args.foo, None);
///     assert_eq!(args.bar, None);
///     assert_eq!(args.baz, Some(Ok(vec![1, 2, 3])));
///     assert_eq!(args.qux, "foobar");
/// }
/// ```
#[macro_export]
macro_rules! sarge {
    (
        $( > $doc:literal )*
        $v:vis $name:ident, $(
            $( > $adoc:literal )*
            $( # $spec:ident )?
            $( $short:literal )?
            $( @ $env:ident )?
            $av:vis
            $long:ident : $typ:ty
            $( = $default:expr )?
        ),* $(,)?
    ) => {
        $v struct $name {
            $(
                $(#[doc = $adoc])*
                $av $long: $crate::__arg_typ!($long, $( $spec, )? $typ, $( $default )?),
            )*
        }

        impl $name {
            /// Prints help for all the arguments.
            ///
            /// Only available on feature `help`.
            #[allow(unused)]
            pub fn print_help() {
                let mut parser = $crate::ArgumentReader::new();
                parser.doc = Some(
                    String::new()
                        $( + "\n" + $doc )*
                );

                $(
                    parser.add::<$typ>(
                        $crate::__var_tag!($( $short )? $long $( $env )? $( $adoc )*)
                    );
                )*

                parser.print_help();
            }

            /// Parse arguments from `std::env::{args,vars}`.
            ///
            /// # Errors
            ///
            /// If any arguments fail to parse their values, this
            /// will forward that error. Otherwise, see
            /// [`ArgParseError`] for a list of all possible errors.
            #[allow(unused)]
            pub fn parse() -> std::result::Result<(Self, std::vec::Vec<std::string::String>), $crate::ArgParseError> {

                Self::parse_provided(
                    std::env::args(),
                    std::env::vars(),
                )
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
            #[allow(unused)]
            pub fn parse_env<
                K: std::convert::AsRef<str>,
                V: std::convert::AsRef<str>,
                I: std::iter::IntoIterator<Item = (K, V)>,
            >(env: I) -> std::result::Result<Self, $crate::ArgParseError> {
                Ok(Self::parse_provided(
                    std::option::Option::<&'static str>::None,
                    env,
                )?.0)
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
            #[allow(unused)]
            pub fn parse_cli<
                A: std::convert::AsRef<str>,
                I: std::iter::IntoIterator<Item = A>,
            >(args: I) -> std::result::Result<(Self, std::vec::Vec<std::string::String>), $crate::ArgParseError> {
                Self::parse_provided(
                    args,
                    std::option::Option::<(&'static str, &'static str)>::None,
                )
            }

            /// Parse from the provided environment variables and CLI arguments.
            ///
            /// # Errors
            ///
            /// See [`parse`] for details.
            #[allow(unused)]
            pub fn parse_provided<
                A: std::convert::AsRef<str>,
                IA: std::iter::IntoIterator<Item = A>,
                K: std::convert::AsRef<str>,
                V: std::convert::AsRef<str>,
                IE: std::iter::IntoIterator<Item = (K, V)>,
            >(
                cli: IA,
                env: IE,
            ) -> std::result::Result<
                    (Self, std::vec::Vec<std::string::String>), $crate::ArgParseError
                >
            {
                let mut parser = $crate::ArgumentReader::new();

                $(
                    let $long = parser.add::<$typ>(
                        $crate::__var_tag!($( $short )? $long $( $env )? )
                    );
                )*

                let args = parser.parse_provided(cli, env)?;

                $(
                    $crate::__parse_arg!($( $spec )? => args, $long, $( $default )?);
                )*

                let me = Self {$(
                    $long,
                )*};

                Ok((me, args.into()))
            }
        }
    };
}
