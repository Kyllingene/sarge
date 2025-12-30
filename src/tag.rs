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
pub fn long<L: Into<String>>(l: L) -> Full {
    Full::from(Cli::Long(l.into()))
}

/// Create a tag with both short and long variants.
#[inline]
#[allow(clippy::needless_pass_by_value)]
pub fn both<S: Into<char>, L: Into<String>>(s: S, l: L) -> Full {
    Full::from(Cli::Both(s.into(), l.into()))
}

/// Create an environment variable argument.
#[inline]
#[allow(clippy::needless_pass_by_value)]
pub fn env<E: Into<String>>(e: E) -> Full {
    Full {
        cli: None,
        env: Some(e.into()),

        #[cfg(feature = "help")]
        doc: None,
        #[cfg(feature = "help")]
        default: None,
    }
}

/// An argument name that may have either a CLI component,
/// environment variable component, or both.
///
/// Create with [`short`], [`long`], [`both`], and [`env`](env()).
#[derive(Debug, Clone)]
pub struct Full {
    pub(crate) cli: Option<Cli>,
    pub(crate) env: Option<String>,

    /// The documentation for this argument.
    #[cfg(feature = "help")]
    pub doc: Option<String>,

    /// The default value for this argument, if known.
    #[cfg(feature = "help")]
    pub(crate) default: Option<String>,
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
    pub fn env<S: Into<String>>(mut self, name: S) -> Self {
        self.env = Some(name.into());
        self
    }

    /// Add documentation to the argument. If `doc.is_empty()`, instead
    /// removes any documentation.
    ///
    /// Only available on feature `help`.
    #[must_use]
    #[cfg(feature = "help")]
    pub fn doc<S: Into<String>>(mut self, doc: S) -> Self {
        let doc = doc.into();
        if doc.is_empty() {
            self.doc = None;
            self
        } else {
            self.doc = Some(doc);
            self
        }
    }

    /// Add a default value to the argument. If `default.is_empty()`, instead
    /// shows an empty default.
    ///
    /// Only available on feature `help`.
    #[must_use]
    #[cfg(feature = "help")]
    pub fn default_value<S: Into<String>>(mut self, default: S) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Remove the default value for the argument.
    ///
    /// Only available on feature `help`.
    #[must_use]
    #[cfg(feature = "help")]
    pub fn clear_default_value(mut self) -> Self {
        self.default = None;
        self
    }

    /// Returns whether or not this tag has a CLI component.
    pub fn has_cli(&self) -> bool {
        self.cli.is_some()
    }

    /// Returns whether or not this tag has an environment variable component.
    pub fn has_env(&self) -> bool {
        self.env.is_some()
    }

    /// Returns whether or not the CLI component matches the given tag.
    /// Automatically determines whether it's a short or long tag.
    pub fn matches_cli(&self, tag: &str) -> bool {
        self.cli.as_ref().is_some_and(|t| t.matches(tag))
    }

    /// Returns whether or not the CLI component matches the given long-form
    /// tag; assumes that the leading `--` has been stripped.
    pub fn matches_long(&self, long: &str) -> bool {
        self.cli.as_ref().is_some_and(|tag| tag.matches_long(long))
    }

    /// Returns whether or not the CLI component matches the given short-form
    /// tag; assumes that the leading `-` has been stripped.
    pub fn matches_short(&self, short: char) -> bool {
        self.cli
            .as_ref()
            .is_some_and(|tag| tag.matches_short(short))
    }

    /// Returns whether or not the environment variable component matches the
    /// given name.
    pub fn matches_env(&self, env: &str) -> bool {
        self.env.as_ref().is_some_and(|arg| arg == env)
    }
}

impl From<Cli> for Full {
    fn from(tag: Cli) -> Self {
        Self {
            cli: Some(tag),
            env: None,

            #[cfg(feature = "help")]
            doc: None,
            #[cfg(feature = "help")]
            default: None,
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
/// `-h` AND `--help`.
#[derive(Debug, Clone)]
pub enum Cli {
    /// A short-form tag, e.g. `-h`.
    Short(char),
    /// A long-form tag, e.g. `--help`.
    Long(String),
    /// Both a long- and short-form tag, e.g. `-h` AND `--help`.
    Both(char, String),
}

impl Cli {
    /// Create a [`Full`] from a [`Cli`].
    pub fn env(self, env: String) -> Full {
        Full {
            cli: Some(self),
            env: Some(env),

            #[cfg(feature = "help")]
            doc: None,
            #[cfg(feature = "help")]
            default: None,
        }
    }

    /// Returns whether or not the given tag matches. Automatically determines
    /// if it's a short or long tag.
    pub fn matches(&self, tag: &str) -> bool {
        if let Some(tag) = tag.strip_prefix("--") {
            self.matches_long(tag)
        } else if let Some(tag) = tag.strip_prefix('-') {
            if let Some(ch) = tag.chars().next() {
                self.matches_short(ch)
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Returns whether or not the given long-form tag matches. Assumes that
    /// the leading `--` has been stripped.
    pub fn matches_long(&self, long: &str) -> bool {
        match self {
            Cli::Short(_) => false,
            Cli::Long(l) | Cli::Both(_, l) => l == long,
        }
    }

    /// Returns whether or not the given short-form tag matches. Assumes that
    /// the leading `-` has been stripped.
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

impl Display for Cli {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Short(ch) => write!(f, "-{ch}"),
            Self::Long(s) => write!(f, "--{s}"),
            Self::Both(ch, s) => write!(f, "-{ch} / --{s}"),
        }
    }
}
