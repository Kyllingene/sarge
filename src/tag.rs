use std::{fmt::Display, hash::Hash};

#[inline]
pub fn short<S: Into<char>>(s: S) -> Tag {
    Tag::Short(s.into())
}

#[inline]
pub fn long<L: ToString>(l: L) -> Tag {
    Tag::Long(l.to_string())
}

#[inline]
pub fn both<S: Into<char>, L: ToString>(s: S, l: L) -> Tag {
    Tag::Both(s.into(), l.to_string())
}

/// An argument tag, or name.
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
