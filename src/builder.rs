use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::encoder::StatusListEncoder;
use crate::error::{BuilderError, StatusTypeError};
use crate::types::{BitsPerStatus, StatusList, StatusType};

#[derive(Debug)]
pub struct StatusListBuilder {
    statuses: Mutex<Vec<StatusType>>,
    bits_per_status: u8,
    last_index: AtomicUsize,
    encoder: StatusListEncoder,
}

impl StatusListBuilder {
    pub fn new(bits_per_status: u8) -> Result<Self, StatusTypeError> {
        BitsPerStatus::try_from(bits_per_status)?;

        Ok(Self {
            statuses: Mutex::new(Vec::new()),
            bits_per_status,
            last_index: AtomicUsize::new(0),
            encoder: StatusListEncoder::new(bits_per_status),
        })
    }

    pub fn from_vec(
        statuses: Vec<StatusType>,
        bits_per_status: u8,
    ) -> Result<Self, StatusTypeError> {
        BitsPerStatus::try_from(bits_per_status)?;

        let last_index = if !statuses.is_empty() {
            statuses.len() - 1
        } else {
            0
        };

        Ok(Self {
            statuses: Mutex::new(statuses),
            bits_per_status,
            last_index: AtomicUsize::new(last_index),
            encoder: StatusListEncoder::new(bits_per_status),
        })
    }

    pub fn add_status(&self, status: StatusType) -> &Self {
        let mut statuses = self.statuses.lock().unwrap();
        let index = statuses.len();

        statuses.push(status);
        self.last_index.store(index, Ordering::SeqCst);
        self
    }

    pub fn get_last_index(&self) -> Option<usize> {
        let index = self.last_index.load(Ordering::SeqCst);
        if index == 0 && self.statuses.lock().unwrap().is_empty() {
            None
        } else {
            Some(index)
        }
    }

    pub fn get_bits_per_status(&self) -> u8 {
        self.bits_per_status
    }

