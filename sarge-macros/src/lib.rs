use proc_macro::{TokenTree, TokenStream};

const IMPLEMENTATION: &str = r##"
impl crate::Arguments for {{NAME}} {
    fn new() -> Self {
        todo!()
    }

    fn parse_args(&mut self, args: &[String]) -> Result<(), crate::ArgParseError> {
        todo!()
    }
}
"##;

#[derive(PartialEq)]
enum State {
    Initial,
    Name,
    Parameters,
    InParam {
        public: bool,
        name: String,
        typ: Option<String>,
        short: Option<char>,
    },
    Complete
}

#[derive(PartialEq)]
struct Param {
    public: bool,
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

        struct : Initial -> Name
        Args : Name -> Parameters
        first: Parameters -> InParam
*/

#[proc_macro_derive(Arguments, attributes(short))]
pub fn derive_arguments_struct(item: TokenStream) -> TokenStream {
    let toks = item.into_iter();

    let mut state = State::Initial;

    let mut name: Option<String> = None;
    // let mut current: Option<Param> = None;
    let mut params: Vec<Param> = Vec::new();

    for tok in toks {
        match tok {
            TokenTree::Group(_) => {},
            TokenTree::Ident(i) => {
                let ident = i.to_string();
                match ident.as_str() {
                    "pub" => match state {
                        State::Initial => continue,
                        State::Parameters => state = State::InParam { public: true, name: String::new(), typ: None, short: None },
                        _ => panic!("Unexpected token: `pub`"),
                    },
                    "struct" => {
                        if state != State::Initial {
                            panic!("Unexpected identifier: `{ident}`");
                        }

                        state = State::Name;
                        continue;
                    },
                    _ => {
                        match &mut state {
                            State::Initial => panic!("Unexpected identifier: `{ident}`"),
                            State::Name => {
                                state = State::Parameters;
                                name = Some(ident);
                                continue;
                            },
                            State::Parameters => {
                                state = State::InParam { public: false, name: ident, typ: None, short: None };
                                // current = Some(Param { name: ident, typ: String::new(), short: None });
                                continue;
                            }
                            State::InParam { typ, .. } => {
                                *typ = Some(ident);
                            }
                            State::Complete => panic!("Internal error: tried to parse more after State::Complete"),
                        }
                    }
                }
            },
            TokenTree::Punct(punct) => match punct.as_char() {
                '}' => match state {
                    State::Parameters => {
                        state = State::Complete;
                        break;
                    }
                    _ => panic!("Unexpected token: `}}`"),
                },
                _ => {}
            },
            TokenTree::Literal(_) => {},
        }
    }

    if state != State::Complete {
        panic!("Internal error: state != Complete");
    }

    let new = IMPLEMENTATION.to_string().replace("{{NAME}}", &name.expect("Expected name for struct"));
    new.parse().unwrap()
}
