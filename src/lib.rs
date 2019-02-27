//! # `celeste`
//! `celeste` is a Rust crate for files used in the 2018 video game Celeste. It currently has a fully-functional writer and parser for BinaryElement files, which are used to store the game's levels.

#[macro_use]
extern crate nom;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate unsigned_varint;
extern crate byteorder;
extern crate itertools;

/// `binel` handles Celeste's `BinaryElement` format, which is used for map files.
pub mod binel;
