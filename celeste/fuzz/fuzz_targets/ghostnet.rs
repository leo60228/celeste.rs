#![no_main]
use libfuzzer_sys::fuzz_target;

use celeste::ghostnet::*;

fuzz_target!(|data: &[u8]| {
    let _ = chunk::<()>(data);
});
