use crate::prelude::*;
use crate::{ArgResult, ArgumentType};
use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

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

    // You can give values a default:
    fifth: u32 = 1,

    // ...or keep any errors:
    #err sixth: u8 = 0,
}

sarge! {
    /// Derived test args
    #[derive(Debug, PartialEq, Eq)]
    DerivedArgs,

    /// Derived test flag
    derived_flag: bool,
}

#[cfg(feature = "help")]
sarge! {
    /// DocComment test args
    #[derive(Debug, PartialEq, Eq)]
    DocCommentArgs,

    /// DocComment test flag
    doc_flag: bool,
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

#[cfg(feature = "help")]
#[test]
fn doc_comments_are_used_for_help() {
    let s = DocCommentArgs::help();
    assert!(s.contains("DocComment test args"));
    assert!(s.contains("DocComment test flag"));
}

#[cfg(feature = "help")]
#[test]
fn derived_args_docs_are_used_for_help() {
    let s = DerivedArgs::help();
    assert!(s.contains("Derived test args"));
    assert!(s.contains("Derived test flag"));
}

#[cfg(feature = "help")]
#[test]
fn macro_help_includes_default_values() {
    let s = DefaultArgs::help();
    assert!(s.contains("default: 127.0.0.1:9912"));
    assert!(s.contains("default: 42"));
    assert!(s.contains("default: true"));

    let s = FromStrDefaultHelpArgs::help();
    assert!(s.contains("default: fromstr-default"));

    let s = HttproxyArgs::help();
    println!("{s}");
    let normalized: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        normalized.contains("(default:127.0.0.1:8081)"),
        "help:\n{s}"
    );
    assert!(
        normalized.contains("(default:127.0.0.1:8082)"),
        "help:\n{s}"
    );
    assert!(normalized.contains("(default:info)"), "help:\n{s}");
    assert!(
        normalized.contains("default:[\"hello\",\"nihao\",\"xxxxx\"]"),
        "help:\n{s}"
    );
    assert!(
        normalized.contains("(default:[1,2,3,4,5,6,7,8])"),
        "help:\n{s}"
    );

    let s = DefaultFnHelpArgs::help();
    let normalized: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(normalized.contains("(default:disabled)"), "help:\n{s}");
    assert!(normalized.contains("(default:\"\")"), "help:\n{s}");
}

#[test]
fn default_fn_help_args_defaults_are_applied() {
    let (args, remainder) =
        DefaultFnHelpArgs::parse_cli(["bin"]).expect("failed to parse DefaultFnHelpArgs");

    assert_eq!(remainder, vec!["bin"]);
    assert_eq!(args.auth, Some(AuthArg::default()));
    assert_eq!(args.empty.as_deref(), Some(""));
}

#[cfg(feature = "help")]
sarge! {
    /// Docs line 1
    /// Docs line 2
    #[allow(dead_code)]
    MixedDocArgs,

    /// Field docs line 1
    /// Field docs line 2
    mixed_flag: bool,
}

