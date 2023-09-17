# sarge

![build status](https://github.com/kyllingene/sarge/actions/workflows/rust.yml/badge.svg)
![license](https://img.shields.io/crates/l/sarge)
![version](https://img.shields.io/crates/v/sarge)

Sarge is a simple, lightweight argument parser. It has two styles of argument: short (e.g. `-h`) and long (e.g. `--help`) (and both), and six different argument types: `i64`, `u64`, `f64`, `String`, `bool`, and `Vec<T> where T: ArgumentType`.

Arguments are registered with an `ArgumentParser`, and when you're ready, `ArgumentParser::parse`. Parsing does two things: it sets the value of each argument, and returns a `Vec<String>` of the values not associated with an argument. Arguments can be created easily via the `tag` function and registered with `ArgumentParser::add`, returning an `ArgumentRef`.

Arguments can be retrieved with `ArgumentRef::get(self)`.

Example:
```rust
use sarge::prelude::*;

# fn main() {
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
    let remainder = parser.parse_args(&arguments).expect("Failed to parse arguments");

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
# }
```

## Custom Types

Using the `ArgumentType` trait, you can implement your own types. Here's an
example (taken from `src/test/custom_type.rs`):

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
}

fn main() {
    let parser = ArgumentParser::new();
    let my_argument = parser.add::<MyCustomType>(tag::long("myarg"));

    let arguments = [
        "custom_type_test".to_string(),
        "--myarg".to_string(),
        "Hello World !".to_string(),
    ];

    let _ = parser.parse_args(&arguments).expect("failed to parse arguments");

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
