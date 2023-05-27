use std::{env, error::Error, fmt::Display, num::ParseIntError};

#[derive(Debug, Clone, Copy, Default)]
pub enum ArgType {
    #[default]
    Flag,
    String,
    Integer,
}

#[derive(Debug, Clone)]
pub enum Tag {
    Short(char),
    Long(String),
    Both(char, String),
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Short(s) => {
                match other {
                    Self::Short(o) => s == o,
                    Self::Both(o, _) => s == o,
                    _ => false,
                }
            }
            Self::Long(s) => {
                match other {
                    Self::Long(o) => s == o,
                    Self::Both(_, o) => s == o,
                    _ => false,
                }
            }
            Self::Both(s1, s2) => {
                match other {
                    Self::Short(o) => s1 == o,
                    Self::Long(o) => s2 == o,
                    Self::Both(o1, o2) => (s1 == o1) || (s2 == o2),
                }
            }
        }
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Short(ch) => write!(f, "-{ch}"),
            Self::Long(s) => write!(f, "--{s}"),
            Self::Both(ch, s) => write!(f, "-{ch} / --{s}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Argument {
    tag: Tag,
    typ: ArgType,

    pub val: Option<ArgValue>,
}

impl Argument {
    /// Creates a new argument.
    pub fn new(tag: Tag, typ: ArgType) -> Self {
        Self {
            tag,
            typ,

            val: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgValue {
    Flag(bool),
    String(String),
    Integer(i32),
}

impl ArgValue {
    /// Returns the contained Flag value. Consumes the enum.
    /// 
    /// Panics if self is ArgValue::Integer or ArgValue::String
    pub fn get_flag(self) -> bool {
        if let Self::Flag(b) = self {
            return b;
        }

        panic!("Cannot get_flag({self:?})");
    }

    /// Returns the contained Integer value. Consumes the enum.
    /// 
    /// Panics if self is ArgValue::Flag or ArgValue::String
    pub fn get_int(self) -> i32 {
        if let Self::Integer(i) = self {
            return i;
        }

        panic!("Cannot get_int({self:?})");
    }

    /// Returns the contained String value. Consumes the enum.
    /// 
    /// Panics if self is ArgValue::Flag or ArgValue::Integer
    pub fn get_str(self) -> String {
        if let Self::String(s) = self {
            return s;
        }

        panic!("Cannot get_str({self:?})");
    }
}

#[derive(Debug, Clone)]
pub enum ArgParseError {
    InvalidInteger(String),
    UnknownFlag(String),
    UnexpectedArgument(String),
    MissingValue(String),
    ConsumedValue(String),
}

impl Display for ArgParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInteger(s) => write!(f, "Invalid integer: `{s}`"),
            Self::UnknownFlag(s) => write!(f, "Unknown flag: `{s}`"),
            Self::UnexpectedArgument(s) => write!(f, "Unexpected argument: `{s}`"),
            Self::MissingValue(s) => write!(f, "Expected value for `{s}`"),
            Self::ConsumedValue(s) => write!(f, "Multiple arguments in `{s}` tried to consume the same value"),
        }
    }
}

impl Error for ArgParseError {}

#[derive(Debug, Clone, Default)]
pub struct ArgumentParser {
    args: Vec<Argument>,
    pub binary: Option<String>,
}

impl ArgumentParser {
    /// Returns an empty ArgumentParser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an argument to the parser.
    pub fn add(&mut self, arg: Argument) {
        self.args.push(arg);
    }

    /// Parse arguments from std::env::args.
    pub fn parse(&mut self) -> Result<Vec<String>, ArgParseError> {
        self.parse_args(env::args().collect::<Vec<_>>())
    }

