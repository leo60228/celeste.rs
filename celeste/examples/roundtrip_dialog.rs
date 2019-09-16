use celeste::{dialog::*, *};

use std::env;
use std::fs;

fn main() -> Result<(), Error<'static>> {
    env_logger::init();

    let dialog_str = fs::read_to_string(env::args().nth(1).unwrap())?;
    let dialog_data: Dialog = dialog_str.parse().unwrap();
    print!("{}", dialog_data);

    Ok(())
}
