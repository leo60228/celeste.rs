#![no_main]
use libfuzzer_sys::fuzz_target;

extern crate celeste;

use celeste::{
    binel::{serialize::*, *},
    *,
};

fn try_parse(map_bytes: &[u8]) -> Result<(), Error> {
    let map_bin = parser::take_file(map_bytes)?.1;
    maps::Map::from_binel(BinElValue::Element(map_bin.root))?;
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let _ = try_parse(data);
});
