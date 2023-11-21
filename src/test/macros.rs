use crate::prelude::*;

sarge! {
    Args,
    first: bool,
    @ok second: Option<String>,
    @err third: Result<Vec<i64>, error_type!(Vec<i64>)>,

    // #[short = 'f']
    fourth: f64,
}

#[test]
fn test_arguments() {
    let args = Args::parse_args(&[
        "--first".to_string(),
        "--third=123,456,789".to_string(),
        "-f 10.11".to_string(),
    ]).expect("failed to parse arguments");

    assert_eq!(args.first, true);
    assert_eq!(args.second, None);
    assert_eq!(args.third, Ok(vec![123, 456, 789]));
    assert_eq!(args.fourth, 10.11);
}
