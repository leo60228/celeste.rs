use snafu::Snafu;
use std::borrow::Cow;
use std::prelude::v1::*;
use std::result::Result as StdResult;

#[cfg(feature = "std")]
use std::{io, sync::Arc};

#[cfg(feature = "std")]
use arc_io_error::IoError;

#[cfg(feature = "std")]
use std::error::Error as StdError;

/// Error type for this crate.
#[derive(Debug, Snafu, Clone)]
pub enum Error<'a> {
    /// Received when the library is unable to parse a BinEl.
    #[snafu(display("Could not parse BinEl `{}`", name))]
    InvalidBinEl {
        /// The name of the BinEl that the library was trying to parse.
        name: Cow<'static, str>,
        /// The name of the BinEl that the library found, if it exists.
        received_name: Option<String>,
    },
    /// An error that occurred while writing a file. Only available when the
    /// `std` feature is enabled.
    #[cfg(feature = "std")]
    #[snafu(display("Error writing file: {}", source))]
    Write {
        /// This is provided by the arc_io_error crate, as nom requires error
        /// types to implement Clone.
        source: IoError,
    },
    /// This error occurs when a BinEl passed to the library has an invalid
    /// binary format.
    #[snafu(display("Error parsing BinEl: {:?}", source))]
    ParseBinEl {
        /// The source of the error.
        #[snafu(source(false))]
        source: (&'a [u8], nom::error::ErrorKind),
    },
    /// This error occurs when a dialog file passed to the library has an
    /// invalid format.
    #[snafu(display("Error parsing Dialog: {:?}", source))]
    ParseDialog {
        /// The source of the error.
        #[snafu(source(false))]
        source: (&'a str, nom::error::ErrorKind),
    },
    /// This error occurs when a file's data is incomplete.
    #[snafu(display("Incomplete data when parsing file"))]
    Incomplete,
    #[doc(hidden)]
    __NonExhaustive,
}

impl Error<'_> {
    /// Create an error from a name of a BinEl.
    pub fn from_name(name: impl Into<Cow<'static, str>>) -> Self {
        Error::InvalidBinEl {
            name: name.into(),
            received_name: None,
        }
    }

    /// Create an error from the received name of a BinEl, along with the actual
    /// name.
    pub fn wrong_name(name: impl Into<Cow<'static, str>>, received_name: String) -> Self {
        Error::InvalidBinEl {
            name: name.into(),
            received_name: Some(received_name),
        }
    }

    /// Shorthand for `Error::Write(...)`. Only availabe when the `std` feature
    /// is enabled.
    #[cfg(feature = "std")]
    pub fn io(kind: io::ErrorKind, text: impl Into<Box<dyn StdError + Send + Sync>>) -> Self {
        Error::Write {
            source: IoError::new(kind, Arc::from(text.into())),
        }
    }
}

#[cfg(feature = "std")]
impl From<io::Error> for Error<'_> {
    fn from(source: io::Error) -> Self {
        Error::Write {
            source: source.into(),
        }
    }
}

impl<'a> From<nom::Err<Error<'a>>> for Error<'a> {
    fn from(source: nom::Err<Error<'a>>) -> Self {
        use nom::Err;
        match source {
            Err::Incomplete(_) => Error::Incomplete,
            Err::Error(source) => source,
            Err::Failure(source) => source,
        }
    }
}

impl<'a> nom::error::ParseError<&'a [u8]> for Error<'a> {
    fn from_error_kind(input: &'a [u8], kind: nom::error::ErrorKind) -> Self {
        Error::ParseBinEl {
            source: (input, kind),
        }
    }

    fn append(input: &'a [u8], kind: nom::error::ErrorKind, _other: Self) -> Self {
        Self::from_error_kind(input, kind)
    }
}

impl<'a> nom::error::ParseError<&'a str> for Error<'a> {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        Error::ParseDialog {
            source: (input, kind),
        }
    }

    fn append(input: &'a str, kind: nom::error::ErrorKind, _other: Self) -> Self {
        Self::from_error_kind(input, kind)
    }
}

pub(crate) type Result<'a, T> = StdResult<T, Error<'a>>;
