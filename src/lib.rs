use std::{env, error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, Default)]
pub enum ArgType {
    #[default]
    Flag,
    String,
    Integer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tag {
    Single(char),
    Double(String),
    Both(char, String),
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(ch) => write!(f, "-{ch}"),
            Self::Double(s) => write!(f, "--{s}"),
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

        panic!("Cannot get_str({:?})", self);
    }

    /// Returns the contained Integer value. Consumes the enum.
    /// 
    /// Panics if self is ArgValue::Flag or ArgValue::String
    pub fn get_int(self) -> i32 {
        if let Self::Integer(i) = self {
            return i;
        }

        panic!("Cannot get_str({:?})", self);
    }

    /// Returns the contained String value. Consumes the enum.
    /// 
    /// Panics if self is ArgValue::Flag or ArgValue::Integer
    pub fn get_str(self) -> String {
        if let Self::String(s) = self {
            return s;
        }

        panic!("Cannot get_str({:?})", self);
    }
}

#[derive(Debug, Clone)]
pub enum ArgParseError {
    InvalidInteger(String),
    UnknownFlag(String),
    UnexpectedArgument(String),
    MissingArgument(String),
}

impl Display for ArgParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInteger(s) => write!(f, "Invalid integer: {s}"),
            Self::UnknownFlag(s) => write!(f, "Unknown flag: {s}"),
            Self::UnexpectedArgument(s) => write!(f, "Unexpected argument: {s}"),
            Self::MissingArgument(t) => write!(f, "Expected argument for {t}"),
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

    /// Adds an argument to the parser
    pub fn add(&mut self, arg: Argument) {
        self.args.push(arg);
    }

    /// Parse the arguments
    pub fn parse(&mut self) -> Result<Vec<String>, ArgParseError> {
        self.parse_args(env::args().collect::<Vec<_>>())
    }

    /// Parses from the provided arguments
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
            if arg.starts_with('-') {
                let mat = self
                    .matches(arg)
                    .ok_or(ArgParseError::UnexpectedArgument(arg.clone()))?;

                match mat.typ {
                    ArgType::Flag => {
                        mat.val = Some(ArgValue::Flag(true));
                    }
                    ArgType::String => {
                        mat.val = Some(ArgValue::String(
                            args.next()
                                .ok_or(ArgParseError::MissingArgument(arg.clone()))?
                                .clone(),
                        ));
                    }
                    ArgType::Integer => {
                        mat.val = Some(ArgValue::Integer(
                            args.next()
                                .ok_or(ArgParseError::MissingArgument(arg.clone()))?
                                .clone()
                                .parse()
                                .map_err(|_| ArgParseError::MissingArgument(arg.clone()))?,
                        ));
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

    /// Checks a tag to see if it matches any argument
    pub fn matches(&mut self, tag: &str) -> Option<&mut Argument> {
        let tag = Self::tag(tag)?;
        for arg in self.args.iter_mut() {
            match arg.tag.clone() {
                Tag::Single(t) => match tag {
                    Tag::Single(s) => {
                        if t == s {
                            return Some(arg);
                        }
                    }
                    _ => continue,
                },
                Tag::Double(t) => match tag.clone() {
                    Tag::Double(s) => {
                        if t == s {
                            return Some(arg);
                        }
                    }
                    _ => continue,
                },
                Tag::Both(s, t) => match tag.clone() {
                    Tag::Single(a) => {
                        if a == s {
                            return Some(arg);
                        }
                    }
                    Tag::Double(b) => {
                        if b == t {
                            return Some(arg);
                        }
                    }
                    _ => continue,
                },
            }
        }

        None
    }

    /// Turns a str into a tag
    pub fn tag(tag: &str) -> Option<Tag> {
        if let Some(t) = tag.strip_prefix("--") {
            return Some(Tag::Double(t.to_string()));
        } else if let Some(t) = tag.strip_prefix('-') {
            if t.len() == 1 {
                return Some(Tag::Single(t.chars().next().unwrap()));
            }
        }

        None
    }
}

/// Macro to ease argument creation.
/// 
/// Example:
/// ```
/// use sarge::{arg, Tag, ArgType, Argument};
/// 
/// // equivalent to `Argument::new(Flag::Single('h'), ArgType::Flag);`
/// arg!(flag, single, 'h');
/// 
/// // equivalent to `Argument::new(Flag::Double("num".into()), ArgType::Integer);`
/// arg!(int, double, "num");
/// 
/// // equivalent to `Argument::new(Flag::Both('a', "bc".into()), ArgType::String);`
/// arg!(str, both, 'a', "bc");
/// ```
#[macro_export]
macro_rules! arg {
    ( flag, single, $tag:expr ) => {
        Argument::new(Tag::Single($tag.into()), ArgType::Flag)
    };
    ( flag, double, $tag:expr ) => {
        Argument::new(Tag::Double($tag.into()), ArgType::Flag)
    };
    ( flag, both, $single:expr, $double:expr ) => {
        Argument::new(Tag::Both($single.into(), $double.into()), ArgType::Flag)
    };

    ( int, single, $tag:expr ) => {
        Argument::new(Tag::Single($tag.into()), ArgType::Integer)
    };
    ( int, double, $tag:expr ) => {
        Argument::new(Tag::Double($tag.into()), ArgType::Integer)
    };

    ( int, both, $single:expr, $double:expr ) => {
        Argument::new(Tag::Both($single.into(), $double.into()), ArgType::Integer)
    };

    ( str, single, $tag:expr ) => {
        Argument::new(Tag::Single($tag.into()), ArgType::String)
    };
    ( str, double, $tag:expr ) => {
        Argument::new(Tag::Double($tag.into()), ArgType::String)
    };

    ( str, both, $single:expr, $double:expr ) => {
        Argument::new(Tag::Both($single.into(), $double.into()), ArgType::String)
    };
}

/// Macro to ease getting arguments from a parser.
/// 
/// Example:
/// ```
/// use sarge::{get_arg, Tag, ArgType, Argument, ArgumentParser};
/// 
/// let mut parser = ArgumentParser::new();
/// parser.add(Argument::new(Tag::Double("help".into()), ArgType::Flag));
/// 
/// // Equivalent to `parser.arg(Tag::Double("help".into())).expect("Failed to get argument");`
/// get_arg!(parser, double, "help").expect("Failed to get argument");
/// ```
#[macro_export]
macro_rules! get_arg {
    ( $parser:ident, single, $tag:expr ) => {
        $parser.arg(Tag::Single($tag.into()))
    };
    ( $parser:ident, double, $tag:expr ) => {
        $parser.arg(Tag::Double($tag.into()))
    };
    ( $parser:ident, both, $single:expr, $double:expr ) => {
        $parser.arg(Tag::Both($single.into(), $double.into()))
    };
}

/// Like `get_arg!`, but instead returns an option containing its ArgValue
/// 
/// Example:
/// ```
/// use sarge::{arg, get_arg, get_val, Tag, ArgType, ArgValue, Argument, ArgumentParser};
/// 
/// let mut parser = ArgumentParser::new();
/// parser.add(arg!(flag, double, "help"));
/// parser.parse_args(vec!["abc".into(), "--help".into()]);
///
/// assert!(get_val!(parser, double, "help").expect("Failed to get argument value") == ArgValue::Flag(true));
/// ```
#[macro_export]
macro_rules! get_val {
    ( $parser:ident, single, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Single($tag.into())) {
            return arg.val.clone();
        }

        None
    })()};
    ( $parser:ident, double, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Double($tag.into())) {
            return arg.val.clone();
        }

        None
    })()};
    ( $parser:ident, both, $single:expr, $double:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Both($single.into(), $double.into())) {
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
    ( $parser:ident, single, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Single($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_flag();
        }

        panic!("Couldn't find argument {}", Tag::Single($tag.into()));
    })()};
    ( $parser:ident, double, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Double($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_flag();
        }

        panic!("Couldn't find argument {}", Tag::Double($tag.into()));
    })()};
    ( $parser:ident, both, $single:expr, $double:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Both($single.into(), $double.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_flag();
        }

        panic!("Couldn't find argument {}", Tag::Both($single.into(), $double.into()));
    })()};
}

