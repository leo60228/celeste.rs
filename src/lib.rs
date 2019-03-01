//! # `celeste`
//! `celeste` is a Rust crate for files used in the 2018 video game Celeste. It currently has a fully-functional writer and parser for `BinaryElement` files, which are used to store the game's levels.

extern crate self as celeste; // necessary for celeste_derive to work

/// `binel` handles Celeste's `BinaryElement` format, which is used for map files.
pub mod binel;
