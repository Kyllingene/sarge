use crate::prelude::*;

sarge! {
    Args,
    first: bool,
    @ok second: String,
    @err third: Vec<i64>,
    'f' fourth: f64,
}

#[test]
fn test_macros() {
    let (args, _) = Args::parse_args(&[
        "--first".to_string(),
        "--third".to_string(), "123,456,789".to_string(),
        "-f".to_string(), "10.11".to_string(),
    ]).expect("failed to parse arguments");

    assert!(args.first);
    assert_eq!(args.second, None);
    assert_eq!(args.third, Ok(vec![123, 456, 789]));
    assert_eq!(args.fourth, 10.11);
}