#[cfg(feature = "help")]
#[test]
fn mixed_docs_are_used_for_help() {
    let s = MixedDocArgs::help();
    assert!(s.contains("Docs line 1"));
    assert!(s.contains("Docs line 2"));
    assert!(s.contains("Field docs line 1"));
    assert!(s.contains("Field docs line 2"));
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

sarge! {
    #[derive(Debug, PartialEq, Eq)]
    FromStrDefaultHelpArgs,

    from_str_val: String = <String as ::std::str::FromStr>::from_str("fromstr-default").unwrap(),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BindAddr(String);

impl fmt::Display for BindAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for BindAddr {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl ArgumentType for BindAddr {
    type Error = Infallible;

    fn from_value(val: Option<&str>) -> ArgResult<Self> {
        Some(Ok(Self(val?.to_string())))
    }

    fn help_default_value(value: &Self) -> Option<String> {
        Some(value.0.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LogLevel(String);

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl ArgumentType for LogLevel {
    type Error = Infallible;

    fn from_value(val: Option<&str>) -> ArgResult<Self> {
        Some(Ok(Self(val?.to_string())))
    }

    fn help_default_value(value: &Self) -> Option<String> {
        Some(value.0.clone())
    }
}

sarge! {
    #[derive(Debug)]
    HttproxyArgs,

    /// The bind addr for serving.
    #ok 'l' @HTTPOXY_BIND bind: BindAddr = BindAddr::from_str("127.0.0.1:8081").unwrap(),

    /// the dir/file will be served
    #ok 'r' @HTTPOXY_REVERSE reverse: BindAddr = BindAddr::from_str("127.0.0.1:8082").unwrap(),

    /// log level: "" means no log, v - info, vv - debug, vvv - trace
    #ok 'v' @HTTPOXY_LOG_LEVEL log_level: LogLevel = LogLevel("info".into()),

    /// log with color?
    #ok colored: bool = false,

    /// help
    #ok 'h' help: bool = false,

    #ok 'c' vec: Vec<String> = vec!["hello", "nihao", "xxxxx"],

    /// test vec2
    #ok 'c' vec2: Vec<u8> = Vec::from([1, 2, 3, 4, 5, 6, 7, 8]),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AuthArg {
    enabled: bool,
}

impl ArgumentType for AuthArg {
    type Error = Infallible;

    fn from_value(_val: Option<&str>) -> ArgResult<Self> {
        Some(Ok(Self { enabled: true }))
    }

    fn help_default_value(value: &Self) -> Option<String> {
        Some(if value.enabled {
            "enabled".to_string()
        } else {
            "disabled".to_string()
        })
    }
}

sarge! {
    #[derive(Debug)]
    DefaultFnHelpArgs,

    /// Optional basic auth in the form "name@password" (enables HTTP Basic auth).
    #ok 'a' auth: AuthArg = AuthArg::default(),

    /// Empty string default.
    #ok empty: String = "",
}

#[cfg(feature = "macros")]
sarge! {
    RepeatableVecArgs,
    #ok 'H' headers: Vec<String>,
}

#[cfg(feature = "macros")]
sarge! {
    RepeatableVecEnvArgs,
    #ok @HEADERS headers: Vec<String>,
}

// Test matrix: wrapper type (none/#ok/#err) × default (none/some) × input
// (missing/parse ok/parse err).
sarge! {
    #[derive(Debug, PartialEq, Eq)]
    OkNoDefaultArgs,
    #ok ok_num: u32,
}

sarge! {
    #[derive(Debug, PartialEq, Eq)]
    ErrNoDefaultArgs,
    #err err_num: u32,
}

sarge! {
    #[derive(Debug, PartialEq, Eq)]
    ErrDefaultArgs,
    #err err_num: u32 = 9,
}

sarge! {
    #[derive(Debug, PartialEq, Eq)]
    PlainNoDefaultArgs,
    num: u32,
}

sarge! {
    #[derive(Debug, PartialEq, Eq)]
    PlainDefaultArgs,
    num: u32 = 7,
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
fn httproxy_style_defaults_are_applied() {
    let (args, remainder) =
        HttproxyArgs::parse_cli(["bin"]).expect("failed to parse httproxy args");

    assert_eq!(remainder, vec!["bin"]);
    assert_eq!(
        args.bind.as_ref().map(|b| b.0.as_str()),
        Some("127.0.0.1:8081")
    );
    assert_eq!(
        args.reverse.as_ref().map(|b| b.0.as_str()),
        Some("127.0.0.1:8082")
    );
    assert_eq!(args.log_level.as_ref().map(|l| l.0.as_str()), Some("info"));
    assert_eq!(args.colored, Some(false));
    assert_eq!(args.help, Some(false));
    assert_eq!(
        args.vec.as_deref(),
        Some(&["hello".into(), "nihao".into(), "xxxxx".into()][..])
    );
    assert_eq!(args.vec2.as_deref(), Some(&[1, 2, 3, 4, 5, 6, 7, 8][..]));
}

#[test]
fn repeatable_vec_type_accumulates_in_macro() {
    let (args, remainder) = RepeatableVecArgs::parse_cli([
        "bin",
        "-H",
        "Connection: close",
        "-H",
        "Date: Sun 14 Dec 2025 16:59:06 GMT",
        "-H",
        "a,b",
    ])
    .expect("failed to parse repeatable vec args");

    assert_eq!(remainder, vec!["bin"]);
    assert_eq!(
        args.headers,
        Some(vec![
            "Connection: close".to_string(),
            "Date: Sun 14 Dec 2025 16:59:06 GMT".to_string(),
            "a".to_string(),
            "b".to_string(),
        ])
    );
}

#[test]
fn repeatable_vec_type_accumulates_in_macro_with_long_form() {
    let (args, remainder) =
        RepeatableVecArgs::parse_cli(["bin", "--headers", "a", "--headers", "b,c"])
            .expect("failed to parse repeatable vec args");

    assert_eq!(remainder, vec!["bin"]);
    assert_eq!(
        args.headers,
        Some(vec!["a".to_string(), "b".to_string(), "c".to_string(),])
    );
}

#[test]
fn repeatable_vec_type_cli_overrides_env_in_macro() {
    let env = [("HEADERS", "env1,env2")];
    let cli = ["bin", "--headers", "cli1", "--headers", "cli2"];

    let (args, remainder) = RepeatableVecEnvArgs::parse_provided(cli, env)
        .expect("failed to parse repeatable vec args");

    assert_eq!(remainder, vec!["bin"]);
    assert_eq!(
        args.headers,
        Some(vec!["cli1".to_string(), "cli2".to_string()])
    );
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

#[test]
fn ok_without_default_missing_is_none() {
    let (args, _) = OkNoDefaultArgs::parse_cli(["bin"]).expect("failed to parse ok args");
    assert_eq!(args.ok_num, None);
}

#[test]
fn ok_without_default_parse_success_is_some() {
    let (args, _) =
        OkNoDefaultArgs::parse_cli(["bin", "--ok-num", "123"]).expect("failed to parse ok args");
    assert_eq!(args.ok_num, Some(123));
}

#[test]
fn ok_without_default_parse_failure_is_none() {
    let (args, _) =
        OkNoDefaultArgs::parse_cli(["bin", "--ok-num", "bad"]).expect("failed to parse ok args");
    assert_eq!(args.ok_num, None);
}

#[test]
fn ok_default_parse_success_overrides_default() {
    let (args, _) =
        DefaultArgs::parse_cli(["bin", "--num", "7"]).expect("failed to parse default args");
    assert_eq!(args.num, Some(7));
}

#[test]
fn ok_default_string_overrides_default() {
    let (args, _) = DefaultArgs::parse_cli(["bin", "--target-addr", "x"])
        .expect("failed to parse default args");
    assert_eq!(args.target_addr.as_deref(), Some("x"));
}

#[test]
fn err_without_default_missing_is_none() {
    let (args, _) = ErrNoDefaultArgs::parse_cli(["bin"]).expect("failed to parse err args");
    assert_eq!(args.err_num, None);
}

#[test]
fn err_without_default_parse_success_is_ok() {
    let (args, _) =
        ErrNoDefaultArgs::parse_cli(["bin", "--err-num", "123"]).expect("failed to parse err args");
    assert_eq!(args.err_num, Some(Ok(123)));
}

#[test]
fn err_without_default_parse_failure_is_err() {
    let (args, _) =
        ErrNoDefaultArgs::parse_cli(["bin", "--err-num", "bad"]).expect("failed to parse err args");
    assert!(matches!(args.err_num, Some(Err(_))));
}

#[test]
fn err_default_missing_uses_default() {
    let (args, _) = ErrDefaultArgs::parse_cli(["bin"]).expect("failed to parse err default args");
    assert_eq!(args.err_num, Ok(9));
}

#[test]
fn err_default_parse_success_overrides_default() {
    let (args, _) =
        ErrDefaultArgs::parse_cli(["bin", "--err-num", "10"]).expect("failed to parse err args");
    assert_eq!(args.err_num, Ok(10));
}

#[test]
fn err_default_parse_failure_is_err() {
    let (args, _) =
        ErrDefaultArgs::parse_cli(["bin", "--err-num", "bad"]).expect("failed to parse err args");
    assert!(args.err_num.is_err());
}

#[test]
#[should_panic(expected = "Tried to unwrap argument that wasn't passed")]
fn plain_without_default_missing_panics() {
    let _ = PlainNoDefaultArgs::parse_cli(["bin"]);
}

#[test]
#[should_panic(expected = "Tried to unwrap argument that failed to parse")]
fn plain_without_default_parse_failure_panics() {
    let _ = PlainNoDefaultArgs::parse_cli(["bin", "--num", "bad"]);
}

#[test]
fn plain_without_default_parse_success_is_value() {
    let (args, _) =
        PlainNoDefaultArgs::parse_cli(["bin", "--num", "5"]).expect("failed to parse plain args");
    assert_eq!(args.num, 5);
}

#[test]
fn plain_default_missing_uses_default() {
    let (args, _) = PlainDefaultArgs::parse_cli(["bin"]).expect("failed to parse plain args");
    assert_eq!(args.num, 7);
}

#[test]
fn plain_default_parse_success_overrides_default() {
    let (args, _) =
        PlainDefaultArgs::parse_cli(["bin", "--num", "9"]).expect("failed to parse plain args");
    assert_eq!(args.num, 9);
}

#[test]
#[should_panic(expected = "Tried to unwrap argument that failed to parse")]
fn plain_default_parse_failure_panics() {
    let _ = PlainDefaultArgs::parse_cli(["bin", "--num", "bad"]);
}