/// Gets an argument from the parser, returning the contained value.
/// 
/// Panics if the argument doesn't exist, or if the arguments type isn't ArgType::Integer.
#[macro_export]
macro_rules! get_int {
    ( $parser:ident, single, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Single($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_int();
        }

        panic!("Couldn't find argument {}", Tag::Single($tag.into()));
    })()};
    ( $parser:ident, double, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Double($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_int();
        }

        panic!("Couldn't find argument {}", Tag::Double($tag.into()));
    })()};
    ( $parser:ident, both, $single:expr, $double:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Both($single.into(), $double.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_int();
        }

        panic!("Couldn't find argument {}", Tag::Both($single.into(), $double.into()));
    })()};
}

/// Gets an argument from the parser, returning the contained value.
/// 
/// Panics if the argument doesn't exist, or if the arguments type isn't ArgType::String.
#[macro_export]
macro_rules! get_str {
    ( $parser:ident, single, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Single($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_str();
        }

        panic!("Couldn't find argument {}", Tag::Single($tag.into()));
    })()};
    ( $parser:ident, double, $tag:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Double($tag.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_str();
        }

        panic!("Couldn't find argument {}", Tag::Double($tag.into()));
    })()};
    ( $parser:ident, both, $single:expr, $double:expr ) => { (|| {
        if let Some(arg) = $parser.arg(Tag::Both($single.into(), $double.into())) {
            return arg.val.clone().expect("Failed to get argument value: arg.val == None").get_str();
        }

        panic!("Couldn't find argument {}", Tag::Both($single.into(), $double.into()));
    })()};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arg_test() {
        let mut parser = ArgumentParser::new();
        parser.add(Argument::new(Tag::Double("name".into()), ArgType::String));
        parser.add(Argument::new(Tag::Single('h'), ArgType::Flag));

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
            parser
                .arg(Tag::Double("name".into()))
                .expect("Couldn't find tag --name")
                .val,
            Some(ArgValue::String("Jonah".into()))
        );

        assert_eq!(
            parser
                .arg(Tag::Single('h'))
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
                .arg(Tag::Double("name".into()))
                .expect("Couldn't find tag --name")
                .val,
            None
        );

        assert_eq!(
            parser
                .arg(Tag::Single('h'))
                .expect("Couldn't find tag -h")
                .val,
            Some(ArgValue::Flag(true))
        );

        assert_eq!(remainder[0], "Jonah".to_string());
    }

    #[test]
    fn macros() {
        let mut parser = ArgumentParser::new();
        parser.add(arg!(flag, single, 'a'));
        parser.add(arg!(flag, double, "bc"));
        parser.add(arg!(flag, both, 'd', "ef"));

        parser.add(arg!(int, single, 'g'));
        parser.add(arg!(int, double, "hi"));
        parser.add(arg!(int, both, 'j', "kl"));

        parser.add(arg!(str, single, 'm'));
        parser.add(arg!(str, double, "no"));
        parser.add(arg!(str, both, 'p', "qr"));

        get_arg!(parser, single, 'a').expect("Failed to get -a");
        get_arg!(parser, double, "bc").expect("Failed to get --bc");
        get_arg!(parser, both, 'd', "ef").expect("Failed to get -d, --ef");

        get_arg!(parser, single, 'g').expect("Failed to get -g");
        get_arg!(parser, double, "hi").expect("Failed to get --hi");
        get_arg!(parser, both, 'j', "kl").expect("Failed to get -j, --kl");

        get_arg!(parser, single, 'm').expect("Failed to get -m");
        get_arg!(parser, double, "no").expect("Failed to get --no");
        get_arg!(parser, both, 'p', "qr").expect("Failed to get -p, --qr");

        parser.parse_args(vec![
            "abc", "-a", "--hi", "10", "-p", "Jack"
        ].iter().map(|s| s.to_string()).collect()).expect("Failed to parse args");

        assert_eq!(get_flag!(parser, single, 'a'), true);
        assert_eq!(get_int!(parser, double, "hi"), 10);
        assert_eq!(get_str!(parser, both, 'p', "qr"), "Jack".to_string());
    }
}
