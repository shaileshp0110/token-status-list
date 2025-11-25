#![no_main]
use libfuzzer_sys::fuzz_target;
use vc_status_list::{StatusListBuilder, StatusType};

fuzz_target!(|data: &[u8]| {
    // Test builder with various inputs and edge cases
    
    // Test with different bit sizes
    for bits in [1, 2, 4, 8] {
        if let Ok(builder) = StatusListBuilder::new(bits) {
            // Add many statuses from fuzz input
            for byte in data.iter().take(1000) {
                let status_value = byte % 16;
                if let Ok(status) = StatusType::try_from(status_value) {
                    builder.add_status(status);
                }
            }
            
            // Try to build - should handle large inputs gracefully
            if let Ok(status_list) = builder.build() {
                // Test serialization with the built status list
                let _ = status_list.to_json();
                let _ = status_list.to_cbor();
            }
        }
    }
    
    // Test from_vec constructor with various inputs
    if data.len() > 0 {
        let mut statuses = Vec::new();
        for byte in data.iter().take(1000) {
            let status_value = byte % 16;
            if let Ok(status) = StatusType::try_from(status_value) {
                statuses.push(status);
            }
        }
        
        for bits in [1, 2, 4, 8] {
            if let Ok(builder) = StatusListBuilder::from_vec(statuses.clone(), bits) {
                let _ = builder.build();
                let _ = builder.get_last_index();
                let _ = builder.get_bits_per_status();
            }
        }
    }
});

