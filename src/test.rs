use crate::prelude::*;

mod custom_type;

#[cfg(feature = "macros")]
mod macros;

#[cfg(feature = "help")]
#[test]
fn help_returns_string() {
    let mut parser = ArgumentReader::new();
    parser.doc = Some("Program docs".to_string());

    let _name = parser.add::<String>(tag::long("name").doc("Name to use"));
    let _help = parser.add::<bool>(tag::short('h').doc("Print help"));

    let s = parser.help();
    assert!(s.contains("[options...] <arguments...>"));
    assert!(s.contains("Program docs"));
    assert!(s.contains("--name"));
    assert!(s.contains("-h"));
    assert!(s.contains("Name to use"));
    assert!(s.contains("Print help"));
}

#[test]
fn basic_arg_test() {
    let mut parser = ArgumentReader::new();
    let name = parser.add(tag::long("name"));
    let help = parser.add(tag::short('h'));

    let args = ["test", "--name", "Jonah"];

    let args = parser
        .clone()
        .parse_cli(args)
        .expect("Failed to parse first arguments");

    assert_eq!(args.remainder(), &["test".to_string()]);
    assert_eq!(name.get(&args), Some(Ok("Jonah".to_string())));
    assert_eq!(help.get(&args), Some(Ok(false)));

    let args = ["test", "-h", "Jonah"];

    let args = parser
        .parse_cli(args)
        .expect("Failed to parse second arguments");

    assert_eq!(args.remainder(), &["test", "Jonah"]);
    assert_eq!(name.get(&args), None);
    assert_eq!(help.get(&args), Some(Ok(true)));
}

#[test]
fn multiple_short() {
    let mut parser = ArgumentReader::new();
    let a = parser.add(tag::short('a'));
    let b = parser.add(tag::short('b'));
    let c = parser.add(tag::short('c'));
    let d = parser.add(tag::short('d'));

    let args = ["test", "-abc"];
    let args = parser.parse_cli(args).expect("Failed to parse args");

    assert_eq!(a.get(&args), Some(Ok(true)));
    assert_eq!(b.get(&args), Some(Ok(true)));
    assert_eq!(c.get(&args), Some(Ok(true)));
    assert_eq!(d.get(&args), Some(Ok(false)));
}

#[test]
fn multiple_short_vals() {
    let mut parser = ArgumentReader::new();
    let a = parser.add(tag::short('a'));
    let b = parser.add(tag::short('b'));
    let c = parser.add(tag::short('c'));
    let d = parser.add::<i64>(tag::short('d'));

    let args = ["test", "-abc", "test"];

    let args = parser.parse_cli(args).expect("Failed to parse args");

    assert_eq!(a.get(&args), Some(Ok(true)));
    assert_eq!(b.get(&args), Some(Ok(true)));
    assert_eq!(c.get(&args), Some(Ok("test".to_string())));
    assert_eq!(d.get(&args), None);
}

#[test]
#[should_panic(expected = "ConsumedValue")]
fn multiple_short_vals_consume_same_value() {
    let mut parser = ArgumentReader::new();
    let _a = parser.add::<bool>(tag::short('a'));
    let _b = parser.add::<bool>(tag::short('b'));
    let _c = parser.add::<String>(tag::short('c'));
    let _d = parser.add::<String>(tag::short('d'));

    let args = ["test", "-abcd", "test"];

    parser.parse_cli(args).unwrap();
}

#[test]
fn list_type() {
    let mut parser = ArgumentReader::new();
    let list = parser.add(tag::long("list"));

    let args = ["test", "--list", "Hello,World,!"];

    let args = parser.parse_cli(args).expect("failed to parse arguments");

    assert_eq!(
        list.get(&args),
        Some(Ok(vec![
            "Hello".to_string(),
            "World".to_string(),
            "!".to_string(),
        ]))
    );
}

#[test]
fn int_list_type() {
    let mut parser = ArgumentReader::new();
    let list = parser.add(tag::long("list"));

    let args = ["test", "--list", "123,456,789"];

    let args = parser.parse_cli(args).expect("failed to parse arguments");

    assert_eq!(list.get(&args), Some(Ok(vec![123i64, 456, 789,])));
}

#[test]
fn repeatable_list_type_accumulates_values() {
    let mut parser = ArgumentReader::new();
    let list = parser.add(tag::short('H'));

    let args = ["test", "-H", "xx", "-H", "xxx", "-H", "sadfh,sjffk"];

    let args = parser.parse_cli(args).expect("failed to parse arguments");

    assert_eq!(
        list.get(&args),
        Some(Ok(vec![
            "xx".to_string(),
            "xxx".to_string(),
            "sadfh".to_string(),
            "sjffk".to_string(),
        ]))
    );
}

#[test]
fn repeatable_list_type_cli_overrides_env() {
    let mut parser = ArgumentReader::new();
    let list = parser.add(tag::long("list").env("LIST"));

    let env = [("LIST", "env1,env2")];
    let cli = ["test", "--list", "cli1", "--list", "cli2"];

    let args = parser
        .parse_provided(cli, env)
        .expect("failed to parse provided arguments");

    assert_eq!(
        list.get(&args),
        Some(Ok(vec!["cli1".to_string(), "cli2".to_string()]))
    );
}

#[test]
fn basic_env_var() {
    let mut parser = ArgumentReader::new();
    let cfg = parser.add(tag::env("CONFIG_DIR"));

    let args = [("CONFIG_DIR", "/cfg")];

    let args = parser
        .parse_provided(&[] as &[String], args)
        .expect("Failed to parse environment");

    assert_eq!(cfg.get(&args), Some(Ok("/cfg".to_string())));
}

#[test]
fn many_env_vars() {
    let mut parser = ArgumentReader::new();
    let cfg = parser.add(tag::env("CONFIG_DIR"));
    let silent = parser.add(tag::env("SILENT"));
    let threads = parser.add(tag::env("THREADS"));
    let unused = parser.add::<i64>(tag::env("UNUSED"));

    let args = [("CONFIG_DIR", "/cfg"), ("SILENT", "0"), ("THREADS", "16")];

    let args = parser
        .parse_provided(&[] as &[String], args)
        .expect("Failed to parse environment");

    assert_eq!(cfg.get(&args), Some(Ok("/cfg".to_string())));
    assert_eq!(silent.get(&args), Some(Ok(false)));
    assert_eq!(threads.get(&args), Some(Ok(16u64)));
    assert_eq!(unused.get(&args), None);
}
