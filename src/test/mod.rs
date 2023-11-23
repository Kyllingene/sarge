use crate::prelude::*;

mod custom_type;

#[cfg(feature = "macros")]
mod macros;

#[allow(unused)]
macro_rules! create_args {
    ( $( $arg:expr ),* $(,)? ) => {
        [ $( $arg.to_string(), )* ]
    };
}

#[allow(unused)]
macro_rules! create_env {
    ( $( $name:expr, $val:expr ),* $(,)? ) => {
        [ $( ($name.to_string(), $val.to_string()), )* ]
    };
}

#[test]
fn basic_arg_test() {
    let parser = ArgumentParser::new();
    let name = parser.add(tag::long("name"));
    let help = parser.add(tag::short('h'));

    let args = create_args!["test", "--name", "Jonah"];

    let remainder = parser
        .parse_cli(&args, false)
        .expect("Failed to parse first arguments");

    assert_eq!(parser.binary(), Some("test".into()));
    assert_eq!(name.get_keep(), Some(Ok("Jonah".to_string())));
    assert_eq!(help.get_keep(), Some(Ok(false)));
    assert!(remainder.is_empty());

    let args = create_args!["test", "-h", "Jonah"];

    let remainder = parser
        .parse_cli(&args, true)
        .expect("Failed to parse second arguments");

    assert_eq!(name.get(), None);
    assert_eq!(help.get(), Some(Ok(true)));
    assert_eq!(remainder.get(0), Some(&"Jonah".to_string()));
}

#[test]
fn multiple_short() {
    let parser = ArgumentParser::new();
    let a = parser.add(tag::short('a'));
    let b = parser.add(tag::short('b'));
    let c = parser.add(tag::short('c'));
    let d = parser.add(tag::short('d'));

    let args = create_args!["test", "-abc"];
    parser
        .parse_cli(&args, false)
        .expect("Failed to parse args");

    assert_eq!(a.get(), Some(Ok(true)));
    assert_eq!(b.get(), Some(Ok(true)));
    assert_eq!(c.get(), Some(Ok(true)));
    assert_eq!(d.get(), Some(Ok(false)));
}

#[test]
fn multiple_short_vals() {
    let parser = ArgumentParser::new();
    let a = parser.add(tag::short('a'));
    let b = parser.add(tag::short('b'));
    let c = parser.add(tag::short('c'));
    let d = parser.add::<i64>(tag::short('d'));

    let args = create_args!["test", "-abc", "test"];

    parser
        .parse_cli(&args, false)
        .expect("Failed to parse args");

    assert_eq!(a.get(), Some(Ok(true)));
    assert_eq!(b.get(), Some(Ok(true)));
    assert_eq!(c.get(), Some(Ok("test".to_string())));
    assert_eq!(d.get(), None);
}

#[test]
#[should_panic(expected = "ConsumedValue")]
fn multiple_short_vals_consume_same_value() {
    let parser = ArgumentParser::new();
    let _a = parser.add::<bool>(tag::short('a'));
    let _b = parser.add::<bool>(tag::short('b'));
    let _c = parser.add::<String>(tag::short('c'));
    let _d = parser.add::<String>(tag::short('d'));

    let args = create_args!["test", "-abcd", "test"];

    parser.parse_cli(&args, false).unwrap();
}

#[test]
fn list_type() {
    let parser = ArgumentParser::new();
    let list = parser.add(tag::long("list"));

    let args = create_args![
        "test",
        "--list",
        "Hello,World,!",
    ];

    let _ = parser
        .parse_cli(&args, false)
        .expect("failed to parse arguments");

    assert_eq!(
        list.get(),
        Some(Ok(vec![
            "Hello".to_string(),
            "World".to_string(),
            "!".to_string(),
        ]))
    );
}

#[test]
fn int_list_type() {
    let parser = ArgumentParser::new();
    let list = parser.add(tag::long("list"));

    let args = create_args![
        "test",
        "--list",
        "123,456,789",
    ];

    let _ = parser
        .parse_cli(&args, false)
        .expect("failed to parse arguments");

    assert_eq!(
        list.get(),
        Some(Ok(vec![
            123i64,
            456,
            789,
        ]))
    );
}

#[test]
fn basic_env_var() {
    let parser = ArgumentParser::new();
    let cfg = parser.add(tag::env("CONFIG_DIR"));

    let args = create_env!["CONFIG_DIR", "/cfg"];

    parser
        .parse_env(args.into_iter(), false)
        .expect("Failed to parse environment");

    assert_eq!(cfg.get(), Some(Ok("/cfg".to_string())));
}

#[test]
fn many_env_vars() {
    let parser = ArgumentParser::new();
    let cfg = parser.add(tag::env("CONFIG_DIR"));
    let silent = parser.add(tag::env("SILENT"));
    let threads = parser.add(tag::env("THREADS"));
    let unused = parser.add::<i64>(tag::env("UNUSED"));

    let args = create_env!["CONFIG_DIR", "/cfg", "SILENT", "0", "THREADS", "16",];

    parser
        .parse_env(args.into_iter(), false)
        .expect("Failed to parse environment");

    assert_eq!(cfg.get(), Some(Ok("/cfg".to_string())));
    assert_eq!(silent.get(), Some(Ok(false)));
    assert_eq!(threads.get(), Some(Ok(16u64)));
    assert_eq!(unused.get(), None);
}