    /// Parses the provided arguments.
    pub fn parse_args(&mut self, args: Vec<String>) -> Result<Vec<String>, ArgParseError> {
        let mut args = args.iter();
        self.binary = args.next().cloned();

        let mut remainder = Vec::new();

        self.args.iter_mut().for_each(|arg| {
            arg.val = match arg.typ {
                ArgType::Flag => Some(ArgValue::Flag(false)),
                _ => None,
            };
        });

        while let Some(arg) = args.next() {
            if let Some(mut long) = arg.strip_prefix("--") {
                let val = if let Some((left, right)) = arg.split_once('=') {
                    long = left;
                    Some(right.to_string())
                } else {
                    args.next().cloned()
                };

                let mut arg = self.arg_mut(Tag::Long(long.to_string()))
                    .ok_or(ArgParseError::UnknownFlag(long.to_string()))?;

                match arg.typ {
                    ArgType::Flag => arg.val = Some(ArgValue::Flag(true)),
                    ArgType::Integer => arg.val = Some(
                        ArgValue::Integer(
                            val
                                .ok_or(ArgParseError::MissingValue(long.to_string()))?
                                .parse()
                                .map_err(|e: ParseIntError| ArgParseError::InvalidInteger(e.to_string()))?
                        )
                    ),
                    ArgType::String => arg.val = Some(
                        ArgValue::String(
                            val
                                .ok_or(ArgParseError::MissingValue(long.to_string()))?
                                .clone()
                        )
                    )
                }
            } else if let Some(short) = arg.strip_prefix('-') {
                if short.is_empty() {
                    remainder.push(String::from("-"));
                } else if short.len() == 1 {
                    let mut arg = self.arg_mut(Tag::Short(short.chars().next().unwrap()))
                        .ok_or(ArgParseError::UnknownFlag(short.to_string()))?;

                    match arg.typ {
                        ArgType::Flag => arg.val = Some(ArgValue::Flag(true)),
                        ArgType::Integer => arg.val = Some(
                            ArgValue::Integer(
                                args.next()
                                    .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                    .parse()
                                    .map_err(|e: ParseIntError| ArgParseError::InvalidInteger(e.to_string()))?
                            )
                        ),
                        ArgType::String => arg.val = Some(
                            ArgValue::String(
                                args.next()
                                    .ok_or(ArgParseError::MissingValue(short.to_string()))?
                                    .clone()
                            )
                        )
                    }
                } else {
                    let mut consumed = false;
                    for ch in short.chars() {
                        let mut arg = self.arg_mut(Tag::Short(ch))
                            .ok_or(ArgParseError::UnknownFlag(ch.to_string()))?;

                        match arg.typ {
                            ArgType::Flag => arg.val = Some(ArgValue::Flag(true)),
                            ArgType::Integer => {
                                if consumed {
                                    return Err(ArgParseError::ConsumedValue(short.to_string()));
                                }

                                consumed = true;
                                arg.val = Some(
                                    ArgValue::Integer(
                                        args.next()
                                            .ok_or(ArgParseError::MissingValue(ch.to_string()))?
                                            .parse()
                                            .map_err(|e: ParseIntError| ArgParseError::InvalidInteger(e.to_string()))?
                                    )
                                )
                            },
                            ArgType::String => {
                                if consumed {
                                    return Err(ArgParseError::ConsumedValue(short.to_string()));
                                }

                                consumed = true;
                                arg.val = Some(
                                    ArgValue::String(
                                        args.next()
                                            .ok_or(ArgParseError::MissingValue(ch.to_string()))?
                                            .clone()
                                    )
                                )
                            }
                        }
                    }
                }
            } else {
                remainder.push(arg.clone());
            }
        }

        Ok(remainder)
    }

    /// Finds an argument by tag.
    pub fn arg(&self, tag: Tag) -> Option<&Argument> {
        self.args.iter().find(|&arg| arg.tag == tag)
    }

    /// Returns all the arguments.
    pub fn args(&self) -> &Vec<Argument> {
        &self.args
    }

