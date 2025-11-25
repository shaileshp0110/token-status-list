#![no_main]
use libfuzzer_sys::fuzz_target;
use vc_status_list::{StatusList, StatusListDecoder};

fuzz_target!(|data: &[u8]| {
    // Test decoder with completely random/malformed data
    // This is important for security - decoder should handle any input gracefully
    
    if data.len() >= 1 {
        // Test with various bit sizes
        for bits in [1, 2, 4, 8] {
            let status_list = StatusList {
                bits,
                lst: data.to_vec(),
                aggregation_uri: None,
            };
            
            // Decoder should handle errors gracefully, not panic
            if let Ok(decoder) = StatusListDecoder::new(&status_list) {
                // Try to read various indices - should handle out-of-bounds gracefully
                for i in 0..1000 {
                    let _ = decoder.get_status(i);
                }
            }
        }
    }
    
    // Test base64url decoder with random strings
    if data.len() > 0 {
        // Create a base64url-like string from the data
        // Use base64url encoding (the library uses base64url, not base64)
        let base64_str = base64url::encode(data);
        // Try to decode - should handle errors gracefully
        let _ = StatusListDecoder::new_from_base64(&base64_str);
    }
});

