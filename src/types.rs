use serde::Serialize;
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusType {
    Valid = 0x00,
    Invalid = 0x01,
    Suspended = 0x02,
    ApplicationSpecific3 = 0x03,
    ApplicationSpecific14 = 0x0E,
    ApplicationSpecific15 = 0x0F,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitsPerStatus {
    OneBit = 1,
    TwoBit = 2,
    FourBit = 4,
    EightBit = 8,
}

impl TryFrom<u8> for BitsPerStatus {
    type Error = StatusTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(BitsPerStatus::OneBit),
            2 => Ok(BitsPerStatus::TwoBit),
            4 => Ok(BitsPerStatus::FourBit),
            8 => Ok(BitsPerStatus::EightBit),
            _ => Err(StatusTypeError::InvalidBitsPerStatus(value)),
        }
    }
}

impl TryFrom<u8> for StatusType {
    type Error = StatusTypeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(StatusType::Valid),
            0x01 => Ok(StatusType::Invalid),
            0x02 => Ok(StatusType::Suspended),
            0x03 => Ok(StatusType::ApplicationSpecific3),
            0x0E => Ok(StatusType::ApplicationSpecific14),
            0x0F => Ok(StatusType::ApplicationSpecific15),
            _ => Err(StatusTypeError::UndefinedStatusType(value)),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct StatusList {
    pub bits: u8,
    #[serde(with = "serde_bytes")]
    pub lst: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_uri: Option<String>,
}

#[derive(Serialize)]
pub struct JsonStatusList<'a> {
    pub bits: u8,
    pub lst: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_uri: Option<&'a String>,
}

#[derive(Serialize, Debug)]
pub struct CborStatusList<'a> {
    pub bits: u8,
    #[serde(rename = "lst")]
    #[serde(with = "serde_bytes")]
    pub lst: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_uri: Option<&'a String>,
}

use crate::error::StatusTypeError;

#[derive(Debug)]
pub enum SerializationError {
    JsonError(String),
    CborError(String),
}

impl StatusList {
    pub fn to_json(&self) -> Result<String, SerializationError> {
        let json_list = JsonStatusList {
            bits: self.bits,
            lst: base64url::encode(&self.lst),
            aggregation_uri: self.aggregation_uri.as_ref(),
        };

        serde_json::to_string(&json_list).map_err(|e| SerializationError::JsonError(e.to_string()))
    }

    pub fn to_cbor(&self) -> Result<String, SerializationError> {
        let cbor_list = CborStatusList {
            bits: self.bits,
            lst: self.lst.clone(),
            aggregation_uri: self.aggregation_uri.as_ref(),
        };

        let mut cbor_data = Vec::new();
        ciborium::ser::into_writer(&cbor_list, &mut cbor_data)
            .map_err(|e| SerializationError::CborError(e.to_string()))?;

        let mut hex = String::with_capacity(cbor_data.len() * 2);
        for byte in cbor_data {
            write!(&mut hex, "{:02x}", byte)
                .map_err(|e| SerializationError::CborError(e.to_string()))?;
        }
        Ok(hex)
    }
}

impl std::fmt::Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationError::JsonError(msg) => write!(f, "JSON serialization error: {}", msg),
            SerializationError::CborError(msg) => write!(f, "CBOR serialization error: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serialization_errors() {
        // Create an invalid status list
        let invalid_status_list = StatusList {
            bits: 1,
            lst: vec![0xFF, 0xFF], // Invalid compressed data
            aggregation_uri: None,
        };

        // Test JSON serialization
        assert!(invalid_status_list.to_json().is_ok());

        // Test CBOR serialization
        assert!(invalid_status_list.to_cbor().is_ok());
    }

    #[test]
    fn test_serialization_formats() {
        let status_list = StatusList {
            bits: 1,
            lst: vec![0xB9, 0xA3],
            aggregation_uri: None,
        };

        // Test JSON format
        let json = status_list.to_json().unwrap();
        assert!(json.contains("\"bits\":1"));
        assert!(json.contains("\"lst\":"));

        // Test CBOR format
        let cbor = status_list.to_cbor().unwrap();
        assert!(cbor.starts_with("a2"));
        assert!(cbor.contains("6462697473"));
        assert!(cbor.contains("636c7374"));
    }

    #[test]
    fn test_serialization_error_display() {
        let json_error = SerializationError::JsonError("test error".to_string());
        assert_eq!(
            json_error.to_string(),
            "JSON serialization error: test error"
        );

        let cbor_error = SerializationError::CborError("test error".to_string());
        assert_eq!(
            cbor_error.to_string(),
            "CBOR serialization error: test error"
        );
    }
}
