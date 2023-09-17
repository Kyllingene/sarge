use crate::prelude::*;

mod custom_type;

#[test]
fn basic_arg_test() {
    let parser = ArgumentParser::new();
    let name = parser.add(tag::long("name"));
    let help = parser.add(tag::short('h'));

    let args = ["abc".to_string(), "--name".to_string(), "Jonah".to_string()];

    match parser.parse_args(&args) {
        Err(e) => {
            panic!("Failed to parse first arguments: {}", e);
        }
        _ => {}
    }

    assert_eq!(parser.binary(), Some("abc".into()));

    assert_eq!(name.get_keep(), Ok("Jonah".to_string()));

    assert_eq!(help.get_keep(), Ok(false));

    let args = ["abc".to_string(), "-h".to_string(), "Jonah".to_string()];

    let remainder = match parser.parse_args(&args) {
        Err(e) => {
            panic!("Failed to parse second arguments: {}", e);
        }
        Ok(r) => r,
    };

    assert_eq!(name.get(), Err(()));

    assert_eq!(help.get(), Ok(true));

    assert_eq!(remainder[0], "Jonah".to_string());
}

#[test]
fn multiple_short() {
    let parser = ArgumentParser::new();
    let a = parser.add(tag::short('a'));
    let b = parser.add(tag::short('b'));
    let c = parser.add(tag::short('c'));
    let d = parser.add(tag::short('d'));

    let args = ["test".to_string(), "-abc".to_string()];
    parser.parse_args(&args).expect("Failed to parse args");

    assert_eq!(a.get(), Ok(true));
    assert_eq!(b.get(), Ok(true));
    assert_eq!(c.get(), Ok(true));
    assert_eq!(d.get(), Ok(false));
}

#[test]
fn multiple_short_vals() {
    let parser = ArgumentParser::new();
    let a = parser.add(tag::short('a'));
    let b = parser.add(tag::short('b'));
    let c = parser.add(tag::short('c'));
    let d = parser.add::<i64>(tag::short('d'));

    // This test and the next ensure no regressions in &Vec
    parser
        .parse_args(
            &vec!["test", "-abc", "test"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
        .expect("Failed to parse args");

    assert_eq!(a.get(), Ok(true));
    assert_eq!(b.get(), Ok(true));
    assert_eq!(c.get(), Ok("test".to_string()));
    assert_eq!(d.get(), Err(()));
}

#[test]
#[should_panic(expected = "ConsumedValue")]
fn multiple_short_vals_consume_same_value() {
    let parser = ArgumentParser::new();
    let _a = parser.add::<bool>(tag::short('a'));
    let _b = parser.add::<bool>(tag::short('b'));
    let _c = parser.add::<String>(tag::short('c'));
    let _d = parser.add::<String>(tag::short('d'));

    parser
        .parse_args(
            &vec!["test", "-abcd", "test"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
        .unwrap();
}

#[test]
fn list_type() {
    let parser = ArgumentParser::new();
    let list = parser.add(tag::long("list"));

    let args = [
        "test".to_string(),
        "--list".to_string(),
        "Hello,World,!".to_string(),
    ];

    let _ = parser.parse_args(&args).expect("failed to parse arguments");

    assert_eq!(
        list.get(),
        Ok(
            vec![
                "Hello".to_string(),
                "World".to_string(),
                "!".to_string(),
            ]
        )
    );
}
