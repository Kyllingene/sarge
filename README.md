# sarge

![build status](https://github.com/kyllingene/sarge/actions/workflows/rust.yml/badge.svg)
![license](https://img.shields.io/crates/l/sarge)
![version](https://img.shields.io/crates/v/sarge)

Sarge is a simple, lightweight argument parser. It has two styles of argument: short (e.g. `-h`) and long (e.g. `--help`) (and both), and six different argument types: `i64`, `u64`, `f64`, `String`, `bool`, and `Vec<T> where T: ArgumentType`. It also supports using environment variables as arguments; in the event of a conflict, command-line.

Arguments are registered with an `ArgumentParser`, and when you're ready, `ArgumentParser::parse`. Parsing does two things: it sets the value of each argument, and returns a `Vec<String>` of the values not associated with an argument. Arguments can be created easily via the `tag::` functions and registered with `ArgumentParser::add`, returning an `ArgumentRef`.

Arguments can be retrieved with `ArgumentRef::get(self)`. Boolean arguments will always be `Ok(true | false)`; other arguments may be `Err(_)` if they failed to parse, or were not provided.

Example:

<details>

```rust
use sarge::prelude::*;

fn main() {
    let parser = ArgumentParser::new();

    // These are borrows on your parser. Don't worry, `ArgumentParser`
    // uses thread-safe interior mutability to make this ergonomic.
    let help = parser.add(tag::both('h', "help")); // This matches either `-h` or `--help`.
    let number = parser.add::<i64>(tag::long("number"));  // This matches only `--number`.

    // These are the fake arguments for our program. Note that any
    // of these could include spaces, but shells generally break
    // arguments on non-quoted whitespace.
    let arguments = vec![
        "my_program".to_string(),
        "abc".to_string(),
        "--number".to_string(),
        "123".to_string(),
        "def".to_string(),
    ];

    // In your application, you would probably use `ArgumentParser::parse()`.
    let remainder = parser.parse_cli(&arguments, false).expect("Failed to parse arguments");

    assert_eq!(
        help.get(), // Consumes `help`; use `get_keep` to retain the reference.
        Ok(false)   // Since we compare it to a `bool`, Rust knows that `help`
    );              // must also be a `bool`.

    assert_eq!(
        number.get(),
        Ok(123)     // However, since 123 could be either an `i64` *or* a `u64`,
    );              // we had to specify `::<i64>` on `parser.add`.

    assert_eq!(
        remainder,  // Remainder is all arguments not paired with a tag, in order.
        vec![
            "abc".to_string(),
            "def".to_string(),
        ]
    );

    assert_eq!(
        parser.binary(), // The first argument, if any.
        Some("my_program".to_string())
    );
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

    assert_eq!(just_env.get(), Ok(false));

    // Since the CLI argument was given, it uses that instead.
    assert_eq!(both.get(), Ok(123i64));
}
```

</details>

## Custom Types

Using the `ArgumentType` trait, you can implement your own types. Here's an
example (taken from `src/test/custom_type.rs`):

<details>

```rust
use sarge::{prelude::*, custom::*};

#[derive(Debug, PartialEq, Eq)]
struct MyCustomType(Vec<String>);

impl ArgumentType for MyCustomType {
    /// This gets returned from `ArgumentRef::get` in the event
    /// of a failed parse. This must be `Debug`.
    type Error = ();

    /// What type of input. For custom types, you want
    /// an `ArgumentValue::String`.
    fn arg_type() -> ArgumentValueType {
        ArgumentValueType::String
    }

    /// Do your parsing here. This just splits on spaces.
    fn from_value(val: ArgumentValue) -> Result<Self, Self::Error> {
        if let ArgumentValue::String(val) = val {
            Ok(Self(val.split(' ').map(|s| s.to_string()).collect()))
        } else {
            Err(())
        }
    }

    /// In the event the argument wasn't given a value, this
    /// will be called to determine if there should be a default
    /// value. If you omit this, it defaults to `None`.
    fn default_value() -> Option<Self> {
        // Here, we return an empty vector.
        Some(MyCustomType(Vec::new()))
    }
}

fn main() {
    let parser = ArgumentParser::new();
    let my_argument = parser.add::<MyCustomType>(tag::long("myarg"));

    let arguments = [
        "custom_type_test".to_string(),
        "--myarg".to_string(),
        "Hello World !".to_string(),
    ];

    let _ = parser.parse_cli(&arguments, false).expect("failed to parse arguments");

    assert_eq!(
        my_argument.get(),
        Ok(
            MyCustomType(
                vec![
                    "Hello".to_string(),
                    "World".to_string(),
                    "!".to_string(),
                ]
            )
        )
    );
}
```

</details>
