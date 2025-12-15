use crate::prelude::*;

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
