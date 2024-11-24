mod builder;
mod decoder;
mod encoder;
mod error;
mod types;

pub use builder::StatusListBuilder;
pub use decoder::StatusListDecoder;
pub use encoder::StatusListEncoder;
pub use error::{BuilderError, StatusTypeError};
pub use types::{BitsPerStatus, StatusList, StatusType};

#[cfg(test)]
mod tests;
