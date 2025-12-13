use crate::prelude::*;

mod anyhow {
    pub use ::std::result::Result::Ok;
    pub struct Error;
}
use anyhow::Ok as AnyhowOk;

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

    // You can give values an infallible default:
    fifth: u32 = 1,

    // ...or keep any errors:
    #err sixth: u8 = 0,
}

sarge! {
    > "Derived test args"
    #[derive(Debug, PartialEq, Eq)]
    DerivedArgs,

    > "Derived test flag"
    derived_flag: bool,
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
        "--sixth",
        "256",
    ])
    .expect("failed to parse arguments");

    assert!(args.first_arg);
    assert_eq!(args.second, None);
    assert_eq!(args.third, Some(Ok(vec![123, 456, 789])));
    assert_eq!(args.fourth, 10.11);
    assert_eq!(args.fifth, 1);
    assert!(args.sixth.is_err());
}

#[test]
fn struct_attributes_are_applied() {
    let (args, remainder) = DerivedArgs::parse_cli(["bin"]).expect("failed to parse derived args");

    assert_eq!(remainder, vec!["bin"]);
    assert_eq!(
        args,
        DerivedArgs {
            derived_flag: false
        }
    );
    let rendered = format!("{args:?}");
    assert!(rendered.contains("derived_flag"));
}

#[test]
fn ok_name_pollution_is_ignored() {
    let (args, remainder) = DerivedArgs::parse_cli(["polluted"])
        .expect("sarge should ignore anyhow::Ok import");

    let _ = AnyhowOk::<(), anyhow::Error>(());

    assert_eq!(remainder, vec!["polluted"]);
    assert!(!args.derived_flag);
}
