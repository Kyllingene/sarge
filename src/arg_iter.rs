/// An iterator over any remaining arguments.
///
/// TODO: skip consumed arguments (e.g. does `-f bar` return `bar` or nothing?)
pub struct ArgumentIter<'arg, A: AsRef<str>> {
    args: &'arg [A],
    i: usize,
}

impl<'arg, A: AsRef<str>> ArgumentIter<'arg, A> {
    /// Create a new iterator over the given arguments.
    pub(crate) fn new(args: &'arg [A]) -> Self {
        Self { args, i: 0 }
    }
}

impl<'arg, A: AsRef<str>> Iterator for ArgumentIter<'arg, A> {
    type Item = &'arg str;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i >= self.args.len() {
                break None;
            }

            let arg = self.args[self.i].as_ref();
            if arg == "-" || arg == "--" || !arg.starts_with('-') {
                self.i += 1;
                break Some(arg);
            }

            self.i += 1;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.args.len() - self.i))
    }
}
