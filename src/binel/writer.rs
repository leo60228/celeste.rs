use std::io::prelude::*;

/// Write a string using a varint for the length.
///
/// # Examples:
/// ```
/// use std::io::Cursor;
/// use celeste::binel::writer::*;
///
/// let mut buf = Cursor::new(vec![0; 12]);
///
/// put_string(&mut buf, "CELESTE MAP").unwrap();
///
/// assert_eq!(&buf.get_ref()[..], b"\x0bCELESTE MAP");
/// ```
pub fn put_string<W: Write>(writer: &mut W, string: &str) -> std::io::Result<()> {
    let mut length_buf = unsigned_varint::encode::usize_buffer();
    let length = unsigned_varint::encode::usize(string.len(), &mut length_buf);
    writer.write_all(length)?;

    writer.write_all(string.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    #[test]
    fn put_header() {
        let mut buf = Cursor::new(vec![0; 12]);

        super::put_string(&mut buf, "CELESTE MAP").unwrap();

        assert_eq!(&buf.get_ref()[..], b"\x0bCELESTE MAP");
    }
}
