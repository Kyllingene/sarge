#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArgFmt {
    /// The width of the string, in characters.
    pub(crate) width: usize,
    /// The number of lines used, in characters.
    pub(crate) height: usize,
    /// The width of the long variant, if any. Doesn't include leading dashes.
    pub(crate) long_width: Option<usize>,
    /// Whether or not the argument has a short form.
    pub(crate) short: bool,
}

impl ArgFmt {
    pub fn new(short: bool) -> Self {
        Self {
            short,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doc {
    body: String,
    val: Option<String>,
}

impl Doc {
    /// Create a new `Doc` with a body.
    ///
    /// To add a value, see [`Doc::val`].
    pub fn new<S: ToString>(body: S) -> Self {
        Self {
            body: body.to_string(),
            val: None,
        }
    }

    /// Update the body of this documentation.
    #[must_use]
    pub fn body<S: ToString>(mut self, body: S) -> Self {
        self.body = body.to_string();
        self
    }

    /// Update the name of the value associated with this argument.
    #[must_use]
    pub fn val<S: ToString>(mut self, val: S) -> Self {
        self.val = Some(val.to_string());
        self
    }

    /// Delete the value associated with this argument, if any.
    pub fn no_val(&mut self) -> Option<String> {
        self.val.take()
    }
    
    /// Formats the documentation according to the given parameters.
    ///
    /// - `width`: What column the string will be placed at.
    pub(crate) fn format(&self, _width: usize) -> String {
        let mut s = String::new();

        // TODO: wrap long (> 80 char) lines

        if let Some(val) = &self.val {
            s.push_str(val);
        }

        s.push_str(&self.body);

        s
    }
}

