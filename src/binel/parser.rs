use super::*;
use log::*;
use nom::branch::alt;
use nom::bytes::complete::*;
use nom::combinator::map;
use nom::multi::count;
use nom::number::complete::*;
use nom::sequence::preceded;
use nom::{error::ParseError, take_str, IResult};
use nom_varint::take_varint;
use std::prelude::v1::*;

/// Take a string with the length being a varint.
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
    let (buf, length) = match take_varint(buf) {
        Ok(res) => res,
        Err(nom::Err::Error((buf, kind))) => {
            return Err(nom::Err::Error(E::from_error_kind(buf, kind)))
        }
        Err(nom::Err::Failure((buf, kind))) => {
            return Err(nom::Err::Failure(E::from_error_kind(buf, kind)))
        }
        Err(nom::Err::Incomplete(needed)) => return Err(nom::Err::Incomplete(needed)),
    };
    let (buf, string) = take_str!(buf, length)?;
    Ok((buf, string.to_string()))
}

/// Lookup a u16 from a `&[u8]` in a string lookup table.
pub fn take_lookup<'a: 'b, 'b, E: 'b>(
    lookup: &'b [String],
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], &'b String, E> + 'b
where
    E: ParseError<&'a [u8]>,
{
    map(le_u16, move |index| &lookup[index as usize])
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

/// Parse a `BinEl` from a `&[u8]`. Tested solely in integration tests due to complexity.
pub fn take_element<'a: 'b, 'b, E>(
    lookup: &'b [String],
) -> impl Fn(&'a [u8]) -> IResult<&'a [u8], BinEl, E> + 'b
where
    E: ParseError<&'a [u8]>,
{
    move |buf| {
        debug!("taking element");
        let mut buf = buf;

        debug!("taking name");
        let (lookup_buf, name) = take_lookup(&lookup)(buf)?;
        buf = lookup_buf;
        debug!("name is {}", name);

        let mut binel = BinEl::new(&name);

        let (attr_count_buf, attr_count) = le_u8(buf)?;
        buf = attr_count_buf;

        for _ in 0..attr_count {
            debug!("taking key");
            let (key_buf, key) = take_lookup(&lookup)(buf)?;
            debug!("key is {}", key);
            debug!("taking value");
            let (val_buf, val) = take_elemattr(&lookup)(key_buf)?;
            buf = val_buf;
            binel.attributes.insert(key.clone(), val);
        }

        let (mut buf, child_count) = le_u16(buf)?;
        debug!("{} children", child_count);
        for _ in 0..child_count {
            let (child_buf, child) = take_element(lookup)(buf)?;
            buf = child_buf;
            binel.insert(child);
        }

        debug!("got element");

        Ok((buf, binel))
    }
}

/// Parse a `BinFile` from a `&[u8]`. Tested solely in integration tests due to complexity.
pub fn take_file<'a, E>(buf: &'a [u8]) -> IResult<&'a [u8], BinFile, E>
where
    E: ParseError<&'a [u8]>,
{
    let (buf, _) = tag(b"\x0bCELESTE MAP")(buf)?;
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
