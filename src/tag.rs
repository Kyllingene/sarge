//! Everything surrounding [tags](`FullTag`).

use std::{fmt::Display, hash::Hash};

/// Create a tag with just a short variant.
#[inline]
pub fn short<S: Into<char>>(s: S) -> FullTag {
    FullTag::from(Tag::Short(s.into()))
}

/// Create a tag with just a long variant.
#[inline]
pub fn long<L: ToString>(l: L) -> FullTag {
    FullTag::from(Tag::Long(l.to_string()))
}

/// Create a tag with both short and long variants.
#[inline]
pub fn both<S: Into<char>, L: ToString>(s: S, l: L) -> FullTag {
    FullTag::from(Tag::Both(s.into(), l.to_string()))
}

/// Create an environment variable tag.
#[inline]
pub fn env<E: ToString>(e: E) -> FullTag {
    FullTag {
        cli: None,
        env: Some(e.to_string()),
    }
}

/// An argument name that may have either a CLI component,
/// environment variable component, or both.
#[derive(Debug, Clone)]
pub struct FullTag {
    pub(crate) cli: Option<Tag>,
    pub(crate) env: Option<String>,
}

impl FullTag {
    /// Add a CLI component.
    pub fn cli(mut self, tag: Tag) -> Self {
        self.cli = Some(tag);
        self
    }

    /// Add an environment variable component.
    pub fn env<S: ToString>(mut self, name: S) -> Self {
        self.env = Some(name.to_string());
        self
    }

    /// Returns whether or not this tag has a CLI component.
    pub fn has_cli(&self) -> bool {
        self.cli.is_some()
    }

    /// Returns whether or not this tag has an environment
    /// variable component.
    pub fn has_env(&self) -> bool {
        self.env.is_some()
    }

    pub fn matches(&self, tag: &str) -> bool {
        self.cli.as_ref().map_or(false, |t| t.matches(tag))
    }

    pub fn matches_long(&self, long: &str) -> bool {
        self.cli
            .as_ref()
            .map_or(false, |tag| tag.matches_long(long))
    }

    pub fn matches_short(&self, short: char) -> bool {
        self.cli
            .as_ref()
            .map_or(false, |tag| tag.matches_short(short))
    }
}

impl From<Tag> for FullTag {
    fn from(tag: Tag) -> Self {
        Self {
            cli: Some(tag),
            env: None,
        }
    }
}

impl Hash for FullTag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(tag) = &self.cli {
            core::mem::discriminant(tag).hash(state);
        }

        if let Some(arg) = &self.env {
            arg.hash(state);
        }
    }
}

/// An argument tag, or name. Easiest to create via
/// [`short`], [`long`], and [`both`].
///
/// `Short` means one dash and one character, e.g. `-h`.
/// `Long` means two dashes and any number of characters,
/// e.g. `--help`. `Both` means all of the above, e.g.
/// `-h` OR `--help`.
#[derive(Debug, Clone)]
pub enum Tag {
    Short(char),
    Long(String),
    Both(char, String),
}

impl Tag {
    /// Create a [`FullTag`] from a [`Tag`].
    pub fn env(self, arg: String) -> FullTag {
        FullTag {
            cli: Some(self),
            env: Some(arg),
        }
    }

    pub fn matches(&self, tag: &str) -> bool {
        if let Some(tag) = tag.strip_prefix("--") {
            self.matches_long(tag)
        } else if let Some(tag) = tag.strip_prefix('-') {
            if tag.len() == 1 {
                self.matches_short(tag.chars().next().unwrap())
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn matches_long(&self, long: &str) -> bool {
        match self {
            Tag::Short(_) => false,
            Tag::Long(l) | Tag::Both(_, l) => l == long,
        }
    }

    pub fn matches_short(&self, short: char) -> bool {
        match self {
            Tag::Long(_) => false,
            Tag::Short(s) | Tag::Both(s, _) => *s == short,
        }
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Short(s) => match other {
                Self::Short(o) => s == o,
                Self::Both(o, _) => s == o,
                _ => false,
            },
            Self::Long(s) => match other {
                Self::Long(o) => s == o,
                Self::Both(_, o) => s == o,
                _ => false,
            },
            Self::Both(s1, s2) => match other {
                Self::Short(o) => s1 == o,
                Self::Long(o) => s2 == o,
                Self::Both(o1, o2) => (s1 == o1) || (s2 == o2),
            },
        }
    }
}

impl Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
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
