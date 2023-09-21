use proc_macro::{TokenTree, TokenStream};

#[derive(PartialEq)]
enum State {
    Initial,
    Name,
    Parameters,
    InParam {
        name: String,
        typ: String,
        short: Option<char>,
    },
    Complete
}

#[derive(PartialEq)]
struct Param {
    name: String,
    typ: String,
    short: Option<char>,
}

/*
    example:

        struct Args {
            first: bool,
            second: Option<String>,
            third: Result<Vec<i64>, ArgParseError>,

            #[short = 'f']
            fourth: f64,
        }
*/

#[proc_macro_derive(Arguments, attributes(short))]
pub fn derive_arguments_struct(item: TokenStream) -> TokenStream {
    let toks = item.into_iter();

    let mut state = State::Initial;

    let mut name: Option<String> = None;
    let mut current: Option<Param> = None;
    let mut params: Vec<Param> = Vec::new();

    for tok in toks {
        match tok {
            TokenTree::Group(_) => {},
            TokenTree::Ident(i) => {
                let ident = i.to_string();
                match ident.as_str() {
                    "pub" => continue,
                    "struct" => {
                        if state != State::Initial {
                            panic!("Unexpected identifier: `{ident}`");
                        }

                        state = State::Name;
                        continue;
                    },
                    _ => {
                        match state {
                            State::Initial => panic!("Unexpected identifier: `{ident}`"),
                            State::Name => {
                                state = State::Name;
                                name = Some(ident);
                                continue;
                            },
                            State::Parameters => {
                                current = Some(Param { name: ident, typ: String::new(), short: None });
                            }
                            State::InParam { name, typ, short } => todo!(),
                            State::Complete => panic!("Internal error: tried to parse more after State::Complete"),
                        }
                    }
                }
            },
            TokenTree::Punct(_) => {},
            TokenTree::Literal(_) => {},
        }
    }

    if state != State::Complete {
        panic!("Internal error: state != Complete");
    }

    let new = include_str!("implement.rs").to_string().replace("{{NAME}}", &name.expect("Expected name for struct"));
    new.parse().unwrap()
}
