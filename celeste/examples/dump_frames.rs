use celeste::{ghostnet::*, *};
use nom::combinator::iterator;
use nom::error::VerboseError;
use std::{env, fs};

fn main() -> Result<(), Error<'static>> {
    let dump = fs::read(env::args().nth(1).unwrap())?;
    let mut iter = iterator::<_, _, VerboseError<_>, _>(&dump as &[u8], frame);
    for frame in &mut iter {
        println!("{:#?}", frame);
    }

    println!("{:?}", iter.finish());

    Ok(())
}
