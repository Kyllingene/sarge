use crate::prelude::*;

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
