use crate::prelude::*;

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
