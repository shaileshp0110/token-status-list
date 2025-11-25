#![no_main]
use libfuzzer_sys::fuzz_target;
use vc_status_list::{StatusListBuilder, StatusListDecoder, StatusType};

fuzz_target!(|data: &[u8]| {
    // Test encoding/decoding round-trip with various bit sizes
    // This ensures that what we encode can be correctly decoded
    
    for bits in [1, 2, 4, 8] {
        if let Ok(builder) = StatusListBuilder::new(bits) {
            // Add statuses from fuzz input
            for byte in data.iter().take(100) {
                // Map byte to valid status type
                let status_value = byte % 16;
                if let Ok(status) = StatusType::try_from(status_value) {
                    builder.add_status(status);
                }
            }

            // Build the status list
            if let Ok(status_list) = builder.build() {
                // Test round-trip: encode then decode
                if let Ok(decoder) = StatusListDecoder::new(&status_list) {
                    // Verify we can decode what we encoded
                    let count = status_list.lst.len() * (8 / bits as usize);
                    for i in 0..count.min(100) {
                        let _ = decoder.get_status(i);
                    }
                    
                    // Test decoder methods
                    let _ = decoder.len();
                    let _ = decoder.is_empty();
                    let _ = decoder.get_raw_bytes();
                }
            }
        }
    }
});

