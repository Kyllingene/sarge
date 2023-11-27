use crate::prelude::*;

sarge! {
    Args,

    // Underscores are automatically converted to dashes at compile-time
    first_arg: bool,

    // `ok` treats parsing errors as if the argument wasn't passed at all
    #ok second: String,

    // `err` returns an `Option<Result<T, _>>`, with any parsing errors
    #err 't' third: Vec<i64>,

    // A character before the name (but after the kind) is the short version
    // of the argument, i.e. `-f`
    'f' fourth: f64,
}

#[test]
fn test_macros() {
    let (args, _) = Args::parse_cli([
        "test",
        "--first-arg",
        "--third",
        "123,456,789",
        "-f",
        "10.11",
    ])
    .expect("failed to parse arguments");

    assert!(args.first_arg);
    assert_eq!(args.second, None);
    assert_eq!(args.third, Some(Ok(vec![123, 456, 789])));
    assert_eq!(args.fourth, 10.11);
}
