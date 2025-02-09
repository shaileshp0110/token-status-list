use crate::error::DecoderError;
use crate::types::{StatusList, StatusType};
use flate2::read::ZlibDecoder;
use std::io::Read;

pub struct StatusListDecoder {
    raw_bytes: Vec<u8>,
    bits_per_status: u8,
}

impl StatusListDecoder {
    pub fn new(status_list: &StatusList) -> Result<Self, DecoderError> {
        let mut decoder = ZlibDecoder::new(&status_list.lst[..]);
        let mut raw_bytes = Vec::new();
        decoder
            .read_to_end(&mut raw_bytes)
            .map_err(|e| DecoderError::DecompressionError(e.to_string()))?;

        Ok(Self {
            raw_bytes,
            bits_per_status: status_list.bits,
        })
    }

    pub fn get_status(&self, index: usize) -> Result<StatusType, DecoderError> {
        let statuses_per_byte = 8 / self.bits_per_status as usize;
        let byte_index = index / statuses_per_byte;
        let position_in_byte = index % statuses_per_byte;

        if byte_index >= self.raw_bytes.len() {
            return Err(DecoderError::InvalidByteIndex(byte_index));
        }

        let byte = self.raw_bytes[byte_index];

        if self.bits_per_status == 8 {
            StatusType::try_from(byte).map_err(|_| DecoderError::InvalidStatusType(byte))
        } else {
            let bit_shift = match self.bits_per_status {
                1 => position_in_byte,
                2 => match position_in_byte {
                    0 => 0,
                    1 => 2,
                    2 => 4,
                    3 => 6,
                    _ => unreachable!(),
                },
                4 => {
                    if position_in_byte == 0 {
                        4
                    } else {
                        0
                    }
                }
                _ => unreachable!(),
            };

            let mask = (1u8 << self.bits_per_status) - 1;
            let value = (byte >> bit_shift) & mask;

            StatusType::try_from(value).map_err(|_| DecoderError::InvalidStatusType(value))
        }
    }

    pub fn get_raw_bytes(&self) -> &[u8] {
        &self.raw_bytes
    }

    pub fn len(&self) -> usize {
        self.raw_bytes.len() * (8 / self.bits_per_status as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.raw_bytes.is_empty()
    }

    pub fn new_from_base64(base64_str: &str) -> Result<Self, DecoderError> {
        let compressed =
            base64url::decode(base64_str).map_err(|e| DecoderError::Base64Error(e.to_string()))?;

        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut raw_bytes = Vec::new();
        decoder
            .read_to_end(&mut raw_bytes)
            .map_err(|e| DecoderError::DecompressionError(e.to_string()))?;

        Ok(Self {
            raw_bytes,
            bits_per_status: 8,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::StatusListBuilder;
    use serde_json::Value;

    #[test]
    fn test_decode_1bit_encoding() -> Result<(), DecoderError> {
        let builder = StatusListBuilder::new(1)
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid);

        let status_list = builder
            .build()
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        let decoder = StatusListDecoder::new(&status_list)?;

        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(1)?, StatusType::Invalid);
        assert_eq!(decoder.get_status(2)?, StatusType::Valid);
        assert_eq!(decoder.get_status(3)?, StatusType::Invalid);

        Ok(())
    }

    #[test]
    fn test_decode_2bit_encoding() -> Result<(), DecoderError> {
        let builder = StatusListBuilder::new(2)
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Suspended)
            .add_status(StatusType::ApplicationSpecific3);

        let status_list = builder
            .build()
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        let decoder = StatusListDecoder::new(&status_list)?;

        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(1)?, StatusType::Invalid);
        assert_eq!(decoder.get_status(2)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific3);

        Ok(())
    }

    #[test]
    fn test_decode_4bit_encoding() -> Result<(), DecoderError> {
        let builder = StatusListBuilder::new(4)
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Suspended)
            .add_status(StatusType::ApplicationSpecific15);
        let status_list = builder
            .build()
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        let decoder = StatusListDecoder::new(&status_list)?;
        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(2)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific15);

        Ok(())
    }

    #[test]
    fn test_decode_8bit_encoding() -> Result<(), DecoderError> {
        let builder = StatusListBuilder::new(8)
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Suspended)
            .add_status(StatusType::ApplicationSpecific15);
        let status_list = builder
            .build()
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        let decoder = StatusListDecoder::new(&status_list)?;
        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(2)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific15);

        Ok(())
    }

    #[test]
    fn test_base64_decoding() -> Result<(), DecoderError> {
        let builder = StatusListBuilder::new(8)
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Suspended)
            .add_status(StatusType::ApplicationSpecific3);

        let status_list = builder
            .build()
            .map_err(|e| DecoderError::StatusListCreationError(e.to_string()))?;
        let json = status_list
            .to_json()
            .map_err(|e| DecoderError::SerializationError(e.to_string()))?;
        let decoded: Value = serde_json::from_str(&json)
            .map_err(|e| DecoderError::SerializationError(e.to_string()))?;

        let base64_str = decoded["lst"].as_str().unwrap();

        let decoder = StatusListDecoder::new_from_base64(base64_str)?;

        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(1)?, StatusType::Invalid);
        assert_eq!(decoder.get_status(2)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific3);

        Ok(())
    }

    #[test]
    fn test_decoder_base64_error() {
        let status_list = StatusList {
            bits: 2,
            lst: vec![0xFF, 0xFF],
            aggregation_uri: None,
        };

        match StatusListDecoder::new(&status_list) {
            Err(DecoderError::DecompressionError(_)) => (),
            _ => panic!("Expected DecompressionError"),
        }
    }

    #[test]
    fn test_decoder_base64_from_string() {
        match StatusListDecoder::new_from_base64("invalid base64!@#$") {
            Err(e) => assert!(e.to_string().contains("Base64 decoding error")),
            _ => panic!("Expected Base64 decoding error"),
        }
    }

    #[test]
    fn test_decoder_decompression_error() {
        let status_list = StatusList {
            bits: 2,
            lst: "SGVsbG8gV29ybGQh".as_bytes().to_vec(),
            aggregation_uri: None,
        };

        match StatusListDecoder::new(&status_list) {
            Err(DecoderError::DecompressionError(_)) => (),
            _ => panic!("Expected DecompressionError"),
        }
    }

    #[test]
    fn test_decoder_invalid_byte_index() {
        let builder = StatusListBuilder::new(2).unwrap();
        builder.add_status(StatusType::Valid);
        let status_list = builder.build().unwrap();
        let decoder = StatusListDecoder::new(&status_list).unwrap();

        match decoder.get_status(100) {
            Err(DecoderError::InvalidByteIndex(_)) => (),
            _ => panic!("Expected InvalidByteIndex error"),
        }
    }

    #[test]
    fn test_decoder_invalid_status_type() {
        let status_list = StatusList {
            bits: 8,
            lst: "eJzLBQAAdgB2".as_bytes().to_vec(),
            aggregation_uri: None,
        };

        if let Ok(decoder) = StatusListDecoder::new(&status_list) {
            match decoder.get_status(0) {
                Err(DecoderError::InvalidStatusType(_)) => (),
                _ => panic!("Expected InvalidStatusType error"),
            }
        }
    }

    #[test]
    fn test_decoder_error_display() {
        let errors = [
            DecoderError::Base64Error("invalid input".to_string()),
            DecoderError::DecompressionError("invalid data".to_string()),
            DecoderError::InvalidByteIndex(100),
            DecoderError::InvalidStatusType(255),
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
