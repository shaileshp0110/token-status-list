use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusList {
    pub bits: u8,
    pub lst: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_uri: Option<String>,
}

use crate::error::StatusTypeError;
