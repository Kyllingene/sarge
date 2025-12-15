#[cfg(feature = "help")]
use crate::prelude::*;

#[cfg(feature = "help")]
#[test]
fn help_returns_string() {
    let mut parser = ArgumentReader::new();
    parser.doc = Some("Program docs".to_string());

    let _name = parser.add::<String>(tag::long("name").doc("Name to use"));
    let _help = parser.add::<bool>(tag::short('h').doc("Print help"));

    let s = parser.help();
    assert!(s.contains("[options...] <arguments...>"));
    assert!(s.contains("Program docs"));
    assert!(s.contains("--name"));
    assert!(s.contains("-h"));
    assert!(s.contains("Name to use"));
    assert!(s.contains("Print help"));
}