    /// Finds an argument by tag.
    /// 
    /// Returns a mutable reference.
    fn arg_mut(&mut self, tag: Tag) -> Option<&mut Argument> {
        self.args.iter_mut().find(|arg| arg.tag == tag)
    }
}

/// Macro to ease argument creation.
/// 
/// Example:
/// ```
/// use sarge::arg;
/// 
/// // equivalent to `Argument::new(Flag::Short('h'), ArgType::Flag);`
/// arg!(flag, short, 'h');
/// 
/// // equivalent to `Argument::new(Flag::Long("num".into()), ArgType::Integer);`
/// arg!(int, long, "num");
/// 
/// // equivalent to `Argument::new(Flag::Both('a', "bc".into()), ArgType::String);`
/// arg!(str, both, 'a', "bc");
/// ```
#[macro_export]
macro_rules! arg {
    ( flag, short, $tag:expr ) => {
        $crate::Argument::new($crate::Tag::Short($tag.into()), $crate::ArgType::Flag)
    };
    ( flag, long, $tag:expr ) => {
        $crate::Argument::new($crate::Tag::Long($tag.into()), $crate::ArgType::Flag)
    };
    ( flag, both, $short:expr, $long:expr ) => {
        $crate::Argument::new($crate::Tag::Both($short.into(), $long.into()), $crate::ArgType::Flag)
    };

    ( int, short, $tag:expr ) => {
        $crate::Argument::new($crate::Tag::Short($tag.into()), $crate::ArgType::Integer)
    };
    ( int, long, $tag:expr ) => {
        $crate::Argument::new($crate::Tag::Long($tag.into()), $crate::ArgType::Integer)
    };

    ( int, both, $short:expr, $long:expr ) => {
        $crate::Argument::new($crate::Tag::Both($short.into(), $long.into()), $crate::ArgType::Integer)
    };

    ( str, short, $tag:expr ) => {
        $crate::Argument::new($crate::Tag::Short($tag.into()), $crate::ArgType::String)
    };
    ( str, long, $tag:expr ) => {
        $crate::Argument::new($crate::Tag::Long($tag.into()), $crate::ArgType::String)
    };

    ( str, both, $short:expr, $long:expr ) => {
        $crate::Argument::new($crate::Tag::Both($short.into(), $long.into()), $crate::ArgType::String)
    };
}

/// Macro to ease getting arguments from a parser.
/// 
/// Example:
/// ```
/// use sarge::{get_arg, Tag, ArgType, Argument, ArgumentParser};
/// 
/// let mut parser = ArgumentParser::new();
/// parser.add(Argument::new(Tag::Long("help".into()), ArgType::Flag));
/// 
/// // Equivalent to `parser.arg(Tag::Long("help".into())).expect("Failed to get argument");`
/// get_arg!(parser, long, "help").expect("Failed to get argument");
/// ```
#[macro_export]
macro_rules! get_arg {
    ( $parser:ident, short, $tag:expr ) => {
        $parser.arg($crate::Tag::Short($tag.into()))
    };
    ( $parser:ident, long, $tag:expr ) => {
        $parser.arg($crate::Tag::Long($tag.into()))
    };
    ( $parser:ident, both, $short:expr, $long:expr ) => {
        $parser.arg($crate::Tag::Both($short.into(), $long.into()))
    };
}

/// Like `get_arg!`, but instead returns an option containing its ArgValue
/// 
/// Example:
/// ```
/// use sarge::{arg, get_val, ArgValue, ArgumentParser};
/// 
/// let mut parser = ArgumentParser::new();
/// parser.add(arg!(flag, long, "help"));
/// parser.parse_args(vec!["abc".into(), "--help".into()]);
///
/// assert_eq!(get_val!(parser, long, "help").expect("Failed to get argument value"), ArgValue::Flag(true));
/// ```
#[macro_export]
macro_rules! get_val {
    ( $parser:ident, short, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Short($tag.into())) {
            return arg.val.clone();
        }

        None
    })()};
    ( $parser:ident, long, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Long($tag.into())) {
            return arg.val.clone();
        }

        None
    })()};
    ( $parser:ident, both, $short:expr, $long:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Both($short.into(), $long.into())) {
            return arg.val.clone();
        }

        None
    })()};
}