    pub fn build(&self) -> Result<StatusList, BuilderError> {
        let statuses = self.statuses.lock().unwrap();
        let bytes = self.encoder.encode_statuses(&statuses)?;
        self.encoder.finalize(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_thread_safety() {
        let builder = StatusListBuilder::new(2).unwrap();
        let builder_arc = std::sync::Arc::new(builder);
        let mut handles = vec![];

        for _ in 0..10 {
            let builder_clone = builder_arc.clone();
            let handle = thread::spawn(move || {
                builder_clone.add_status(StatusType::Valid);
                builder_clone.add_status(StatusType::Invalid);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let statuses = builder_arc.statuses.lock().unwrap();
        assert_eq!(statuses.len(), 20); // 10 threads * 2 statuses each
    }

    #[test]
    fn test_from_vec_constructor() {
        let statuses = vec![
            StatusType::Invalid,              // 1 - index 0
            StatusType::Suspended,            // 2 - index 1
            StatusType::Valid,                // 0 - index 2
            StatusType::ApplicationSpecific3, // 3 - index 3
            StatusType::Valid,                // 0 - index 4
            StatusType::Invalid,              // 1 - index 5
            StatusType::Valid,                // 0 - index 6
            StatusType::Invalid,              // 1 - index 7
            StatusType::Invalid,              // 1 - index 8
            StatusType::Suspended,            // 2 - index 9
            StatusType::ApplicationSpecific3, // 3 - index 10
            StatusType::ApplicationSpecific3, // 3 - index 11
        ];
        let bits_per_status = 2;

        let builder = StatusListBuilder::from_vec(statuses.clone(), bits_per_status).unwrap();

        assert_eq!(builder.bits_per_status, bits_per_status);
        assert_eq!(*builder.statuses.lock().unwrap(), statuses);
        assert_eq!(builder.last_index.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn test_from_vec_invalid_bits() {
        let statuses = vec![
            StatusType::Valid,
            StatusType::Invalid,
            StatusType::Suspended,
            StatusType::ApplicationSpecific3,
        ];
        let invalid_bits = 3;

        assert!(StatusListBuilder::from_vec(statuses, invalid_bits).is_err());
    }

    #[test]
    fn test_different_bit_sizes() {
        let one_bit_statuses = vec![
            StatusType::Valid,
            StatusType::Invalid,
            StatusType::Valid,
            StatusType::Invalid,
            StatusType::Valid,
            StatusType::Invalid,
            StatusType::Valid,
            StatusType::Invalid,
        ];
        let builder = StatusListBuilder::from_vec(one_bit_statuses.clone(), 1).unwrap();
        assert_eq!(builder.last_index.load(Ordering::SeqCst), 7);

        let two_bit_statuses = vec![
            StatusType::Valid,
            StatusType::Invalid,
            StatusType::Suspended,
            StatusType::ApplicationSpecific3,
        ];
        let builder = StatusListBuilder::from_vec(two_bit_statuses.clone(), 2).unwrap();
        assert_eq!(builder.last_index.load(Ordering::SeqCst), 3);

        let four_bit_statuses = vec![StatusType::Valid, StatusType::Invalid];
        let builder = StatusListBuilder::from_vec(four_bit_statuses.clone(), 4).unwrap();
        assert_eq!(builder.last_index.load(Ordering::SeqCst), 1);

        let eight_bit_statuses = vec![StatusType::Valid];
        let builder = StatusListBuilder::from_vec(eight_bit_statuses.clone(), 8).unwrap();
        assert_eq!(builder.last_index.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_add_status() {
        let builder = StatusListBuilder::new(2).unwrap();

        builder.add_status(StatusType::Valid);
        builder.add_status(StatusType::Invalid);
        builder.add_status(StatusType::Suspended);
        builder.add_status(StatusType::ApplicationSpecific3);

        assert_eq!(builder.last_index.load(Ordering::SeqCst), 3);
        assert_eq!(builder.statuses.lock().unwrap().len(), 4);
    }

    #[test]
    fn test_builder_invalid_bits_per_status() {
        let invalid_bits = [0, 3, 5, 6, 7, 9, 16];
        for bits in invalid_bits {
            match StatusListBuilder::new(bits) {
                Err(StatusTypeError::InvalidBitsPerStatus(val)) => {
                    assert_eq!(val, bits);
                    assert_eq!(
                        StatusTypeError::InvalidBitsPerStatus(val).to_string(),
                        format!(
                            "Invalid bits per status value: {}. Must be 1, 2, 4, or 8",
                            bits
                        )
                    );
                }
                _ => panic!("Expected InvalidBitsPerStatus error for {}", bits),
            }
        }
    }

    #[test]
    fn test_status_type_error_messages() {
        let error = StatusListBuilder::new(3).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Invalid bits per status value: 3. Must be 1, 2, 4, or 8"
        );

        let error = StatusListBuilder::from_vec(vec![StatusType::Valid], 3).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Invalid bits per status value: 3. Must be 1, 2, 4, or 8"
        );
    }
    #[test]
    fn test_spec_example() {
        let statuses = vec![
            StatusType::Invalid,
            StatusType::Valid,
            StatusType::Valid,
            StatusType::Invalid,
            StatusType::Invalid,
            StatusType::Invalid,
            StatusType::Valid,
            StatusType::Invalid,
            StatusType::Invalid,
            StatusType::Invalid,
            StatusType::Valid,
            StatusType::Valid,
            StatusType::Valid,
            StatusType::Invalid,
            StatusType::Valid,
            StatusType::Invalid,
        ];
        let builder = StatusListBuilder::from_vec(statuses, 1).unwrap();
        let status_list = builder.build().unwrap();

        let json = status_list.to_json().unwrap();
        assert_eq!(json, r#"{"bits":1,"lst":"eNrbuRgAAhcBXQ"}"#);

        let cbor = status_list.to_cbor().unwrap();
        assert_eq!(cbor, "a2646269747301636c73744a78dadbb918000217015d");
    }
}
