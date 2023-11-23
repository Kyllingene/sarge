use std::convert::Infallible;

use crate::{prelude::*, ArgumentType, ArgResult};

#[derive(Debug, PartialEq, Eq)]
struct MyCustomType(Vec<String>);

impl ArgumentType for MyCustomType {
    type Error = Infallible;

    fn from_value(val: Option<&str>) -> ArgResult<Self> {
        Some(Ok(Self(
            val?.split(' ')
                .map(|s| s.to_string())
                .collect()
        )))
    }
}

#[test]
fn custom_type() {
    let parser = ArgumentParser::new();
    let my_argument = parser.add(tag::long("myarg"));

    let arguments = [
        "custom_type_test".to_string(),
        "--myarg".to_string(),
        "Hello World !".to_string(),
    ];

    let _ = parser.parse_cli(&arguments, false).expect("failed to parse arguments");

    assert_eq!(
        my_argument.get(),
        Some(Ok(MyCustomType(
            vec![
                "Hello".to_string(),
                "World".to_string(),
                "!".to_string(),
            ]
        )))
    );
}