/// Gets an argument from the parser, returning the contained value.
/// 
/// Panics if the argument doesn't exist, or if the arguments type isn't ArgType::Flag.
#[macro_export]
macro_rules! get_flag {
    ( $parser:ident, short, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Short($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_flag();
        }

        panic!("Couldn't find argument {}", $crate::Tag::Short($tag.into()));
    })()};
    ( $parser:ident, long, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Long($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_flag();
        }

        panic!("Couldn't find argument {}", $crate::Tag::Long($tag.into()));
    })()};
    ( $parser:ident, both, $short:expr, $long:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Both($short.into(), $long.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_flag();
        }

        panic!("Couldn't find argument {}", $crate::Tag::Both($short.into(), $long.into()));
    })()};
}

/// Gets an argument from the parser, returning the contained value.
/// 
/// Panics if the argument doesn't exist, or if the arguments type isn't ArgType::Integer.
#[macro_export]
macro_rules! get_int {
    ( $parser:ident, short, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Short($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_int();
        }

        panic!("Couldn't find argument {}", ::Tag::Short($tag.into()));
    })()};
    ( $parser:ident, long, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Long($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_int();
        }

        panic!("Couldn't find argument {}", $crate::Tag::Long($tag.into()));
    })()};
    ( $parser:ident, both, $short:expr, $long:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Both($short.into(), $long.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_int();
        }

        panic!("Couldn't find argument {}", $crate::Tag::Both($short.into(), $long.into()));
    })()};
}

