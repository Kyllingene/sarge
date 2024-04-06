# Sarge

![build status](https://github.com/kyllingene/sarge/actions/workflows/rust.yml/badge.svg)
![license](https://img.shields.io/crates/l/sarge)
![version](https://img.shields.io/crates/v/sarge)

Sarge is a small, opinionated arguments parser. It uses clever techniques to
make parsing arguments as quick and painless as possible. It has zero
dependencies, reducing cruft and therefore build times. Here are some
differences with the industry standard, [clap](https://crates.io/crates/clap):

- No dependencies
    - Leads to small size: `256KiB` compared to clap's `5.4MiB`\*
      (shallow clone of git repository | `du -h`)
    - Leads to fast builds: `0.46s` to clap's `19.97s`, clean build\*
      (times on my old laptop over cellular)
- No proc macros
    - Provides a powerful *regular* macro through the default feature `macros`
- Provides a cleaner builder-like interface
- Isn't a jack-of-all-trades
    - Doesn't support weird syntaxes
    - All struct-style arguments have to have a long form
    - Focuses on sensible defaults to minimize effort for everyone involved
    - Doesn't provide help messages, completions, etc.
    - Doesn't support nested arguments
- Isn't run by committee
    - Not out of disdain, but there's only one maintainer, so...
- Isn't a giant project
    - Those can be great, but can also be overkill
- Has first-class support for custom argument types

\**Disclaimer:* these numbers might not be perfectly up-to-date, but there
shouldn't be any major changes on sarge's side.

One or more of the above might be a deal-breaker. That's okay! I made sarge so
that there was a good, light alternative to clap. Use whichever one suits
your use-case. I personally use sarge for all my projects, because they're all
small; this forces me to be active in maintaining it.

## Features

- Zero dependencies (yes, this is my favorite feature)
- First-class "builder" pattern, but better
    - Used to be the only option, so it's been fleshed out
- Non-proc macro for building a CLI interface
    - Supports default values
- Supports environment variables
- Custom argument kinds
    - Simply impl a trait and it works like a builtin
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
    - I want them to be top-notch

## Contributing

The above mostly stem from two things: a single maintainer, and a lack of
interest.  If you use sarge, ***please*** star it on GitHub, or even better,
leave issues!  It tells me that others are interested in the project, and
pushes me to be more rigorous and develop it more.

As for the single maintainer, I am happy to accept pull requests. Just make
sure it passes `cargo fmt`, `cargo clippy` and `cargo test`. Some features may
be out of scope for sarge; the goal isn't infinite customizability, so if a
feature significantly complicates anything, it might not be accepted.

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

    // Here's every feature in one argument: a `Result<T, _>` that can be set
    // via `-b`, `--baz`, or `BAZ=`, and defaults to [1, 2, 3] if not passed.
    #err 'b' @BAZ baz: Vec<u64> = vec![1, 2, 3],
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
        "test",           // Usually the name of the executable.
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

    assert_eq!(remainder, vec!["test", "foobar"]);

    assert!(args.first);
    assert_eq!(args.second, "Hello, World!");
    assert_eq!(args.env_var, 42);
    assert_eq!(args.foo, None);
    assert_eq!(args.bar, None);
    assert_eq!(args.baz, Ok(vec![1, 2, 3]));
}
```

</details>

## Environment Variables

Sarge also supports using environment variables as arguments. This is automatically
done when you call `parse`, or you can use `parse_env` to pass the variables yourself.

Here's a quick example:

<details>

```rust
use sarge::prelude::*;

fn main() {
    let mut parser = ArgumentReader::new();

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
    let args = parser.parse_provided(&cli_args, env_args).unwrap();

    // `args` has the type `Arguments`, which contains two things:
    // - The CLI arguments that weren't part of a tagged argument
    // - The tagged arguments and their values
    //
    // To get a value from an `ArgumentRef`, use `.get(&Arguments)`:

    assert_eq!(just_env.get(&args), Some(Ok(false)));

    // Since the CLI argument was given, it uses that instead.
    assert_eq!(both.get(&args), Some(Ok(123i64)));
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

sarge! {
    Args,

    #err my_argument: MyCustomType,
}

fn main() {
    let arguments = [
        "custom_type_test".to_string(),
        "--my-argument".to_string(),
        "Hello World !".to_string(),
    ];

    let (args, _) = Args::parse_provided(&arguments, None::<(&str, &str)>).expect("failed to parse arguments");

    assert_eq!(
        args.my_argument,
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
