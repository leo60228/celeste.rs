use super::{Dialog, DialogEntry, DialogKey};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::line_ending;
use nom::combinator::{flat_map, iterator, map};
use nom::multi::fold_many0;
use nom::sequence::separated_pair;
use nom::IResult;

pub fn entry_name<'a>(inp: &'a str) -> IResult<&'a str, &'a str> {
    take_till(|c| c == '=')(inp)
}

pub fn entry_text<'a>(level: usize) -> impl Fn(&'a str) -> IResult<&'a str, DialogEntry<'a>> {
    move |inp| {
        let inp = inp.trim_start_matches(&[' ', '\t'] as &[char]);
        let mut first = inp.chars().nth(0);
        let mut taken = 0;
        let mut rem = inp;
        'outer: loop {
            while first != Some('\r') && first != Some('\n') {
                rem = match first {
                    Some(c) => {
                        taken += c.len_utf8();
                        &inp[taken..]
                    }
                    None => break 'outer,
                };
                first = rem.chars().nth(0);
            }
            taken += match first {
                Some('\r') => 2,
                Some('\n') => 1,
                _ => 0,
            };
            rem = &inp[taken..];
            for _ in 0..level {
                first = rem.chars().nth(0);
                rem = match first {
                    Some(c) if c == ' ' || c == '\t' => {
                        taken += c.len_utf8();
                        &inp[taken..]
                    }
                    Some(_) => break 'outer,
                    None => break 'outer,
                };
            }
            rem = &inp[..taken];
        }
        let last = inp.bytes().nth(taken - 1);
        if last == Some(b'\r') || last == Some(b'\n') {
            taken -= 1;
        }
        Ok((
            rem,
            DialogEntry {
                indented_str: &inp[..taken],
                level,
            },
        ))
    }
}

pub fn parse_entry<'a>(level: usize) -> impl Fn(&'a str) -> IResult<&'a str, DialogKey<'a>> {
    map(
        separated_pair(entry_name, tag("="), entry_text(level + 1)),
        Into::into,
    )
}

pub fn parse_entries<'a>(inp: &'a str) -> IResult<&'a str, Dialog<'a>> {
    let mut iter = iterator(
        inp,
        flat_map(
            fold_many0(
                alt((tag(" "), tag("\t"), line_ending)),
                0,
                |acc: usize, item| {
                    if item == " " || item == "\t" {
                        acc + 1
                    } else {
                        acc
                    }
                },
            ),
            parse_entry,
        ),
    );
    let dialog = iter.collect();
    Ok((iter.finish()?.0, dialog))
}

#[cfg(test)]
mod tests {
    use crate::dialog::*;
    use std::collections::HashMap;

    #[test]
    fn dialog_entry() {
        let (rem, DialogKey(name, entry)) =
            super::parse_entry(1)("ABC=\n\t\t123\n\n\t\tDEF=").unwrap();
        assert_eq!(rem, "\n\t\tDEF=");
        assert_eq!(name, "ABC");
        assert_eq!(entry.unindent(), "123");
    }

    #[test]
    fn short_dialog_entry() {
        let (rem, DialogKey(name, entry)) = super::parse_entry(1)("ABC=123\n\tDEF=").unwrap();
        assert_eq!(rem, "DEF=");
        assert_eq!(name, "ABC");
        assert_eq!(entry.unindent(), "123");
    }

    #[test]
    fn dialog_entries() {
        let mut map = HashMap::new();
        map.insert(
            "ABC",
            DialogEntry {
                indented_str: "\n\t123",
                level: 1,
            },
        );
        map.insert(
            "DEF",
            DialogEntry {
                indented_str: "456",
                level: 1,
            },
        );
        assert_eq!(
            super::parse_entries("ABC=\n\t123\nDEF=456").unwrap(),
            ("", map.into())
        );
    }

    #[test]
    fn indented_dialog_entries() {
        let mut map = HashMap::new();
        map.insert(
            "ABC",
            DialogEntry {
                indented_str: "\n\t\t123",
                level: 2,
            },
        );
        map.insert(
            "DEF",
            DialogEntry {
                indented_str: "456",
                level: 2,
            },
        );
        assert_eq!(
            super::parse_entries("\tABC=\n\t\t123\n\n\tDEF=\t456").unwrap(),
            ("", map.into())
        );
    }
}
