/// `parser` parses BinaryElement files.
pub mod parser;

/// `writer` writers BinaryElement files.
pub mod writer;

/// Holds BinaryElement files.
pub struct BinFile<'a, 'b> {
    pub package: &'a str,
    pub rest: &'b [u8] // TODO: finish parser
}
