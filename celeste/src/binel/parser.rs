use super::*;
use nom::branch::alt;
use nom::bytes::complete::*;
use nom::combinator::{map, map_opt, map_res};
use nom::multi::{count, length_data};
use nom::number::complete::*;
use nom::sequence::preceded;
use nom::{error::ParseError, IResult};
use nom_varint::take_varint;
use std::str::from_utf8;

/// Take a borrowed string with the length being a varint.
///
/// # Examples:
/// ```
/// use celeste::binel::parser::take_str;
/// use celeste::Error;
///
/// let header = b"\x0bCELESTE MAP";
///
/// assert_eq!(take_str::<Error>(&header[..]).unwrap(), (&b""[..], "CELESTE MAP"));
/// ```
pub fn take_str<'a, E>(buf: &'a [u8]) -> IResult<&'a [u8], &'a str, E>
where
    E: ParseError<&'a [u8]>,
{
    map_res(length_data(take_varint), from_utf8)(buf)
}

/// Take an owned string with the length being a varint.
///
/// # Examples:
/// ```
/// use celeste::binel::parser::take_string;
/// use celeste::Error;
///
/// let header = b"\x0bCELESTE MAP";
///
/// assert_eq!(take_string::<Error>(&header[..]).unwrap(), (&b""[..], "CELESTE MAP".to_string()));
/// ```
pub fn take_string<'a, E>(buf: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>,
{
    map(take_str, String::from)(buf)
}

/// Lookup a u16 from a `&[u8]` in a string lookup table.
pub fn take_lookup<'a: 'b, 'b, E: 'b>(
    lookup: &'b [String],
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'b String, E> + 'b
where
    E: ParseError<&'a [u8]>,
{
    map_opt(le_u16, move |index| lookup.get(index as usize))
}

/// Take a single character from a Celeste RLE-encoded string in a `&[u8]`.
pub fn take_rle_char<'a, E>(buf: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>,
{
    let (buf, times) = le_u8(buf)?;
    let (buf, byte) = le_u8(buf)?;
    Ok((buf, (byte as char).to_string().repeat(times as usize)))
}

/// Take a Celeste RLE-encoded string from a `&[u8]`
pub fn take_rle_string<'a, E>(buf: &'a [u8]) -> IResult<&'a [u8], String, E>
where
    E: ParseError<&'a [u8]>,
{
    let (buf, len) = le_i16(buf)?;
    let (buf, chars) = count(take_rle_char, (len / 2) as usize)(buf)?;
    Ok((buf, chars.concat()))
}

/// Parse a `BinElAttr` from a `&[u8]`.
///
/// # Examples:
/// ```
/// use celeste::binel::*;
/// use celeste::Error;
///
/// assert_eq!(parser::take_elemattr::<Error>(&[])(b"\x01\x05").unwrap(), ((&b""[..], BinElAttr::Int(5))));
/// ```
#[allow(clippy::unknown_clippy_lints)]
#[allow(renamed_and_removed_lints)]
#[allow(clippy::cognitive_complexity)]
#[allow(clippy::cyclomatic_complexity)]
pub fn take_elemattr<'a: 'b, 'b, E: 'b>(
    lookup: &'b [String],
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], BinElAttr, E> + 'b
where
    E: ParseError<&'a [u8]>,
{
    alt((
        preceded(
            tag(b"\x00"),
            map(le_u8, |byte: u8| BinElAttr::Bool(byte != 0)),
        ),
        preceded(
            tag(b"\x01"),
            map(le_u8, |byte: u8| BinElAttr::Int(i32::from(byte))),
        ),
        preceded(
            tag(b"\x02"),
            map(le_i16, |word: i16| BinElAttr::Int(i32::from(word))),
        ),
        preceded(tag(b"\x03"), map(le_i32, BinElAttr::Int)),
        preceded(tag(b"\x04"), map(le_f32, BinElAttr::Float)),
        preceded(
            tag(b"\x05"),
            map(take_lookup(lookup), |string: &String| {
                BinElAttr::Text(string.clone())
            }),
        ),
        preceded(tag(b"\x06"), map(take_string, BinElAttr::Text)),
        preceded(tag(b"\x07"), map(take_rle_string, BinElAttr::Text)),
    ))
}

/// Parse a `BinEl` from a `&[u8]`. Tested solely in integration tests due to
/// complexity.
pub fn take_element<'a: 'b, 'b, E>(
    lookup: &'b [String],
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], BinEl, E> + 'b
where
    E: ParseError<&'a [u8]>,
{
    move |buf| {
        let mut buf = buf;

        let (lookup_buf, name) = take_lookup(&lookup)(buf)?;
        buf = lookup_buf;

        let mut binel = BinEl::new(&name);

        let (attr_count_buf, attr_count) = le_u8(buf)?;
        buf = attr_count_buf;

        for _ in 0..attr_count {
            let (key_buf, key) = take_lookup(&lookup)(buf)?;
            let (val_buf, val) = take_elemattr(&lookup)(key_buf)?;
            buf = val_buf;
            binel.attributes.insert(key.clone(), val);
        }

        let (mut buf, child_count) = le_u16(buf)?;
        for _ in 0..child_count {
            let (child_buf, child) = take_element(lookup)(buf)?;
            buf = child_buf;
            binel.insert(child);
        }

        Ok((buf, binel))
    }
}

/// Parse a `BinFile` from a `&[u8]`. Tested solely in integration tests due to
/// complexity.
pub fn take_file<'a, E>(buf: &'a [u8]) -> IResult<&'a [u8], BinFile, E>
where
    E: ParseError<&'a [u8]>,
{
    #[cfg(not(fuzzing))]
    let (buf, _) = tag(b"\x0bCELESTE MAP")(buf)?;
    #[cfg(fuzzing)]
    let (buf, _) = take_string(buf)?;
    let (buf, package) = take_string(buf)?;
    let (buf, length) = le_i16(buf)?;
    let (buf, lookup) = count(take_string, length as usize)(buf)?;
    let (buf, root) = take_element(&lookup)(buf)?;
    Ok((buf, BinFile { package, root }))
}

#[cfg(test)]
mod test {
    use super::*;
    use celeste::Error;

    #[test]
    fn take_header() {
        assert_eq!(
            take_string::<Error>(&b"\x0bCELESTE MAPdummy"[..]).unwrap(),
            (&b"dummy"[..], "CELESTE MAP".to_string())
        );
    }
}
