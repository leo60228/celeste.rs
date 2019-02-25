//! (nom doesn't currently support documenting parsers, so I'm putting them in the module
//! description)
//!
//! # `take_string`
//! Take a string with the length being a varint.
//!
//! ## Examples:
//! ```
//! use celeste::binel::parser::take_string;
//!
//! let header = b"\x0bCELESTE MAP";
//!
//! assert_eq!(take_string(&header[..]), Ok((&b""[..], "CELESTE MAP")));
//! ```
//!
//! # `take_file`
//! Parse a BinFile from a `&[u8]`. Tested solely in integration tests due to complexity.

extern crate nom;

use std::str;
use nom::rest;
use nom_varint::take_varint;
use super::BinFile;

named!(pub take_string<&[u8], &str>, do_parse!(
    length: take_varint >>
    string: take_str!(length) >>
    ( string )
));

named!(pub take_file<&[u8], BinFile>, do_parse!(
    tag!(b"\x0bCELESTE MAP") >>
    package: take_string >>
    remaining: rest >>
    ( BinFile { package, rest: remaining } )
));

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn take_header_length() {
        assert_eq!(take_varint(&b"\x0bCELESTE MAPdummy"[..]), Ok((&b"CELESTE MAPdummy"[..], 0x0b)));
    }

    #[test]
    fn take_header() {
        assert_eq!(take_string(&b"\x0bCELESTE MAPdummy"[..]), Ok((&b"dummy"[..], "CELESTE MAP")));
    }
}
