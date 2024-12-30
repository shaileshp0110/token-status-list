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

#[derive(Debug)]
pub enum DecoderError {
    Base64Error(String),
    DecompressionError(String),
    InvalidByteIndex(usize),
    InvalidStatusType(u8),
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderError::Base64Error(msg) => write!(f, "Base64 decoding error: {}", msg),
            DecoderError::DecompressionError(msg) => write!(f, "ZLIB decompression error: {}", msg),
            DecoderError::InvalidByteIndex(idx) => write!(f, "Invalid byte index: {}", idx),
            DecoderError::InvalidStatusType(val) => write!(f, "Invalid status type value: {}", val),
        }
    }
}

impl std::error::Error for DecoderError {}
