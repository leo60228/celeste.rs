//! # `celeste`
//! `celeste` is a Rust crate for parsing files used in the 2018 video game Celeste.

#[macro_use]
extern crate nom;
extern crate unsigned_varint;

/// `binel` handles parsing Celeste's BinaryElement format, which is used for map files.
pub mod binel;
