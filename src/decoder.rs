use crate::error::StatusTypeError;
use crate::types::{StatusList, StatusType};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use flate2::read::ZlibDecoder;
use std::io::Read;

pub struct StatusListDecoder {
    raw_bytes: Vec<u8>,
    bits_per_status: u8,
}

impl StatusListDecoder {
    pub fn new(status_list: &StatusList) -> Result<Self, Box<dyn std::error::Error>> {
        //decode base64url (no padding)
        let compressed = URL_SAFE_NO_PAD
            .decode(&status_list.lst)
            .map_err(|e| format!("Base64 decoding error: {}", e))?;

        //decompress ZLIB
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut raw_bytes = Vec::new();
        decoder
            .read_to_end(&mut raw_bytes)
            .map_err(|e| format!("ZLIB decompression error: {}", e))?;

        #[cfg(debug_assertions)]
        println!(
            "Decoded raw bytes: {:?}",
            raw_bytes
                .iter()
                .map(|b| format!("{:08b}", b))
                .collect::<Vec<_>>()
        );

        Ok(Self {
            raw_bytes,
            bits_per_status: status_list.bits,
        })
    }

    pub fn get_status(&self, index: usize) -> Result<StatusType, StatusTypeError> {
        let statuses_per_byte = 8 / self.bits_per_status as usize;
        let byte_index = index / statuses_per_byte;
        let position_in_byte = index % statuses_per_byte;

        if byte_index >= self.raw_bytes.len() {
            return Err(StatusTypeError::InvalidByteIndex(byte_index));
        }

        let byte = self.raw_bytes[byte_index];

        //8-bit encoding
        if self.bits_per_status == 8 {
            return StatusType::try_from(byte);
        }

        // 1,2,4 bit encoding
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

        StatusType::try_from(value)
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

    pub fn new_from_base64(base64_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // decode base64url (no padding)
        let compressed = URL_SAFE_NO_PAD
            .decode(base64_str)
            .map_err(|e| format!("Base64 decoding error: {}", e))?;

        //decompress ZLIB
        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut raw_bytes = Vec::new();
        decoder
            .read_to_end(&mut raw_bytes)
            .map_err(|e| format!("ZLIB decompression error: {}", e))?;

        Ok(Self {
            raw_bytes,
            bits_per_status: 8, // You might need to pass this as a parameter or determine it from the data
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::StatusListBuilder;
    use serde_json::Value;

    #[test]
    fn test_decode_1bit_encoding() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = StatusListBuilder::new(1)?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid);

        let status_list = builder.build()?;
        let decoder = StatusListDecoder::new(&status_list)?;

        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(1)?, StatusType::Invalid);
        assert_eq!(decoder.get_status(2)?, StatusType::Valid);
        assert_eq!(decoder.get_status(3)?, StatusType::Invalid);

        Ok(())
    }

    #[test]
    fn test_decode_2bit_encoding() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = StatusListBuilder::new(2)?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Suspended)
            .add_status(StatusType::ApplicationSpecific3);

        let status_list = builder.build()?;
        let decoder = StatusListDecoder::new(&status_list)?;

        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(1)?, StatusType::Invalid);
        assert_eq!(decoder.get_status(2)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific3);

        Ok(())
    }
    #[test]
    fn test_decode_4bit_encoding() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = StatusListBuilder::new(4)?;
        builder
            .add_status(StatusType::Valid) // 0
            .add_status(StatusType::Invalid) // 1
            .add_status(StatusType::Suspended) // 2
            .add_status(StatusType::ApplicationSpecific15); // 15
        let status_list = builder.build()?;
        let decoder = StatusListDecoder::new(&status_list)?;
        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(2)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific15);

        Ok(())
    }
    #[test]
    fn test_decode_8bit_encoding() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = StatusListBuilder::new(8)?;
        builder
            .add_status(StatusType::Valid) // 0
            .add_status(StatusType::Invalid) // 1
            .add_status(StatusType::Suspended) // 2
            .add_status(StatusType::ApplicationSpecific15); // 15
        let status_list = builder.build()?;
        let decoder = StatusListDecoder::new(&status_list)?;
        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(2)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific15);

        Ok(())
    }
    #[test]
    fn test_base64_decoding() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = StatusListBuilder::new(8)?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Suspended)
            .add_status(StatusType::ApplicationSpecific3);

        let status_list = builder.build()?;
        let serialized = serde_json::to_string(&status_list)?;
        let decoded: Value = serde_json::from_str(&serialized)?;

        // Get the base64 encoded string
        let base64_str = decoded["lst"].as_str().unwrap();

        let decoder = StatusListDecoder::new_from_base64(base64_str)?;

        // Verify the decoded values
        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(1)?, StatusType::Invalid);
        assert_eq!(decoder.get_status(2)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific3);

        Ok(())
    }
}
