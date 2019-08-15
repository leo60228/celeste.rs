use super::{Dialog, DialogKey};
use std::fmt;
use std::prelude::v1::*;

impl fmt::Display for DialogKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let DialogKey(name, entry) = self;
        let level = entry.level - 1;

        for _ in 0..level {
            write!(f, "\t")?;
        }

        write!(f, "{}={}", name, entry.indented_str)?;

        Ok(())
    }
}

impl fmt::Display for Dialog<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for key in self {
            write!(f, "{}\n\n", key)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::dialog::*;

    #[test]
    fn key() {
        let key = DialogKey(
            "ABC",
            DialogEntry {
                indented_str: "123",
                level: 2,
            },
        );
        assert_eq!(&key.to_string(), "\tABC=123");
    }

    #[test]
    fn map() {
        let abc = DialogKey(
            "ABC",
            DialogEntry {
                indented_str: "123",
                level: 2,
            },
        );
        let def = DialogKey(
            "DEF",
            DialogEntry {
                indented_str: "456",
                level: 2,
            },
        );
        let mut dialog = Dialog::new();
        dialog.insert(abc);
        dialog.insert(def);
        let dialog_str = dialog.to_string();
        let mut split: Vec<_> = dialog_str.split("\n\n").collect();
        split.sort();
        assert_eq!(&split, &["", "\tABC=123", "\tDEF=456"]);
    }
}
