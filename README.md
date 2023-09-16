# sarge

## std-only command-line arguments parser

Sarge is a simple, lightweight argument parser. It has two styles of argument: short (e.g. `-h`) and long (e.g. `--help`) (and both), and five different argument types: `i64`, `u64`, `f64`, `String`, and `bool`.

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
    let supplied_args = vec![
        "my_program".to_string(),
        "abc".to_string(),
        "--number".to_string(),
        "123".to_string(),
        "def".to_string(),
    ];

    // In your application, you would probably use `ArgumentParser::parse()`.
    let remainder = parser.parse_args(&supplied_args).expect("Failed to parse arguments");

    assert_eq!(
        help.get(), // Consumes `help`; use `get_keep` to retain the reference.
        Some(false) // Since we compare it to a `bool`, Rust knows that `help`
                    // must also be a `bool`.
    );

    assert_eq!(
        number.get(),
        Some(123)   // However, since 123 could be either an `i64` *or* a `u64`,
                    // we had to specify `::<i64>` on `parser.add`.
    );

    assert_eq!(
        remainder,
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
