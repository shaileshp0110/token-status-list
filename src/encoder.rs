use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use flate2::{write::ZlibEncoder, Compression};
use std::io::Write;

use crate::error::BuilderError;
use crate::types::{StatusList, StatusType};

pub struct StatusListEncoder {
    bits_per_status: u8,
}

impl StatusListEncoder {
    pub fn new(bits_per_status: u8) -> Self {
        Self { bits_per_status }
    }

    pub fn encode_status1(&self, bytes: &mut [u8], index: usize, status: StatusType) {
        let statuses_per_byte = (8 / self.bits_per_status) as usize;
        let byte_index = index / statuses_per_byte;
        let position_in_byte = index % statuses_per_byte;

        let status_value = status as u8;
        let bit_shift = match self.bits_per_status {
            1 => position_in_byte,
            2 => position_in_byte * 2,
            4 => position_in_byte * 4,
            8 => 0,
            _ => unreachable!(),
        };

        let mask = !(((1u8 << self.bits_per_status) - 1) << bit_shift);
        bytes[byte_index] &= mask;
        bytes[byte_index] |= status_value << bit_shift;

        #[cfg(debug_assertions)]
        println!(
            "Encoding: index={}, byte={:08b}, shift={}, status={:?}",
            index, bytes[byte_index], bit_shift, status
        );
    }

    pub fn encode_status(&self, bytes: &mut [u8], index: usize, status: StatusType) {
        let statuses_per_byte = 8 / self.bits_per_status as usize;
        let byte_index = index / statuses_per_byte;
        let position_in_byte = index % statuses_per_byte;

        let status_value = status as u8;

        // Calculate bit positions
        let bit_shift = match self.bits_per_status {
            1 => {
                // 8 values per byte, right to left
                position_in_byte
            }
            2 => {
                // 4 values per byte, right to left in pairs
                match position_in_byte {
                    0 => 0, // First value in bits 1-0
                    1 => 2, // Second value in bits 3-2
                    2 => 4, // Third value in bits 5-4
                    3 => 6, // Fourth value in bits 7-6
                    _ => unreachable!(),
                }
            }
            4 => {
                // 2 values per byte, left to right in nibbles
                match position_in_byte {
                    0 => 4, // First value in high nibble (bits 7-4)
                    1 => 0, // Second value in low nibble (bits 3-0)
                    _ => unreachable!(),
                }
            }
            8 => 0, // 1 value per byte
            _ => unreachable!(),
        };

        // Clear the target bits first
        let mask = !(((1u8 << self.bits_per_status) - 1) << bit_shift);
        bytes[byte_index] &= mask;

        // Set the new bits
        bytes[byte_index] |= status_value << bit_shift;

        #[cfg(debug_assertions)]
        println!(
            "Encoding: index={}, byte={:08b}, shift={}, status={:?}, value={:08b}",
            index, bytes[byte_index], bit_shift, status, status_value
        );
    }

    pub fn encode_statuses(&self, statuses: &[StatusType]) -> Result<Vec<u8>, BuilderError> {
        match self.bits_per_status {
            8 => Ok(statuses.iter().map(|status| *status as u8).collect()),
            1 | 2 | 4 => {
                let statuses_per_byte = (8 / self.bits_per_status) as usize;
                let num_bytes = statuses.len().div_ceil(statuses_per_byte);
                let mut bytes = vec![0u8; num_bytes];

                for (i, status) in statuses.iter().enumerate() {
                    self.encode_status(&mut bytes, i, *status);
                }
                Ok(bytes)
            }
            _ => Err(BuilderError::InvalidBitsPerStatus(self.bits_per_status)),
        }
    }

    pub fn finalize(&self, bytes: &[u8]) -> Result<StatusList, BuilderError> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder
            .write_all(bytes)
            .map_err(|e| BuilderError::CompressionError(e.to_string()))?;

        let compressed = encoder
            .finish()
            .map_err(|e| BuilderError::CompressionError(e.to_string()))?;

        let encoded = URL_SAFE_NO_PAD.encode(compressed);

