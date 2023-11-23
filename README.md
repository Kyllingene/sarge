# Sarge

![build status](https://github.com/kyllingene/sarge/actions/workflows/rust.yml/badge.svg)
![license](https://img.shields.io/crates/l/sarge)
![version](https://img.shields.io/crates/v/sarge)

Sarge is a small, opinionated arguments parser. It uses clever techniques to
make parsing arguments as quick and painless as possible. It has zero
dependencies, reducing cruft and therefore build times. Here are some
differences with the industry standard, [clap](https://crates.io/crates/clap):

- No dependencies
- No proc macros
    - Provides a very powerful *regular* macro through the feature `macros`
- Supports environment variables
- Provides a cleaner builder interface
- Isn't a jack-of-all-trades
    - Doesn't support weird syntaxes
    - All macro-made arguments have to have a long form
    - Focuses on sensible defaults to minimize effort for everyone involved
    - Doesn't provide help messages, completions, etc.
    - Doesn't support nested arguments
- Isn't run by committee
    - Not out of disdain, but there's only one maintainer, so...
- Isn't a giant project
    - Those can be great, but can also be overkill
- Has first-class support for custom argument types

One or more of the above might be a deal-breaker. That's okay! I made sarge so
that there was a good, light alternative to clap. Use whichever one suits
your use-case. I personally use sarge for all my projects, because they're all
small; this forces me to be active in maintaining it.

## Features

- First-class "builder" pattern, but better
    - Used to be the only option, so it's been fleshed out
- Non-proc macro for building a CLI interface
- Zero dependencies (yes, this is my favorite feature)
- Custom argument kinds
    - Simply impl a trait and it works like a builtin
- Thread safety when using the builder 
- The following builtin argument types:
    - `bool`
    - `i8/i16/i32/i64`
    - `u8/u16/u32/u64`
    - `f32/f64`
    - `String`
    - `Vec<T>` where `T: ArgumentType`

## Grocery list

- Better unit testing
    - There are tests for everything, but they aren't top-priority yet
- More maintainers
- Better code styling
    - Probably remove `clippy::pedantic` and get more fine-grained
- Better, fuller docs
    - They're usable, but (like tests) aren't top-priority

## Contributing

These mostly stem from two things: a single maintainer, and a lack of interest.
If you use sarge, ***please*** star it on GitHub, or leave issues! It tells me
that others are interested in the project, and pushes me to be more rigorous
and develop it more.

As for the single maintainer, I am happy to accept pull requests. Just make
sure it passes `cargo fmt`, `cargo clippy` and `cargo test`.

## Examples

Here's a giant example using all the bells and whistles; note that if you
disable the `macros` feature, this won't compile:

<details>

```rust
use sarge::prelude::*;

// This is a normal, non-proc macro. That means sarge is still
// zero-dependency! The syntax may seem a little strange at first, but it
// should help greatly when defining your CLI interface.
sarge! {
    // This is the name of our struct.
    Args,

    // These are our arguments. Each will have a long variant matching the
    // field name one-to-one, with one exception: all underscores are
    // replaced by dashes at compile-time.
    //
    // The hashtags denote the arg 'wrapper'. No wrapper means it will be
    // unwrapped; if the argument wasn't passed, or it failed to parse, this
    // will panic. Thankfully, `bool` arguments are immune to both, and
    // `String` arguments are immune to the latter.

    first: bool, // true if `--first` is passed, false otherwise

    // If you want a short variant (e.g. '-s'), you can specify one with a char
    // literal before the name (but after the wrapper, if any):
    's' second: String,

    // You can also specify an environment variable counterpart. If an argument
    // has values for both an environment variable and a CLI argument, the CLI
    // argument takes precedence.
    @ENV_VAR env_var: i32,

    // `#err` makes the argument an `Option<Result<T, _>>`.
    #err foo: f32,

    // `#ok` makes the argument an `Option<T>`, discarding any parsing errors.
    #ok bar: f64,

    // Here's every feature in one argument:
    // an `Option<Result<T, _>>` that can be set via `-b`, `--baz`, or `BAZ=`.
    #err 'b' @BAZ baz: Vec<u64>,
}

// Some utility macros to make this example less verbose.

macro_rules! create_args {
    ( $( $arg:expr ),* $(,)? ) => {
        [ $( $arg.to_string(), )* ]
    };
}

macro_rules! create_env {
    ( $( $name:expr, $val:expr ),* $(,)? ) => {
        [ $( ($name.to_string(), $val.to_string()), )* ]
    };
}

fn main() {
    let args = create_args![
        "test",           // The name of the executable.
        "--first",
        "-s", "Hello, World!",
        "--bar=badnum",   // The syntax `--arg=val` is valid for long tags.
        "foobar",         // This value isn't part of an argument.
        "--baz", "1,2,3", // Remember this value...
    ];

    let env = create_env![
        "ENV_VAR", "42",
        "BAZ", "4,5,6",   // ...and this one.
    ];

    // Normally, you would use `::parse()` here. However, since this gets run
    // as a test, we'll manually pass the arguments along.
    let (args, remainder) = Args::parse_provided(&args, env.into_iter())
        .expect("Failed to parse arguments");

    assert_eq!(remainder, vec!["foobar"]);

    assert!(args.first);
    assert_eq!(args.second, "Hello, World!");
    assert_eq!(args.env_var, 42);
    assert_eq!(args.foo, None);
    assert_eq!(args.bar, None);
    assert_eq!(args.baz, Some(Ok(vec![1, 2, 3])));
}
```

</details>

## Environment Variables

Sarge also supports using environment variables as arguments. This is automatically
done when you call `parse`, or you can use `parse_env` to pass the variables yourself.
It takes an `Iterator<Item = (String, String)>` as a reciever, the same type
`std::env::args()` returns.

Here's a quick example:

<details>

```rust
use sarge::prelude::*;

fn main() {
    let parser = ArgumentParser::new();

    // This can only be specified via environment variable.
    let just_env = parser.add(tag::env("JUST_ENV"));

    // This can be specified as either an environment variable,
    // or a regular CLI argument. If both are given, the CLI
    // argument takes precedence.
    let both = parser.add(tag::long("cli-form").env("ENV_FORM"));

    // Here are the CLI arguments...
    let cli_args = [
        "test".to_string(),
        "--cli-form=123".to_string(),
    ];

    // ...and the "environment" variables.
    let env_args = [
        // Boolean arguments treat `0`, `false`, and no argument as false,
        // while everything else is true.
        ("JUST_ENV".to_string(), "0".to_string()),
        ("ENV_FORM".to_string(), "456".to_string()),
    ].into_iter();

    // `parser.parse()` would automatically use `std::env::vars`.
    parser.parse_provided(&cli_args, env_args).unwrap();

    assert_eq!(just_env.get(), Some(Ok(false)));

    // Since the CLI argument was given, it uses that instead.
    assert_eq!(both.get(), Some(Ok(123i64)));
}
```

</details>

## Custom Types

Using the `ArgumentType` trait, you can implement your own types. Here's an
example (taken from `src/test/custom_type.rs`):

<details>

```rust
use std::convert::Infallible;
use sarge::{prelude::*, ArgumentType, ArgResult};

#[derive(Debug, PartialEq, Eq)]
struct MyCustomType(Vec<String>);

impl ArgumentType for MyCustomType {
    /// This gets returned from `ArgumentRef::get` in the event
    /// of a failed parse.
    type Error = Infallible;

    /// Do your parsing here. This just splits on spaces.
    /// If the argument was passed without a value, `val == None`.
    fn from_value(val: Option<&str>) -> ArgResult<Self> {
        Some(Ok(Self(
            val?.split(' ')
                .map(|s| s.to_string())
                .collect()
        )))
    }
}

fn main() {
    let parser = ArgumentParser::new();
    let my_argument = parser.add(tag::long("myarg"));

    let arguments = [
        "custom_type_test".to_string(),
        "--myarg".to_string(),
        "Hello World !".to_string(),
    ];

    let _ = parser.parse_cli(&arguments, false).expect("failed to parse arguments");

    assert_eq!(
        my_argument.get(),
        Some(Ok(MyCustomType(
            vec![
                "Hello".to_string(),
                "World".to_string(),
                "!".to_string(),
            ]
        )))
    );
}
```

</details>
