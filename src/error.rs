use core::fmt;

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::string::{String, ToString};

/// A specialized [`Result`][std-result-result] type for `store` operations.
///
/// [std-result-result]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = ::core::result::Result<T, Error>;

/// Errors that can occur when dumping/loading data.
#[derive(Debug)]
pub enum Error {
    EndOfStream,
    UnsupportedDataStructure,
    InvalidEncoding,
    InvalidUtf8Encoding,
    SequencesMustHaveLength,
    SequenceTooLong,
    TooManyEnumVariants,
    Custom {
        msg: &'static str,
    },
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    Serde,
    #[cfg(any(feature = "std", feature = "alloc"))]
    Serde {
        msg: String,
    },
}

impl fmt::Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            self::Error::EndOfStream => {
                write!(f, "reached end of stream but needed more bytes")
            }
            self::Error::UnsupportedDataStructure => {
                write!(f, "encountered an unsupported data structure")
            }
            self::Error::InvalidEncoding => {
                write!(f, "encountered a value with an unexpected binary encoding")
            }
            self::Error::InvalidUtf8Encoding => {
                write!(f, "encountered a string with invalid UTF-8 encoding")
            }
            self::Error::SequencesMustHaveLength => {
                write!(f, "sequences must have a known length")
            }
            self::Error::SequenceTooLong => {
                write!(f, "encountered a sequence with too great a length")
            }
            self::Error::TooManyEnumVariants => {
                write!(f, "encountered an enum with too many variants")
            }
            self::Error::Custom { msg } => {
                write!(f, "internal error: {}", msg)
            }
            #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
            self::Error::Serde => {
                write!(f, "internal serde error")
            }
            #[cfg(any(feature = "std", feature = "alloc"))]
            self::Error::Serde { msg } => {
                write!(f, "internal serde error: {}", msg)
            }
        }
    }
}

impl From<::byteio::Error> for Error {
    fn from(_: ::byteio::Error) -> Self {
        Error::EndOfStream
    }
}

#[cfg(feature = "std")]
impl ::alloc::error::Error for Error {}

impl ::serde::de::Error for Error {
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        Error::Serde
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Serde { msg: msg.to_string() }
    }
}

impl ::serde::ser::Error for Error {
    #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        Error::Serde
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Error::Serde { msg: msg.to_string() }
    }
}
