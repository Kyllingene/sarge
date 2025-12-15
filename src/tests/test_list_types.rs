use crate::prelude::*;

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
