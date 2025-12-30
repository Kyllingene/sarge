use sarge::prelude::*;

fn main() {
    let mut parser = ArgumentReader::new();
    parser.doc = Some("An example demonstrating automatic documentation generation.".into());
    parser.add::<bool>(tag::both('a', "abc").env("ABC").doc("Super duper docs"));
    parser.add::<bool>(tag::short('b').env("BAR"));
    parser.add::<String>(tag::long("baz-arg").default_value("baz"));
    parser.add::<u32>(tag::both('f', "foo").doc("Hello, World!"));
    parser.add::<bool>(tag::short('x').doc("Testing testing 123"));
    parser.add::<bool>(tag::long("xy").doc("Testing testing 456"));
    parser.add::<Vec<i8>>(tag::env("ENV_ONLY").doc(
        "This is really, really long, multiline argument\ndocumentation, it'll wrap nicely I hope",
    ));

    parser.print_help();
}
