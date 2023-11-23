use crate::{ArgumentType, ArgResult, prelude::*};

#[derive(Debug, PartialEq, Eq)]
struct MyCustomType(Vec<String>);

impl ArgumentType for MyCustomType {
    type Error = ();

    fn from_value(val: Option<&str>) -> ArgResult<Self> {
        Some(Ok(Self(val?.split(' ').map(|s| s.to_string()).collect())))
    }
}

#[test]
fn custom_type() {
    let parser = ArgumentParser::new();
    let my_argument = parser.add::<MyCustomType>(tag::long("list"));

    let args = [
        "custom_type_test".to_string(),
        "--list".to_string(),
        "Hello World !".to_string(),
    ];

    let _ = parser
        .parse_cli(&args, false)
        .expect("failed to parse arguments");

    assert_eq!(
        my_argument.get(),
        Some(Ok(MyCustomType(vec![
            "Hello".to_string(),
            "World".to_string(),
            "!".to_string(),
        ])))
    );
}
