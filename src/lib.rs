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

#[derive(Debug, Clone)]
pub struct Argument {
    tag: Tag,
    typ: ArgType,

    val: Option<ArgValue>,
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
}

impl ArgumentParser {
    /// Returns an empty
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an argument to the parser
    pub fn add(&mut self, arg: Argument) {
        self.args.push(arg);
    }

    /// Parse the arguments
    pub fn parse(&mut self) -> Result<(&Vec<Argument>, Vec<String>), ArgParseError> {
        self.parse_args(env::args().collect::<Vec<_>>())
    }

    /// Parses from the provided arguments
    pub fn parse_args(
        &mut self,
        args: Vec<String>,
    ) -> Result<(&Vec<Argument>, Vec<String>), ArgParseError> {
        let mut args = args.iter();
        args.next();

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

        Ok((&self.args, remainder))
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
            Ok(r) => { r.1 }
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
}
