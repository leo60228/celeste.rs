use std::prelude::v1::*;

use super::{Dialog, DialogEntry, DialogKey};
use crate::Result;

use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "dialog/dialog.pest"]
struct DialogParser;

pub fn parse(input: &str) -> Result<Dialog> {
    let mut map = Dialog::new();
    let file = DialogParser::parse(Rule::file, &input)
        .unwrap_or_else(|e| panic!("{}", e))
        .next()
        .expect("file always produces output upon a successful parse");
    for rule in file.into_inner() {
        match rule.as_rule() {
            Rule::comment => {}
            Rule::entry => {
                let mut inner = rule.into_inner();
                let level = inner
                    .next()
                    .expect("entry always has 3 inner rules")
                    .as_str()
                    .len()
                    + 1;
                let key = inner
                    .next()
                    .expect("entry always has 3 inner rules")
                    .as_str();
                let indented_str = inner
                    .next()
                    .expect("entry always has 3 inner rules")
                    .as_str();
                let entry = DialogEntry {
                    level,
                    indented_str,
                };
                map.insert(DialogKey(key, entry));
            }
            _ => unreachable!(),
        }
    }
    Ok(map)
}
