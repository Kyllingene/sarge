use crate::prelude::*;

mod anyhow {
    pub use ::std::result::Result::Ok;
    #[allow(dead_code)]
    pub struct Error;
}

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

mod polluted_ok_import {
    use super::anyhow;
    use crate::prelude::*;
    use anyhow::Ok;

    sarge! {
        PollutedArgs,
        polluted_flag: bool,
    }

    #[test]
    fn ok_import_does_not_break_macro() {
        let (args, remainder) =
            PollutedArgs::parse_cli(["polluted"]).expect("sarge should ignore anyhow::Ok import");

        let _ = Ok::<(), anyhow::Error>(());

        assert_eq!(remainder, vec!["polluted"]);
        assert!(!args.polluted_flag);
    }
}

sarge! {
    #[derive(Debug, PartialEq, Eq)]
    DefaultArgs,

    // Default value (String).
    socket_addr: String = "127.0.0.1:9912",

    // `#ok` default is a plain value; macro wraps it in `Some(...)`.
    #ok 't' target_addr: String = "127.0.0.1:9911",

    // `#ok + default` applies only to missing values; parse failures become `None`.
    #ok 'n' num: u32 = 42,

    // `#err` default is a plain value (not `Some(Ok(...))`).
    #err 'h' help: bool = true,

    // `Vec<String>` defaults can be specified without `.into()` per element.
    #ok 'd' data: Vec<String> = vec![r#"{"name":"hello"}"#],
}

#[test]
fn defaults_are_applied() {
    let (args, remainder) = DefaultArgs::parse_cli(["bin"]).expect("failed to parse default args");

    assert_eq!(remainder, vec!["bin"]);
    assert_eq!(args.socket_addr, "127.0.0.1:9912");
    assert_eq!(args.target_addr.as_deref(), Some("127.0.0.1:9911"));
    assert_eq!(args.num, Some(42));
    assert_eq!(args.help, Ok(true));
    assert_eq!(args.data, Some(vec![r#"{"name":"hello"}"#.to_string()]));
}

#[test]
fn ok_default_is_none_on_parse_error() {
    let (args, _) = DefaultArgs::parse_cli(["bin", "--num", "not-a-number"])
        .expect("parse_cli should succeed; #ok turns parse failures into None");

    assert_eq!(args.num, None);
}

#[test]
fn ok_default_never_none_when_missing() {
    let (args, _) = DefaultArgs::parse_cli(["bin"]).expect("failed to parse default args");

    assert!(args.target_addr.is_some());
    assert!(args.num.is_some());
}

#[test]
fn ok_default_does_not_default_on_parse_error() {
    // `#ok + default` applies only to missing values; parse failures become `None`.
    let (args, _) = DefaultArgs::parse_cli(["bin", "--num", "bad"])
        .expect("parse_cli should succeed; #ok turns parse failures into None");

    assert_eq!(args.num, None);
}
