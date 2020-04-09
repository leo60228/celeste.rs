use crate::{Error, Result};
use derive_more::{From, Into};
use indexmap::map::{self, IndexMap};
use shrinkwraprs::Shrinkwrap;
use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::iter::{self, FromIterator};
use std::prelude::v1::*;

mod parser;
mod writer;

type DialogIndexMap<'a> =
    IndexMap<&'a str, DialogEntry<'a>, hashbrown::hash_map::DefaultHashBuilder>;

/// A dialog file.
#[derive(PartialEq, Eq, Debug, From, Into, Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Dialog<'a>(pub DialogIndexMap<'a>);

impl<'a> TryFrom<&'a str> for Dialog<'a> {
    type Error = Error<'a>;

    fn try_from(s: &'a str) -> Result<Self> {
        Ok(parser::parse(s)?)
    }
}

/// Parse a `Dialog` from a string. Implemented as an extension trait since
/// FromStr doesn't support lifetimes.
pub trait ParseExt<'a>: private::Parseable {
    /// Parse.
    fn parse(&'a self) -> Result<Dialog<'a>>;
}

impl<'a> ParseExt<'a> for str {
    fn parse(&'a self) -> Result<Dialog<'a>> {
        self.try_into()
    }
}

impl<'a> ParseExt<'a> for String {
    fn parse(&'a self) -> Result<Dialog<'a>> {
        (&*self as &str).try_into()
    }
}

mod private {
    pub trait Parseable {}

    impl Parseable for str {}
    impl Parseable for String {}
}

impl<'a> Extend<DialogKey<'a>> for Dialog<'a> {
    fn extend<I: IntoIterator<Item = DialogKey<'a>>>(&mut self, iter: I) {
        self.0
            .extend(iter.into_iter().map(<_ as Into<(_, _)>>::into))
    }
}

impl<'a> Extend<(&'a str, DialogEntry<'a>)> for Dialog<'a> {
    fn extend<I: IntoIterator<Item = (&'a str, DialogEntry<'a>)>>(&mut self, iter: I) {
        self.0.extend(iter.into_iter())
    }
}

impl<'a> FromIterator<DialogKey<'a>> for Dialog<'a> {
    fn from_iter<I: IntoIterator<Item = DialogKey<'a>>>(iter: I) -> Self {
        let mut map: DialogIndexMap = Default::default();
        map.extend(
            iter.into_iter()
                .map(<(&str, DialogEntry) as From<DialogKey>>::from),
        );
        Dialog(map)
    }
}

impl<'a> FromIterator<(&'a str, DialogEntry<'a>)> for Dialog<'a> {
    fn from_iter<I: IntoIterator<Item = (&'a str, DialogEntry<'a>)>>(iter: I) -> Self {
        let mut map: DialogIndexMap = Default::default();
        map.extend(iter.into_iter());
        Dialog(map)
    }
}

type KeyTuple<'a> = (&'a str, DialogEntry<'a>);
type KeyRefTuple<'a, 'b> = (&'b &'a str, &'b DialogEntry<'a>);
type TupleToKey<'a> = fn(KeyTuple<'a>) -> DialogKey<'a>;
type RefTupleToKey<'a, 'b> = fn(KeyRefTuple<'a, 'b>) -> DialogKey<'a>;

type DialogIntoIter<'a> = map::IntoIter<&'a str, DialogEntry<'a>>;
type DialogIter<'a, 'b> = map::Iter<'b, &'a str, DialogEntry<'a>>;

/// Returned by `Dialog::into_iter`.
pub struct IntoIter<'a>(iter::Map<DialogIntoIter<'a>, TupleToKey<'a>>);

impl<'a> Iterator for IntoIter<'a> {
    type Item = DialogKey<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Returned by `Dialog::iter`.
pub struct Iter<'a, 'b>(iter::Map<DialogIter<'a, 'b>, RefTupleToKey<'a, 'b>>);

impl<'a> Iterator for Iter<'a, '_> {
    type Item = DialogKey<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> IntoIterator for Dialog<'a> {
    type Item = DialogKey<'a>;
    type IntoIter = IntoIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter().map(From::from))
    }
}

impl<'a, 'b> IntoIterator for &'b Dialog<'a> {
    type Item = DialogKey<'a>;
    type IntoIter = Iter<'a, 'b>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self.0.iter().map(From::from))
    }
}

impl<'a> Dialog<'a> {
    /// Create an empty `Dialog`.
    pub fn new() -> Self {
        Dialog(Default::default())
    }

    /// Insert a new key into the struct.
    pub fn insert<'b: 'a>(&mut self, key: DialogKey<'b>) -> Option<DialogKey<'a>> {
        let DialogKey(name, entry) = key;
        self.0.insert(name, entry).map(|e| DialogKey(key.0, e))
    }

    /// Iterate over each key.
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}

impl Default for Dialog<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// A dialog key, with a name and contents.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, From, Into)]
pub struct DialogKey<'a>(pub &'a str, pub DialogEntry<'a>);

impl<'a, 'b> From<KeyRefTuple<'a, 'b>> for DialogKey<'a> {
    fn from((&name, &entry): KeyRefTuple<'a, 'b>) -> Self {
        DialogKey(name, entry)
    }
}

/// A dialog entry, containing the raw string and the indentation level.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
pub struct DialogEntry<'a> {
    /// The indented string of dialog.
    pub indented_str: &'a str,
    /// The indentation level of the entry.
    pub level: usize,
}

impl DialogEntry<'_> {
    /// Remove the indentation from the DialogEntry. Returns a subslice if
    /// possible, and returns a freshly allocated `String` otherwise.
    pub fn unindent(&self) -> Cow<str> {
        let mut counter = 0;
        let trim = move |c| {
            if counter < self.level && (c == '\t' || c == ' ') {
                counter += 1;
                true
            } else {
                false
            }
        };

        if self.level == 0 || self.indented_str.trim_start().lines().count() <= 1 {
            match self.indented_str.chars().next() {
                Some('\r') => &self.indented_str[2..],
                Some('\n') => &self.indented_str[1..],
                _ => self.indented_str,
            }
            .trim_start_matches(trim)
            .trim_end()
            .into()
        } else {
            let mut string = String::with_capacity(self.indented_str.len());

            for line in self
                .indented_str
                .trim_start()
                .lines()
                .map(|s| s.trim_start_matches(trim))
            {
                string.push_str(line);
                string.push('\n');
            }
            string.truncate(string.trim_end().len());
            string.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unindented_entry() {
        let string = "ABC=\r\n123\r\n456";
        let slice = &string[4..];
        assert_eq!(slice, "\r\n123\r\n456", "sanity check");
        let entry = DialogEntry {
            indented_str: slice,
            level: 0,
        };
        assert_eq!(entry.unindent(), Cow::Borrowed("123\r\n456"));
    }
    #[test]
    fn short_entry() {
        let string = "ABC=\t123";
        let slice = &string[4..];
        assert_eq!(slice, "\t123", "sanity check");
        let entry = DialogEntry {
            indented_str: slice,
            level: 1,
        };
        assert_eq!(entry.unindent(), Cow::Borrowed("123"));
    }
    #[test]
    fn long_entry() {
        let string = "ABC=\n\t123\n\t456";
        let slice = &string[4..];
        assert_eq!(slice, "\n\t123\n\t456", "sanity check");
        let entry = DialogEntry {
            indented_str: slice,
            level: 1,
        };
        assert_eq!(entry.unindent(), Cow::Borrowed("123\n456").into_owned());
    }
}
