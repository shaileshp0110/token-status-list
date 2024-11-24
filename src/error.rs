use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum StatusTypeError {
    UndefinedStatusType(u8),
    InvalidByteIndex(usize),
    InvalidBitsPerStatus(u8),
}

impl fmt::Display for StatusTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatusTypeError::UndefinedStatusType(x) => {
                write!(f, "Undefined Status Type {}", x)
            }
            StatusTypeError::InvalidByteIndex(x) => {
                write!(f, "Invalid Byte Index {}", x)
            }
            StatusTypeError::InvalidBitsPerStatus(x) => {
                write!(
                    f,
                    "Invalid bits per status value: {}. Must be 1, 2, 4, or 8",
                    x
                )
            }
        }
    }
}

impl Error for StatusTypeError {}

#[derive(Debug)]
pub enum BuilderError {
    InvalidBitsPerStatus(u8),
    CompressionError(String),
    EncodingError(String),
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuilderError::InvalidBitsPerStatus(bits) => {
                write!(
                    f,
                    "Invalid bits per status value: {}. Must be 1, 2, 4, or 8",
                    bits
                )
            }
            BuilderError::CompressionError(msg) => {
                write!(f, "Compression error: {}", msg)
            }
            BuilderError::EncodingError(msg) => {
                write!(f, "Encoding error: {}", msg)
            }
        }
    }
}

impl Error for BuilderError {}
