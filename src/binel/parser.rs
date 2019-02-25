extern crate nom;

use nom::Err::*;
use nom::Needed::Unknown;

/// Parses a variable-byte length from a map.
///
/// # Examples
///
/// ```
/// use celeste::binel::parser::take_length;
///
/// assert_eq!(take_length(&[0x0b]), Ok((&[] as &[u8], 0x0b)));
/// ```
pub fn take_length(i: &[u8]) -> nom::IResult<&[u8], usize> {
    let mut res: usize = 0;
    let mut count: usize = 0;
    let mut remainder = i;
    loop {
        let byte = match take!(remainder, 1) {
            Ok((rest, bytes)) => {remainder = rest; bytes[0]},
            Err(_) => return Err(Incomplete(Unknown)),
        };
        res += ((byte as usize) & 127) << (count * 7);
        count += 1;
        if (byte >> 7) == 0 {
            return Ok((remainder, res));
        }
    }
}

#[test]
fn parse_length_simple() {
    assert_eq!(take_length(&[0x0b, 0x01, 0x02, 0x03]), Ok((b"\x01\x02\x03" as &[u8], 11)));
}

#[test]
fn parse_length_twobyte() {
    assert_eq!(take_length(&[0x84, 0x02, 0x04, 0x05, 0x06]), Ok((b"\x04\x05\x06" as &[u8], 260)));
}
