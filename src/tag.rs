//! Everything surrounding [tags](`Full`).

use std::{fmt::Display, hash::Hash};

/// Create a tag with just a short variant.
#[inline]
pub fn short<S: Into<char>>(s: S) -> Full {
    Full::from(Cli::Short(s.into()))
}

/// Create a tag with just a long variant.
#[inline]
#[allow(clippy::needless_pass_by_value)]
pub fn long<L: ToString>(l: L) -> Full {
    Full::from(Cli::Long(l.to_string()))
}

/// Create a tag with both short and long variants.
#[inline]
#[allow(clippy::needless_pass_by_value)]
pub fn both<S: Into<char>, L: ToString>(s: S, l: L) -> Full {
    Full::from(Cli::Both(s.into(), l.to_string()))
}

/// Create an environment variable tag.
#[inline]
#[allow(clippy::needless_pass_by_value)]
pub fn env<E: ToString>(e: E) -> Full {
    Full {
        cli: None,
        env: Some(e.to_string()),
    }
}

/// An argument name that may have either a CLI component,
/// environment variable component, or both.
#[derive(Debug, Clone)]
pub struct Full {
    pub(crate) cli: Option<Cli>,
    pub(crate) env: Option<String>,
}

impl Full {
    /// Add a CLI component.
    #[must_use]
    pub fn cli(mut self, tag: Cli) -> Self {
        self.cli = Some(tag);
        self
    }

    /// Add an environment variable component.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
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

impl From<Cli> for Full {
    fn from(tag: Cli) -> Self {
        Self {
            cli: Some(tag),
            env: None,
        }
    }
}

impl Hash for Full {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(tag) = &self.cli {
            core::mem::discriminant(tag).hash(state);
        }

        if let Some(arg) = &self.env {
            arg.hash(state);
        }
    }
}

/// A CLI argument tag, or name. Easiest to create via
/// [`short`], [`long`], and [`both`].
///
/// `Short` means one dash and one character, e.g. `-h`.
/// `Long` means two dashes and any number of characters,
/// e.g. `--help`. `Both` means all of the above, e.g.
/// `-h` OR `--help`.
#[derive(Debug, Clone)]
pub enum Cli {
    Short(char),
    Long(String),
    Both(char, String),
}

impl Cli {
    /// Create a [`Full`] from a [`Cli`].
    pub fn env(self, arg: String) -> Full {
        Full {
            cli: Some(self),
            env: Some(arg),
        }
    }

    // The only panic is `unwrap`, which is checked here.
    #[allow(clippy::missing_panics_doc)]
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
            Cli::Short(_) => false,
            Cli::Long(l) | Cli::Both(_, l) => l == long,
        }
    }

    pub fn matches_short(&self, short: char) -> bool {
        match self {
            Cli::Long(_) => false,
            Cli::Short(s) | Cli::Both(s, _) => *s == short,
        }
    }
}

impl PartialEq for Cli {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Short(s) => match other {
                Self::Short(o) | Self::Both(o, _) => s == o,
                Self::Long(_) => false,
            },
            Self::Long(s) => match other {
                Self::Long(o) | Self::Both(_, o) => s == o,
                Self::Short(_) => false,
            },
            Self::Both(s1, s2) => match other {
                Self::Short(o) => s1 == o,
                Self::Long(o) => s2 == o,
                Self::Both(o1, o2) => (s1 == o1) || (s2 == o2),
            },
        }
    }
}

impl Hash for Cli {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Display for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Short(ch) => write!(f, "-{ch}"),
            Self::Long(s) => write!(f, "--{s}"),
            Self::Both(ch, s) => write!(f, "-{ch} / --{s}"),
        }
    }
}
