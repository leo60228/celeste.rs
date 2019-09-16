//! # `celeste`
//! `celeste` is a Rust crate for files used in the 2018 video game Celeste. It
//! currently has a fully-functional writer and parser for `BinaryElement`
//! files, which are used to store the game's levels.

#![recursion_limit = "1024"]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate no_std_compat as std;

extern crate self as celeste; // necessary for celeste_derive to work

/// `binel` handles Celeste's `BinaryElement` format, which is used for map
/// files.
pub mod binel;

/// `maps` provides structs for maps parsed using `binel`.
#[cfg(feature = "derive")]
pub mod maps;

/// `dialog` handles Celeste's dialog files.
pub mod dialog;

pub mod ghostnet;

mod error;
pub use error::*;
