// set CELESTE_DIALOG_URL to Celeste's English.txt

use celeste::dialog::{Dialog, DialogKey, ParseExt};
use std::env;
use std::fmt::Write;

fn get_url(url: &str) -> String {
    let url = reqwest::Url::parse(url).unwrap();

    if url.scheme() == "file" {
        std::fs::read_to_string(url.to_file_path().unwrap()).unwrap()
    } else {
        reqwest::get(url).unwrap().text().unwrap()
    }
}

#[test]
#[ignore]
fn roundtrip() {
    let url = env::var("CELESTE_DIALOG_URL").unwrap();
    let dialog_txt = get_url(&url);
    let dialog: Dialog = dialog_txt.parse().unwrap();

    let new_dialog_txt = dialog.to_string();
    let new_dialog = new_dialog_txt.parse().unwrap();

    let mut succeeded = true;
    let mut error = "".to_string();

    for DialogKey(key, value) in dialog {
        match new_dialog.get(key) {
            Some(v2) if v2.unindent() == value.unindent() => {}
            Some(bad) => {
                succeeded = false;
                let bad = bad.unindent();
                let value = value.unindent();
                if bad.len() > 80 || value.len() > 80 {
                    writeln!(error, "long key {} incorrect", key).unwrap();
                } else {
                    writeln!(error, "key {}: {:?} != {:?}", key, bad, value).unwrap();
                }
            }
            None => {
                succeeded = false;
                writeln!(error, "key {} missing", key).unwrap();
            }
        }
    }

    if !succeeded {
        panic!("{}", error);
    }
}
