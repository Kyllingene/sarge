# sarge

## std-only command-line arguments parser

Sarge is a simple, lightweight argument parser. It has two styles of argument: short (e.g. `-h`) and long (e.g. `--help`), and three types: flag (`bool`, present equals true), int (`i32`, e.g. `-n 123`, `--number 123` or `--number=123`) or string (`String`, e.g. `-s foo`, `--str "foo bar"` or `--str=bar`).

Arguments are registered with an `ArgumentParser`, and when you're ready, `ArgumentParser::parse`. Parsing does two things: it sets the value of each argument, and returns a `Vec<String>` of the values not associated with an argument. Arguments can be created easily via the `arg!` macro.

Arguments can be retrieved with `ArgumentParser::arg(Tag)`. `Tag`s are the post-dash part of the argument, i.e. `help` in `--help`. They can be created easily via the `tag!` macro.

Example:
```rust
use sarge::{ArgumentParser, arg, get_flag, get_int};

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help")); // after parser.parse(), this will be either ArgValue::Flag(true) or ArgValue::Flag(false)
    parser.add(arg!(int, long, "number"));     // after parser.parse(), this will be either None or
                                               // Some(ArgValue::Int(_))

    let supplied_args = vec![
        "my_program".to_string(),
        "abc".to_string(),
        "--number".to_string(),
        "123".to_string(),
        "def".to_string(),
    ];

    // parser.parse() is a wrapper for parser.parse_args(env::args().collect::<Vec<_>>())
    let remainder = parser.parse_args(supplied_args).expect("Failed to parse arguments");

    assert_eq!(
        get_flag!(parser, both, 'h', "help"), // panics if -h/--help isn't a flag
        false
    );

    assert_eq!(
        get_int!(parser, long, "number"), // panics if --number isn't an int or wasn't supplied
        123
    );

    assert_eq!(
        remainder,
        vec![
            "abc".to_string(),
            "def".to_string(),
        ]
    );

    assert_eq!(
        parser.binary, // first argument, if any
        Some("my_program".to_string())
    );
}
```