/// Gets an argument from the parser, returning the contained value.
/// 
/// Panics if the argument doesn't exist, or if the arguments type isn't ArgType::String.
#[macro_export]
macro_rules! get_str {
    ( $parser:ident, short, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Short($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_str();
        }

        panic!("Couldn't find argument {}", $crate::Tag::Short($tag.into()));
    })()};
    ( $parser:ident, long, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Long($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_str();
        }

        panic!("Couldn't find argument {}", $crate::Tag::Long($tag.into()));
    })()};
    ( $parser:ident, both, $short:expr, $long:expr ) => { (|| {
        if let Some(arg) = $parser.arg($crate::Tag::Both($short.into(), $long.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_str();
        }

        panic!("Couldn't find argument {}", $crate::Tag::Both($short.into(), $long.into()));
    })()};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arg_test() {
        let mut parser = ArgumentParser::new();
        parser.add(Argument::new(Tag::Long("name".into()), ArgType::String));
        parser.add(Argument::new(Tag::Short('h'), ArgType::Flag));

        let args: Vec<String> = vec!["abc", "--name", "Jonah"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        match parser.parse_args(args) {
            Err(e) => {
                panic!("Failed to parse first arguments: {}", e);
            }
            _ => {}
        }

        assert_eq!(
            parser.binary,
            Some("abc".into())
        );

        assert_eq!(
            parser
                .arg(Tag::Long("name".into()))
                .expect("Couldn't find tag --name")
                .val,
            Some(ArgValue::String("Jonah".into()))
        );

        assert_eq!(
            parser
                .arg(Tag::Short('h'))
                .expect("Couldn't find tag -h")
                .val,
            Some(ArgValue::Flag(false))
        );

        let args: Vec<String> = vec!["abc", "-h", "Jonah"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let remainder = match parser.parse_args(args) {
            Err(e) => {
                panic!("Failed to parse second arguments: {}", e);
            }
            Ok(r) => r,
        };

        assert_eq!(
            parser
                .arg(Tag::Long("name".into()))
                .expect("Couldn't find tag --name")
                .val,
            None
        );

        assert_eq!(
            parser
                .arg(Tag::Short('h'))
                .expect("Couldn't find tag -h")
                .val,
            Some(ArgValue::Flag(true))
        );

        assert_eq!(remainder[0], "Jonah".to_string());
    }

    #[test]
    fn macros() {
        let mut parser = ArgumentParser::new();
        parser.add(arg!(flag, short, 'a'));
        parser.add(arg!(flag, long, "bc"));
        parser.add(arg!(flag, both, 'd', "ef"));

        parser.add(arg!(int, short, 'g'));
        parser.add(arg!(int, long, "hi"));
        parser.add(arg!(int, both, 'j', "kl"));

        parser.add(arg!(str, short, 'm'));
        parser.add(arg!(str, long, "no"));
        parser.add(arg!(str, both, 'p', "qr"));

        get_arg!(parser, short, 'a').expect("Failed to get -a");
        get_arg!(parser, long, "bc").expect("Failed to get --bc");
        get_arg!(parser, both, 'd', "ef").expect("Failed to get -d, --ef");

        get_arg!(parser, short, 'g').expect("Failed to get -g");
        get_arg!(parser, long, "hi").expect("Failed to get --hi");
        get_arg!(parser, both, 'j', "kl").expect("Failed to get -j, --kl");

        get_arg!(parser, short, 'm').expect("Failed to get -m");
        get_arg!(parser, long, "no").expect("Failed to get --no");
        get_arg!(parser, both, 'p', "qr").expect("Failed to get -p, --qr");

        parser.parse_args(vec![
            "abc", "-a", "--hi", "10", "-p", "Jack"
        ].iter().map(|s| s.to_string()).collect()).expect("Failed to parse args");

        assert_eq!(get_flag!(parser, short, 'a'), true);
        assert_eq!(get_int!(parser, long, "hi"), 10);
        assert_eq!(get_str!(parser, both, 'p', "qr"), "Jack".to_string());
    }

    #[test]
    fn multiple_short() {
        let mut parser = ArgumentParser::new();
        parser.add(arg!(flag, short, 'a'));
        parser.add(arg!(flag, short, 'b'));
        parser.add(arg!(flag, short, 'c'));
        parser.add(arg!(flag, short, 'd'));

        parser.parse_args(vec![
            "test", "-abc"]
        .iter().map(|s| s.to_string()).collect()).expect("Failed to parse args");

        assert_eq!(get_flag!(parser, short, 'a'), true);
        assert_eq!(get_flag!(parser, short, 'b'), true);
        assert_eq!(get_flag!(parser, short, 'c'), true);
        assert_eq!(get_flag!(parser, short, 'd'), false);
    }

    #[test]
    fn multiple_short_vals() {
        let mut parser = ArgumentParser::new();
        parser.add(arg!(flag, short, 'a'));
        parser.add(arg!(flag, short, 'b'));
        parser.add(arg!(str, short, 'c'));
        parser.add(arg!(str, short, 'd'));

        parser.parse_args(vec![
            "test", "-abc", "test"]
        .iter().map(|s| s.to_string()).collect()).expect("Failed to parse args");

        assert_eq!(get_flag!(parser, short, 'a'), true);
        assert_eq!(get_flag!(parser, short, 'b'), true);
        assert_eq!(get_str!(parser, short, 'c'), "test".to_string());
        assert_eq!(get_val!(parser, short, 'd'), None);
    }

    #[test]
    #[should_panic(expected = "ConsumedValue")]
    fn multiple_short_vals_consume_same_value() {
        let mut parser = ArgumentParser::new();
        parser.add(arg!(flag, short, 'a'));
        parser.add(arg!(flag, short, 'b'));
        parser.add(arg!(str, short, 'c'));
        parser.add(arg!(str, short, 'd'));

        parser.parse_args(vec![
            "test", "-abcd", "test"
        ].iter().map(|s| s.to_string()).collect()).unwrap();
    }
}
