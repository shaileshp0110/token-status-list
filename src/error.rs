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
    StatusListCreationError(String),
    SerializationError(String),
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderError::Base64Error(msg) => write!(f, "Base64 decoding error: {}", msg),
            DecoderError::DecompressionError(msg) => write!(f, "ZLIB decompression error: {}", msg),
            DecoderError::InvalidByteIndex(idx) => write!(f, "Invalid byte index: {}", idx),
            DecoderError::InvalidStatusType(val) => write!(f, "Invalid status type value: {}", val),
            DecoderError::StatusListCreationError(msg) => {
                write!(f, "Status list creation error: {}", msg)
            }
            DecoderError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for DecoderError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_list_creation_error_display() {
        let error = DecoderError::StatusListCreationError("invalid bits per status".to_string());
        assert_eq!(
            error.to_string(),
            "Status list creation error: invalid bits per status"
        );
    }

    #[test]
    fn test_serialization_error_display() {
        let error = DecoderError::SerializationError("failed to serialize JSON".to_string());
        assert_eq!(
            error.to_string(),
            "Serialization error: failed to serialize JSON"
        );
    }

    #[test]
    fn test_all_decoder_error_variants() {
        let errors = [
            DecoderError::Base64Error("invalid base64".to_string()),
            DecoderError::DecompressionError("failed to decompress".to_string()),
            DecoderError::InvalidByteIndex(42),
            DecoderError::InvalidStatusType(255),
            DecoderError::StatusListCreationError("invalid creation".to_string()),
            DecoderError::SerializationError("invalid json".to_string()),
        ];

        for error in errors {
            let error_string = error.to_string();
            match error {
                DecoderError::Base64Error(_) => {
                    assert!(error_string.contains("Base64 decoding error"));
                }
                DecoderError::DecompressionError(_) => {
                    assert!(error_string.contains("ZLIB decompression error"));
                }
                DecoderError::InvalidByteIndex(_) => {
                    assert!(error_string.contains("Invalid byte index"));
                }
                DecoderError::InvalidStatusType(_) => {
                    assert!(error_string.contains("Invalid status type value"));
                }
                DecoderError::StatusListCreationError(_) => {
                    assert!(error_string.contains("Status list creation error"));
                }
                DecoderError::SerializationError(_) => {
                    assert!(error_string.contains("Serialization error"));
                }
            }
        }
    }
}
