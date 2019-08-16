use celeste::{dialog::*, *};

use std::env;
use std::fs;

fn main() -> Result<'static, ()> {
    env_logger::init();

    let dialog_bytes = fs::read_to_string(env::args().nth(1).unwrap())?;
    let dialog_data = parser::parse_entries::<Error>(&dialog_bytes).unwrap().1;
    print!("{}", dialog_data);

    Ok(())
}