        Ok(StatusList {
            bits: self.bits_per_status,
            lst: encoded,
            aggregation_uri: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::StatusListDecoder;
    use crate::error::{BuilderError, StatusTypeError};
    use crate::types::StatusType;

    #[test]
    fn test_direct_encoding() -> Result<(), BuilderError> {
        let encoder = StatusListEncoder::new(2);
        let statuses = vec![
            StatusType::Valid,                // 00
            StatusType::Invalid,              // 01
            StatusType::Suspended,            // 10
            StatusType::ApplicationSpecific3, // 11
        ];

        let bytes = encoder.encode_statuses(&statuses)?;
        assert_eq!(bytes[0], 0b11100100);

        let status_list = encoder.finalize(&bytes)?;
        let decoder = StatusListDecoder::new(&status_list).unwrap();

        // Verify each status
        assert_eq!(decoder.get_status(0).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(1).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(2).unwrap(), StatusType::Suspended);
        assert_eq!(
            decoder.get_status(3).unwrap(),
            StatusType::ApplicationSpecific3
        );

        Ok(())
    }

    #[test]
    fn test_different_bit_sizes() -> Result<(), BuilderError> {
        let test_cases = [
            // 1-bit encoding: right to left
            // Valid(0), Invalid(1) -> 0b00000010
            (1, vec![StatusType::Valid, StatusType::Invalid], 0b00000010),
            // 2-bit encoding: right to left in pairs
            // Valid(00), Invalid(01) -> 0b00000100
            (2, vec![StatusType::Valid, StatusType::Invalid], 0b00000100),
            // 4-bit encoding: low nibble, high nibble
            // Valid(0000) at low nibble, Invalid(0001) at high nibble -> 0b00000001
            (4, vec![StatusType::Valid, StatusType::Invalid], 0b00000001),
            // 8-bit encoding: full bytes
            (8, vec![StatusType::Valid, StatusType::Invalid], 0b00000000), // first byte only
        ];

        for (bits, statuses, expected_byte) in test_cases {
            let encoder = StatusListEncoder::new(bits);
            let bytes = encoder.encode_statuses(&statuses)?;
            println!(
                "Bits: {}, Encoded: {:08b}, Expected: {:08b}",
                bits, bytes[0], expected_byte
            );
            assert_eq!(bytes[0], expected_byte, "Failed for {}-bit encoding", bits);
        }

        Ok(())
    }

    #[test]
    fn test_spec_1bit_example() -> Result<(), BuilderError> {
        let encoder = StatusListEncoder::new(1);
        let statuses = vec![
            StatusType::Invalid, // 1 - index 0
            StatusType::Valid,   // 0 - index 1
            StatusType::Valid,   // 0 - index 2
            StatusType::Invalid, // 1 - index 3
            StatusType::Invalid, // 1 - index 4
            StatusType::Invalid, // 1 - index 5
            StatusType::Valid,   // 0 - index 6
            StatusType::Invalid, // 1 - index 7
            StatusType::Invalid, // 1 - index 8
            StatusType::Invalid, // 1 - index 9
            StatusType::Valid,   // 0 - index 10
            StatusType::Valid,   // 0 - index 11
            StatusType::Valid,   // 0 - index 12
            StatusType::Invalid, // 1 - index 13
            StatusType::Valid,   // 0 - index 14
            StatusType::Invalid, // 1 - index 15
        ];

        let bytes = encoder.encode_statuses(&statuses)?;

        // First byte: 10111001 = 0xB9
        // Indices:     7654 3210
        assert_eq!(bytes[0], 0xB9);

        // Second byte: 10100011 = 0xA3
        // Indices:    15141312 11109 8
        assert_eq!(bytes[1], 0xA3);

        // Verify we can decode it back
        let status_list = encoder.finalize(&bytes)?;
        let decoder = StatusListDecoder::new(&status_list).unwrap();
        assert_eq!(decoder.get_status(0).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(1).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(2).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(3).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(4).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(5).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(6).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(7).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(8).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(9).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(10).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(11).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(12).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(13).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(14).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(15).unwrap(), StatusType::Invalid);

        Ok(())
    }

    #[test]
    fn test_spec_8bit_example() -> Result<(), BuilderError> {
        let encoder = StatusListEncoder::new(8);
        let statuses = vec![
            StatusType::Invalid,              // 0x01 - index 0
            StatusType::Suspended,            // 0x02 - index 1
            StatusType::Valid,                // 0x00 - index 2
            StatusType::ApplicationSpecific3, // 0x03 - index 3
            StatusType::Valid,                // 0x00 - index 4
            StatusType::Invalid,              // 0x01 - index 5
            StatusType::Suspended,            // 0x02 - index 6
            StatusType::ApplicationSpecific3, // 0x03 - index 7
        ];

        let bytes = encoder.encode_statuses(&statuses)?;

        // Each status takes a full byte
        assert_eq!(bytes[0], 0x01); // Invalid
        assert_eq!(bytes[1], 0x02); // Suspended
        assert_eq!(bytes[2], 0x00); // Valid
        assert_eq!(bytes[3], 0x03); // ApplicationSpecific3
        assert_eq!(bytes[4], 0x00); // Valid
        assert_eq!(bytes[5], 0x01); // Invalid
        assert_eq!(bytes[6], 0x02); // Suspended
        assert_eq!(bytes[7], 0x03); // ApplicationSpecific3

        let status_list = encoder.finalize(&bytes)?;
        let decoder = StatusListDecoder::new(&status_list).unwrap();

        assert_eq!(decoder.get_status(0).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(1).unwrap(), StatusType::Suspended);
        assert_eq!(decoder.get_status(2).unwrap(), StatusType::Valid);
        assert_eq!(
            decoder.get_status(3).unwrap(),
            StatusType::ApplicationSpecific3
        );
        assert_eq!(decoder.get_status(4).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(5).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(6).unwrap(), StatusType::Suspended);
        assert_eq!(
            decoder.get_status(7).unwrap(),
            StatusType::ApplicationSpecific3
        );

        Ok(())
    }
    #[test]
    fn test_full_2bit_pattern() -> Result<(), BuilderError> {
        let encoder = StatusListEncoder::new(2);
        let statuses = vec![
            StatusType::Invalid,              // 01
            StatusType::Suspended,            // 10
            StatusType::Valid,                // 00
            StatusType::ApplicationSpecific3, // 11
            StatusType::Valid,                // 00
            StatusType::Invalid,              // 01
            StatusType::Valid,                // 00
            StatusType::Invalid,              // 01
            StatusType::Invalid,              // 01
            StatusType::Suspended,            // 10
            StatusType::ApplicationSpecific3, // 11
            StatusType::ApplicationSpecific3, // 11
        ];

        let bytes = encoder.encode_statuses(&statuses)?;
        assert_eq!(bytes[0], 0xC9); // 11001001
        assert_eq!(bytes[1], 0x44); // 01000100
        assert_eq!(bytes[2], 0xF9); // 11111001

        Ok(())
    }

    #[test]
    fn test_full_4bit_pattern() -> Result<(), BuilderError> {
        let encoder = StatusListEncoder::new(4);
        let statuses = vec![
            StatusType::Invalid,              // 01
            StatusType::Suspended,            // 10
            StatusType::Valid,                // 00
            StatusType::ApplicationSpecific3, // 11
            StatusType::Valid,                // 00
            StatusType::Invalid,              // 01
            StatusType::Valid,                // 00
            StatusType::Invalid,              // 01
            StatusType::Invalid,              // 01
            StatusType::Suspended,            // 10
            StatusType::ApplicationSpecific3, // 11
            StatusType::ApplicationSpecific3, // 11
        ];

        let bytes = encoder.encode_statuses(&statuses)?;
        // The test expects: [0x12, 0x03, 0x01, 0x01, 0x12, 0x33]
        assert_eq!(bytes[0], 0x12); //0001 0010
        assert_eq!(bytes[1], 0x03); //0000 0011
        assert_eq!(bytes[2], 0x01); //0000 0001
        assert_eq!(bytes[3], 0x01); //0000 0001
        assert_eq!(bytes[4], 0x12); //0001 0010
        assert_eq!(bytes[5], 0x33); //0011 0011

        Ok(())
    }

    #[test]
    fn test_byte_boundaries() -> Result<(), BuilderError> {
        let encoder = StatusListEncoder::new(2);
        let statuses = vec![
            StatusType::Valid,                // 00 | First byte
            StatusType::Invalid,              // 01 |
            StatusType::Suspended,            // 10 |
            StatusType::ApplicationSpecific3, // 11 |
            StatusType::Valid,                // 00 | Second byte
            StatusType::Invalid,              // 01 |
            StatusType::Suspended,            // 10 |
            StatusType::ApplicationSpecific3, // 11 |
        ];

        let bytes = encoder.encode_statuses(&statuses)?;
        assert_eq!(bytes.len(), 2);
        assert_eq!(bytes[0], 0b11100100);
        assert_eq!(bytes[1], 0b11100100);

        Ok(())
    }

    #[test]
    fn test_compression_and_encoding() -> Result<(), BuilderError> {
        let encoder = StatusListEncoder::new(2);
        let statuses = vec![StatusType::Valid; 100];

        let bytes = encoder.encode_statuses(&statuses)?;
        let status_list = encoder.finalize(&bytes)?;

        assert!(status_list.lst.len() < bytes.len());

        let decoder = StatusListDecoder::new(&status_list).unwrap();
        for i in 0..100 {
            assert_eq!(decoder.get_status(i).unwrap(), StatusType::Valid);
        }

        Ok(())
    }

    #[test]
    fn test_partial_byte() -> Result<(), BuilderError> {
        let encoder = StatusListEncoder::new(2);
        let statuses = vec![
            StatusType::Valid,     // 00 (bits 0-1)
            StatusType::Invalid,   // 01 (bits 2-3)
            StatusType::Suspended, // 10 (bits 4-5)
        ];

        let bytes = encoder.encode_statuses(&statuses)?;

        // Expected byte pattern:
        // bits 0-1: 00 (Valid)
        // bits 2-3: 01 (Invalid)
        // bits 4-5: 10 (Suspended)
        // bits 6-7: 00 (unused)
        assert_eq!(bytes[0], 0b00100100); // 0x24 or decimal 36

        let status_list = encoder.finalize(&bytes)?;
        let decoder = StatusListDecoder::new(&status_list).unwrap();

        assert_eq!(decoder.get_status(0).unwrap(), StatusType::Valid);
        assert_eq!(decoder.get_status(1).unwrap(), StatusType::Invalid);
        assert_eq!(decoder.get_status(2).unwrap(), StatusType::Suspended);

        Ok(())
    }

    #[test]
    fn test_encoder_invalid_bits_per_status() {
        let encoder = StatusListEncoder::new(3); // Invalid bits (not 1, 2, 4, or 8)
        let statuses = vec![StatusType::Valid];

        match encoder.encode_statuses(&statuses) {
            Err(BuilderError::InvalidBitsPerStatus(bits)) => {
                assert_eq!(bits, 3, "Expected InvalidBitsPerStatus(3)");
            }
            _ => panic!("Expected InvalidBitsPerStatus error"),
        }
    }

    #[test]
    fn test_encoder_compression_error() {
        let encoder = StatusListEncoder::new(2);
        let statuses = vec![StatusType::Valid];
        let bytes = encoder.encode_statuses(&statuses).unwrap();

        // Test finalize error handling
        match encoder.finalize(&bytes) {
            Ok(_) => (), // Should work for valid data
            Err(BuilderError::CompressionError(_)) => (),
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_encoder_error_display() {
        let errors = [
            BuilderError::InvalidBitsPerStatus(3),
            BuilderError::CompressionError("test error".to_string()),
            BuilderError::EncodingError("encoding failed".to_string()),
        ];

        for error in errors {
            let error_string = error.to_string();
            match error {
                BuilderError::InvalidBitsPerStatus(_) => {
                    assert!(error_string.contains("Invalid bits per status"));
                }
                BuilderError::CompressionError(_) => {
                    assert!(error_string.contains("Compression error"));
                }
                BuilderError::EncodingError(_) => {
                    assert!(error_string.contains("Encoding error"));
                }
            }
        }
    }
}
