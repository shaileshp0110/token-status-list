#![no_main]
use libfuzzer_sys::fuzz_target;
use vc_status_list::{StatusList, StatusListBuilder, StatusType};

fuzz_target!(|data: &[u8]| {
    // Test JSON and CBOR serialization/deserialization with various inputs
    
    // Create a status list from fuzz data
    for bits in [1, 2, 4, 8] {
        if let Ok(builder) = StatusListBuilder::new(bits) {
            // Add statuses from fuzz input
            for byte in data.iter().take(100) {
                let status_value = byte % 16;
                if let Ok(status) = StatusType::try_from(status_value) {
                    builder.add_status(status);
                }
            }
            
            if let Ok(status_list) = builder.build() {
                // Test JSON serialization
                if let Ok(json_str) = status_list.to_json() {
                    // Try to parse it back (if we had a from_json method)
                    // For now, just ensure serialization doesn't panic
                }
                
                // Test CBOR serialization
                if let Ok(cbor_hex) = status_list.to_cbor() {
                    // CBOR is returned as hex string
                    // Ensure it's valid hex and doesn't panic
                }
            }
        }
    }
    
    // Test serialization with edge cases
    // Empty status list
    for bits in [1, 2, 4, 8] {
        if let Ok(builder) = StatusListBuilder::new(bits) {
            if let Ok(status_list) = builder.build() {
                let _ = status_list.to_json();
                let _ = status_list.to_cbor();
            }
        }
    }
    
    // Test with very large compressed data
    if data.len() > 0 {
        let status_list = StatusList {
            bits: 1,
            lst: data.to_vec(),
            aggregation_uri: None,
        };
        
        // Serialization should handle large data gracefully
        let _ = status_list.to_json();
        let _ = status_list.to_cbor();
    }
});

