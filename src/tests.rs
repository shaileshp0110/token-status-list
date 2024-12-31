use crate::{builder::StatusListBuilder, decoder::StatusListDecoder, types::StatusType};
use serde_json::Value;

#[cfg(test)]
mod builder_tests {
    use super::*;

    #[test]
    fn test_new_status_list() -> Result<(), Box<dyn std::error::Error>> {
        StatusListBuilder::new(1)?;
        StatusListBuilder::new(2)?;
        StatusListBuilder::new(4)?;
        StatusListBuilder::new(8)?;
        Ok(())
    }

    #[test]
    fn test_invalid_bits_per_status() {
        assert!(StatusListBuilder::new(0).is_err());
        assert!(StatusListBuilder::new(3).is_err());
        assert!(StatusListBuilder::new(5).is_err());
        assert!(StatusListBuilder::new(16).is_err());
    }
}

#[cfg(test)]
mod encoding_tests {
    use super::*;

    #[test]
    fn test_8bit_encoding() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = StatusListBuilder::new(8)?;
        builder
            .add_status(StatusType::Valid)
            .add_status(StatusType::Invalid);

        let status_list = builder.build()?;
        let decoder = StatusListDecoder::new(&status_list)?;

        assert_eq!(decoder.get_status(0)?, StatusType::Valid);
        assert_eq!(decoder.get_status(1)?, StatusType::Invalid);

        Ok(())
    }
}

#[cfg(test)]
mod spec_compliance_tests {
    use super::*;

    #[test]
    fn test_spec_example() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = StatusListBuilder::new(2)?;

        // Example from the spec using 2-bit encoding
        builder
            .add_status(StatusType::Invalid)
            .add_status(StatusType::Suspended)
            .add_status(StatusType::Valid)
            .add_status(StatusType::ApplicationSpecific3);

        let status_list = builder.build()?;
        let decoder = StatusListDecoder::new(&status_list)?;

        assert_eq!(decoder.get_status(0)?, StatusType::Invalid);
        assert_eq!(decoder.get_status(1)?, StatusType::Suspended);
        assert_eq!(decoder.get_status(2)?, StatusType::Valid);
        assert_eq!(decoder.get_status(3)?, StatusType::ApplicationSpecific3);

        Ok(())
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_index() -> Result<(), Box<dyn std::error::Error>> {
        let mut builder = StatusListBuilder::new(2)?;
        builder.add_status(StatusType::Valid);
        let status_list = builder.build()?;
        let decoder = StatusListDecoder::new(&status_list)?;

        assert!(decoder.get_status(100).is_err());
        Ok(())
    }
}

#[test]
fn test_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = StatusListBuilder::new(1)?;
    builder.add_status(StatusType::Valid);

    let status_list = builder.build()?;
    let serialized = status_list.to_json().unwrap();
    let decoded: Value = serde_json::from_str(&serialized)?;

    // Verify the bits field exists in the JSON
    assert!(decoded.get("bits").is_some());

    Ok(())
}

#[test]
fn test_json_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = StatusListBuilder::new(1)?;
    builder.add_status(StatusType::Valid);
    let status_list = builder.build()?;
    let serialized = status_list.to_json().unwrap();
    let _: Value = serde_json::from_str(&serialized)?;

    Ok(())
}

#[test]
fn test_json_serialization_2bit() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = StatusListBuilder::new(2)?;
    builder.add_status(StatusType::Valid);
    let status_list = builder.build()?;
    let serialized = status_list.to_json().unwrap();
    let _: Value = serde_json::from_str(&serialized)?;

    Ok(())
}

#[test]
fn test_complete_flow() -> Result<(), Box<dyn std::error::Error>> {
    // Test cases for each bit size
    let test_cases = [
        (
            1,
            vec![
                StatusType::Valid,
                StatusType::Invalid,
                StatusType::Valid,
                StatusType::Invalid,
                StatusType::Valid,
                StatusType::Invalid,
                StatusType::Valid,
                StatusType::Invalid,
            ],
        ),
        (
            1,
            vec![
                // From spec example
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
            ],
        ),
        (
            2,
            vec![
                StatusType::Valid,
                StatusType::Invalid,
                StatusType::Suspended,
                StatusType::ApplicationSpecific3,
            ],
        ),
        (
            2,
            vec![
                // From spec example
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
            ],
        ),
        (
            4,
            vec![
                StatusType::Valid,
                StatusType::Invalid,
                StatusType::Suspended,
                StatusType::ApplicationSpecific3,
            ],
        ),
        (
            8,
            vec![
                StatusType::Valid,
                StatusType::Invalid,
                StatusType::Suspended,
                StatusType::ApplicationSpecific3,
            ],
        ),
    ];

    for (bits_per_status, statuses) in test_cases {
        println!("\nTesting {}-bit encoding:", bits_per_status);

        // Build the status list
        let mut builder = StatusListBuilder::new(bits_per_status)?;
        for status in &statuses {
            builder.add_status(*status);
        }
        let status_list = builder.build()?;

        // Test encoding
        println!("Original statuses: {:?}", statuses);

        // Test decoding
        let decoder = StatusListDecoder::new(&status_list)?;
        for (i, expected_status) in statuses.iter().enumerate() {
            let decoded_status = decoder.get_status(i)?;
            println!("Status at position {}: {:?}", i, decoded_status);
            assert_eq!(&decoded_status, expected_status);
        }

        // Test serialization
        let status_list = builder.build()?;
        let serialized = status_list.to_json().unwrap();
        println!("Serialized: {}", serialized);

        // Test deserialization
        let decoded: Value = serde_json::from_str(&serialized)?;
        assert!(decoded.get("bits").is_some());
        println!("Decoded lst: {:?}", decoded.get("lst").unwrap());
        // Verify bits field matches
        if let Some(bits) = decoded.get("bits").and_then(|v| v.as_str()) {
            println!("Bits field: {}", bits);
        }

        // Verify we can't access out of bounds
        assert!(decoder.get_status(statuses.len()).is_err());
    }

    Ok(())
}
