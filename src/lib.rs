//! # `celeste`
//! `celeste` is a Rust crate for parsing files used in the 2018 video game Celeste.

#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;
extern crate unsigned_varint;
#[macro_use]
extern crate log;
extern crate byteorder;
extern crate itertools;

/// `binel` handles parsing Celeste's `BinaryElement` format, which is used for map files.
pub mod binel;
