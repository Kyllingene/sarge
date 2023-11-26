use crate::ArgumentParser;

/// An iterator over the remainder of an argument call.
///
/// `'a` is the lifetime of the slice of strings, while
/// `'b` is the lifetime of the strings themselves.
pub struct RemainderIter<'a, 'b> {
    args: &'a [&'b str],
    i: usize,
}

impl<'a, 'b> From<&'a [&'b str]> for RemainderIter<'a, 'b> {
    fn from(args: &'a [&'b str]) -> Self {
        Self {
            args,
            i: 0,
        }
    }
}

impl<'a, 'b> Iterator for RemainderIter<'a, 'b> {
    type Item = &'b str;

    fn next(&mut self) -> Option<Self::Item> {
        let r = *self.args.get(self.i)?;
        self.i += 1;

        if r.starts_with('-') && r != "-" && r != "--" {
            None
        } else {
            Some(r)
        }
    }
}

