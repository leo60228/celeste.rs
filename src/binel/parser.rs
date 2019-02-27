use nom::{le_i16, le_i32, le_f32, le_u8, le_u16, IResult};
use nom_varint::take_varint;
use super::*;

/// Take a string with the length being a varint.
///
/// # Examples:
/// ```
/// use celeste::binel::parser::take_string;
///
/// let header = b"\x0bCELESTE MAP";
///
/// assert_eq!(take_string(&header[..]), Ok((&b""[..], "CELESTE MAP".to_string())));
/// ```
pub fn take_string(input: &[u8]) -> IResult<&[u8], String> {
    do_parse!(input,
        length: take_varint >>
        string: take_str!(length) >>
        ( string.to_string() )
    )
}

/// Lookup a u16 from a `&[u8]` in a string lookup table.
pub fn take_lookup<'a, 'b>(buf: &'a [u8], lookup: &'b[String]) -> IResult<&'a [u8], &'b String> {
    let (buf, index) = le_u16(buf)?;
    Ok((buf, &lookup[index as usize]))
}

/// Take a single character from a Celeste RLE-encoded string in a `&[u8]`.
pub fn take_rle_char(buf: &[u8]) -> IResult<&[u8], String> {
    do_parse!(buf, 
        times: le_u8 >>
        byte: le_u8 >>
        ( (byte as char).to_string().repeat(times as usize) )
    )
}

/// Take a Celeste RLE-encoded string from a `&[u8]`
pub fn take_rle_string(buf: &[u8]) -> IResult<&[u8], String> {
    do_parse!(buf,
        len: le_i16 >>
        chars: count!(take_rle_char, (len / 2) as usize) >>
        ( chars.concat() )
    )
}

/// Parse a `BinElAttr` from a `&[u8]`.
/// 
/// # Examples:
/// ```
/// use celeste::binel::*;
///
/// assert_eq!(parser::take_elemattr(b"\x01\x05", &[]), Ok((&b""[..], BinElAttr::Int(5))));
/// ```
#[allow(clippy::cyclomatic_complexity)]
pub fn take_elemattr<'a>(buf: &'a [u8], lookup: &[String]) -> IResult<&'a [u8], BinElAttr> {
    debug!("taking attribute (type {})", buf[0]);
    do_parse!(buf, 
        val: alt!(
            do_parse!(tag!(b"\x00") >> byte: le_u8 >> ( BinElAttr::Bool(byte != 0) )) |
            do_parse!(tag!(b"\x01") >> byte: le_u8 >> ( BinElAttr::Int(i32::from(byte)) )) |
            do_parse!(tag!(b"\x02") >> word: le_i16 >> ( BinElAttr::Int(i32::from(word)) )) |
            do_parse!(tag!(b"\x03") >> dword: le_i32 >> ( BinElAttr::Int(dword) )) |
            do_parse!(tag!(b"\x04") >> float: le_f32 >> ( BinElAttr::Float(float) )) |
            do_parse!(tag!(b"\x05") >> string: apply!(take_lookup, lookup) >> ( BinElAttr::Text(string.to_string()) )) |
            do_parse!(tag!(b"\x06") >> string: take_string >> ( BinElAttr::Text(string.to_string()) )) |
            do_parse!(tag!(b"\x07") >> string: take_rle_string >> ( BinElAttr::Text(string.to_string()) ))
        ) >>
        ( {debug!("{:?}", val); val} )
    )
}

/// Parse a `BinEl` from a `&[u8]`. Tested solely in integration tests due to complexity.
pub fn take_element<'a>(buf: &'a [u8], lookup: &[String]) -> IResult<&'a [u8], BinEl> {
    debug!("taking element");
    let mut buf = buf;

    debug!("taking name");
    let (lookup_buf, name) = take_lookup(buf, &lookup)?;
    buf = lookup_buf;
    debug!("name is {}", name);

    let mut binel = BinEl::new(&name);

    let (attr_count_buf, attr_count) = le_u8(buf)?;
    buf = attr_count_buf;

    for _ in 0..attr_count {
        debug!("taking key");
        let (key_buf, key) = take_lookup(buf, &lookup)?;
        debug!("key is {}", key);
        debug!("taking value");
        let (val_buf, val) = take_elemattr(key_buf, &lookup)?;
        buf = val_buf;
        binel.attributes.insert(key.clone(), val);
    }

    let (mut buf, child_count) = le_u16(buf)?;
    debug!("{} children", child_count);
    for _ in 0..child_count {
        let (child_buf, child) = take_element(buf, &lookup)?;
        buf = child_buf;
        binel.insert(child);
    }

    debug!("got element");

    Ok((buf, binel))
}

/// Parse a `BinFile` from a `&[u8]`. Tested solely in integration tests due to complexity.
pub fn take_file(input: &[u8]) -> IResult<&[u8], BinFile> {
    do_parse!(input,
        tag!(b"\x0bCELESTE MAP") >>
        package: take_string >>
        lookup: length_count!(le_i16, take_string) >>
        root: apply!(take_element, &lookup) >>
        ( BinFile { package, root } )
    )
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn take_header_length() {
        assert_eq!(take_varint(&b"\x0bCELESTE MAPdummy"[..]), Ok((&b"CELESTE MAPdummy"[..], 0x0b)));
    }

    #[test]
    fn take_header() {
        assert_eq!(take_string(&b"\x0bCELESTE MAPdummy"[..]), Ok((&b"dummy"[..], "CELESTE MAP".to_string())));
    }
}
