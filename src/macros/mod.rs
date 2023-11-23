//! The [`sarge!`] macro and all it's helper utilities.

#[doc(hidden)]
pub mod const_exprs;

#[macro_export]
#[doc(hidden)]
macro_rules! __parse_arg {
    ( err => $parser:expr, $name:ident ) => {
        let $name = $name.get();
    };

    ( ok => $parser:expr, $name:ident ) => {
        let $name = $name.get().map(|a| a.ok()).flatten();
    };

    ( => $parser:expr, $name:ident ) => {
        let $name = $name
            .get()
            .expect("Tried to unwrap argument that wasn't passed")
            .expect("Tried to unwrap argument that failed to parse");
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __arg_typ {
    ( err , $typ:ty ) => {
        $crate::ArgResult<$typ>
    };

    ( ok , $typ:ty ) => {
        std::option::Option<$typ>
    };

    ( $typ:ty ) => {
        $typ
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __var_tag {
    ( $long:ident ) => {
        $crate::tag::long($crate::__replace!(stringify!($long), '_', '-'))
    };
    ( $short:literal $long:ident ) => {
        $crate::tag::both($short, $crate::__replace!(stringify!($long), '_', '-'))
    };
    ( $long:ident $env:ident ) => {
        $crate::tag::long($crate::__replace!(stringify!($long), '_', '-')).env(stringify!($env))
    };
    ( $short:literal $long:ident $env:ident ) => {
        $crate::tag::both($short, $crate::__replace!(stringify!($long), '_', '-'))
            .env(stringify!($env))
    };
}

#[macro_export]
/// A macro to quickly define your CLI interface with struct-like syntax.
///
/// # Example
///
/// ```
/// # use sarge::prelude::*;
/// // This is a normal, non-proc macro. That means sarge is still
/// // zero-dependency! The syntax may seem a little strange at first, but it
/// // should help greatly when defining your CLI interface.
/// sarge! {
///     // This is the name of our struct.
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
///     // Here's every feature in one argument:
///     // an `Option<Result<T, _>>` that can be set via `-b`, `--baz`, or `BAZ=`.
///     #err 'b' @BAZ baz: Vec<u64>,
/// }
/// 
/// # macro_rules! create_args {
/// #     ( $( $arg:expr ),* $(,)? ) => {
/// #         [ $( $arg.to_string(), )* ]
/// #     };
/// # }
/// # 
/// # macro_rules! create_env {
/// #     ( $( $name:expr, $val:expr ),* $(,)? ) => {
/// #         [ $( ($name.to_string(), $val.to_string()), )* ]
/// #     };
/// # }
/// 
/// fn main() {
///     let args = create_args![
///         "test",           // The name of the executable.
///         "--first",
///         "-s", "Hello, World!",
///         "--bar=badnum",   // The syntax `--arg=val` is valid for long tags.
///         "foobar",         // This value isn't part of an argument.
///         "--baz", "1,2,3", // Remember this value...
///     ];
/// 
///     let env = create_env![
///         "ENV_VAR", "42",
///         "BAZ", "4,5,6",   // ...and this one.
///     ];
/// 
///     // Normally, you would use `::parse()` here. However, since this gets run
///     // as a test, we'll manually pass the arguments along.
///     let (args, remainder) = Args::parse_provided(&args, env.into_iter())
///         .expect("Failed to parse arguments");
/// 
///     assert_eq!(remainder, vec!["foobar"]);
/// 
///     assert!(args.first);
///     assert_eq!(args.second, "Hello, World!");
///     assert_eq!(args.env_var, 42);
///     assert_eq!(args.foo, None);
///     assert_eq!(args.bar, None);
///     assert_eq!(args.baz, Some(Ok(vec![1, 2, 3])));
/// }
/// ```
macro_rules! sarge {
    ( $v:vis $name:ident, $( $( # $spec:ident )? $( $short:literal )? $( @ $env:ident )? $av:vis $long:ident : $typ:ty ),* $(,)? ) => {
        $v struct $name {
            $( $av $long: $crate::__arg_typ!($($spec,)? $typ), )*
        }

        impl $name {
            /// Parse arguments from `std::env::{args,vars}`.
            ///
            /// # Errors
            ///
            /// If any arguments fail to parse their values, this
            /// will forward that error. Otherwise, see
            /// [`ArgParseError`] for a list of all possible errors.
            #[allow(unused)]
            pub fn parse() -> std::result::Result<(Self, std::vec::Vec<std::string::String>), ArgParseError> {
                Self::parse_provided(
                    std::env::args().collect::<std::vec::Vec<_>>().as_slice(),
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
            pub fn parse_env<I: std::iter::Iterator<Item = (std::string::String, std::string::String)>>(
                args: I,
            ) -> std::result::Result<Self, $crate::ArgParseError> {
                Ok(Self::parse_provided(
                    &[],
                    args
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
            pub fn parse_cli(args: &[std::string::String]) -> std::result::Result<(Self, std::vec::Vec<std::string::String>), $crate::ArgParseError> {
                Self::parse_provided(
                    args,
                    None.into_iter()
                )
            }

            /// Parse from the provided environment variables and CLI arguments.
            ///
            /// # Errors
            ///
            /// See [`parse`] for details.
            #[allow(unused)]
            pub fn parse_provided<I>(
                cli: &[std::string::String],
                env: I,
            ) -> std::result::Result<
                    (Self, std::vec::Vec<std::string::String>), $crate::ArgParseError
                > where
                    I: std::iter::Iterator<Item = (std::string::String, std::string::String)>
            {
                let parser = $crate::ArgumentParser::new();

                $(
                    let $long = parser.add::<$typ>(
                        $crate::__var_tag!($( $short )? $long $( $env )?)
                    );
                )*

                let remainder = parser.parse_provided(cli, env)?;

                $(
                    $crate::__parse_arg!($($spec)? => parser, $long);
                )*

                let me = Self {$(
                    $long,
                )*};

                Ok((me, remainder))
            }
        }
    };
}
