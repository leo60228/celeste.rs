use std::prelude::v1::*;

use super::{Dialog, DialogEntry, DialogKey};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{line_ending, not_line_ending};
use nom::combinator::{flat_map, iterator, map, opt};
use nom::error::ParseError;
use nom::multi::fold_many0;
use nom::sequence::{pair, separated_pair};
use nom::IResult;

pub fn entry_name<'a, E>(inp: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: ParseError<&'a str>,
{
    take_till(|c| c == '=')(inp)
}

pub fn entry_text<'a, E>(level: usize) -> impl Fn(&'a str) -> IResult<&'a str, DialogEntry<'a>, E>
where
    E: ParseError<&'a str>,
{
    move |inp| {
        let inp = inp.trim_start_matches(&[' ', '\t'] as &[char]);
        let mut first;
        let mut taken = 0;
        let mut rem = inp;
        'outer: loop {
            first = rem.chars().nth(0);
            let mut esc = false;
            while first != Some('\r') && first != Some('\n') {
                rem = match first {
                    Some('#') if !esc => break 'outer,
                    Some(c) => {
                        esc = c == '{' || c == '\\';
                        taken += c.len_utf8();
                        &inp[taken..]
                    }
                    None => break 'outer,
                };
                first = rem.chars().nth(0);
            }
            while first == Some('\r') || first == Some('\n') {
                taken += match first {
                    Some('\r') => 2,
                    Some('\n') => 1,
                    _ => 0,
                };
                rem = &inp[taken..];
                first = rem.chars().nth(0);
            }
            if rem.starts_with('#') {
                break 'outer;
            }
            let backtrack = taken;
            for _ in 0..level {
                first = rem.chars().nth(0);
                rem = match first {
                    Some(c) if c == ' ' || c == '\t' => {
                        taken += c.len_utf8();
                        &inp[taken..]
                    }
                    _ => {
                        taken = backtrack;
                        break 'outer;
                    }
                };
            }
        }
        rem = &inp[taken..];
        Ok((
            rem,
            DialogEntry {
                indented_str: &inp[..taken],
                level,
            },
        ))
    }
}

pub fn parse_entry<'a, E>(
    level: usize,
) -> impl Fn(&'a str) -> IResult<&'a str, Option<DialogKey<'a>>, E>
where
    E: ParseError<&'a str>,
{
    move |inp| {
        if !inp.starts_with('#') {
            map(
                separated_pair(entry_name, tag("="), entry_text(level + 1)),
                |pair| Some(pair.into()),
            )(inp)
        } else {
            map(pair(not_line_ending, opt(line_ending)), |_| None)(inp)
        }
    }
}

pub fn parse_entries<'a, E>(inp: &'a str) -> IResult<&'a str, Dialog<'a>, E>
where
    E: ParseError<&'a str> + Clone,
{
    let mut iter = iterator(
        inp,
        flat_map(
            fold_many0(
                alt((tag(" "), tag("\t"), line_ending)),
                0,
                |acc: usize, item| match item {
                    "\n" => 0,
                    " " | "\t" => acc + 1,
                    _ => acc,
                },
            ),
            parse_entry,
        ),
    );
    let dialog = iter.flatten().collect();
    Ok((iter.finish()?.0, dialog))
}

#[cfg(test)]
mod tests {
    use crate::dialog::*;
    use indexmap::map::IndexMap;
    use nom::error::VerboseError;

    #[test]
    fn dialog_entry() {
        let (rem, opt) =
            super::parse_entry::<VerboseError<_>>(1)("ABC=\n\t\t123\n\n\t\t456\nDEF=").unwrap();
        let DialogKey(name, entry) = opt.unwrap();
        assert_eq!(rem, "DEF=");
        assert_eq!(name, "ABC");
        assert_eq!(entry.unindent(), "123\n\n456");
    }

    #[test]
    fn short_dialog_entry() {
        let (rem, opt) = super::parse_entry::<VerboseError<_>>(1)("ABC=123\n\tDEF=").unwrap();
        let DialogKey(name, entry) = opt.unwrap();
        assert_eq!(rem, "\tDEF=");
        assert_eq!(name, "ABC");
        assert_eq!(entry.unindent(), "123");
    }

    #[test]
    fn comment() {
        let (rem, opt) = super::parse_entry::<VerboseError<_>>(1)("# asdfasdf = 123").unwrap();
        assert_eq!(opt, None);
        assert_eq!(rem, "");
    }

    #[test]
    fn dialog_entries() {
        let mut map = IndexMap::new();
        map.insert(
            "ABC",
            DialogEntry {
                indented_str: "\n\t123\n",
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
            super::parse_entries::<VerboseError::<_>>("ABC=\n\t123\nDEF=456").unwrap(),
            ("", map.into())
        );
    }

    #[test]
    fn indented_dialog_entries() {
        let mut map = IndexMap::new();
        map.insert(
            "ABC",
            DialogEntry {
                indented_str: "\n\t\t1\\#23\n\n",
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
        let output =
            super::parse_entries::<VerboseError<_>>("\tABC=\n\t\t1\\#23\n\n\tDEF=\t456").unwrap();
        assert_eq!(output, ("", map.into()));
        assert_eq!(output.1["ABC"].unindent(), "1\\#23");
    }

    #[rustfmt::skip]
    #[test]
    fn theo_bug() {
        let input = concat!(
            "\t",
            r#"CH6_THEO_ASK_DEFENSE=
  [THEO left wtf]
  So you want to destroy this {+PART_OF_YOU}?
 CH6_THEO_SAY_DEFENSE=
  [THEO left wtf]
  So you want to destroy this {+PART_OF_YOU}?"#
        );
        let correct_a = r#"
  [THEO left wtf]
  So you want to destroy this {+PART_OF_YOU}?"#;
        let correct_b = concat!(
            r#"
  [THEO left wtf]
  So you want to destroy this {+PART_OF_YOU}?"#,
            "\n"
        );
        let mut map = IndexMap::new();
        map.insert(
            "CH6_THEO_ASK_DEFENSE",
            DialogEntry {
                indented_str: correct_b,
                level: 2,
            },
        );
        map.insert(
            "CH6_THEO_SAY_DEFENSE",
            DialogEntry {
                indented_str: correct_a,
                level: 2,
            },
        );
        assert_eq!(
            super::parse_entries::<VerboseError::<_>>(input).unwrap(),
            ("", map.into())
        );
    }
}
