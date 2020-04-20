use celeste::{ghostnet::*, *};
use nom::combinator::{iterator, recognize};
use nom::error::VerboseError;
use std::{env, fs};

fn main() -> Result<(), Error<'static>> {
    let path = env::args().nth(1).unwrap();
    let dump = fs::read(&path)?;
    let mut iter = iterator::<_, _, VerboseError<_>, _>(&dump as &[u8], recognize(frame));
    for (i, frame) in (&mut iter).into_iter().enumerate() {
        fs::write(format!("{}.{}", path, i), frame)?;
    }

    println!("{:?}", iter.finish());

    Ok(())
}
