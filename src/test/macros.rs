use crate::{prelude::*, ArgParseError};

#[derive(Arguments)]
struct Args {
    first: bool,
    second: Option<String>,
    third: Result<Vec<i64>, ArgParseError>,

    #[short = 'f']
    fourth: f64,
}

#[test]
fn test_arguments() {
    let mut args = Args::new();

    args.parse_args(&[
        "--first".to_string(),
        "--third=123,456,789".to_string(),
        "-f 10.11".to_string(),
    ]).expect("failed to parse arguments");

    assert_eq!(args.first, true);
    assert_eq!(args.second, None);
    assert_eq!(args.third, Ok(vec![123, 456, 789]));
    assert_eq!(args.fourth, 10.11);
}
