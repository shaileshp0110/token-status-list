# Token Status List

A Rust implementation of the [Token Status List specification](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-status-list-06), which defines a mechanism for representing the status of tokens secured by JSON Object Signing and Encryption (JOSE) or CBOR Object Signing and Encryption (COSE).

## Overview

This library implements a compact and efficient way to manage token statuses using bit arrays. Key features include:

- Support for different bit-size encodings (1, 2, 4, or 8 bits per status)
- ZLIB compression of status lists
- Support for standard status types:
- `Valid` (0x00)
- `Invalid` (0x01)
- `Suspended` (0x02)
- `ApplicationSpecific3` (0x03)
- `ApplicationSpecific14` (0x0E)
- `ApplicationSpecific15` (0x0F)

## Usage

### Building a Status List

```rust
use token_status_list::{StatusListBuilder, StatusType};

let mut builder = StatusListBuilder::new(1)?; // 1 bit per status
builder
    .add_status(StatusType::Valid)
    .add_status(StatusType::Invalid);
let status_list = builder.build()?;
```

### Decoding a Status List

```rust
use token_status_list::StatusListDecoder;

let decoder = StatusListDecoder::new(&status_list)?;
let status = decoder.get_status(0)?; // Get status at index 0
assert_eq!(status, StatusType::Valid);
```

### Encoding Format

The status list uses a compact binary encoding format:

1. **Bit-Size Encoding**: Supports multiple bits per status:
   - 1-bit: Only Valid (0) and Invalid (1)
   - 2-bit: Supports up to 4 status types
   - 4-bit: Supports up to 16 status types
   - 8-bit: Supports up to 256 status types

2. **Compression**: 
   - Uses ZLIB compression (DEFLATE algorithm)
   - Applied after bit packing for efficient storage
   - Automatically handled during encoding/decoding

3. **Base64 Encoding**:
   - Final compressed data is encoded using base64url (no padding)
   - Safe for use in URLs and JSON documents

### Serialization and Deserialization

The following example illustrates the JSON representation of the Status List:
byte_array = [0xb9, 0xa3]
```json
{
    "bits": "1",
    "lst": "eNrbuRgAAhcBXQ"
}
```
The status list is serialized to JSON format with two main fields
Where:
- `bits`: The number of bits used per status (1, 2, 4, or 8)
- `lst`: The base64url-encoded, ZLIB-compressed status list

The following example illustrates the CBOR representation of the Status List in Hex:

byte_array = [0xb9, 0xa3]
```
encoded:
a2646269747301636c73744a78dadbb918000217015d
```

// Serialization
let status_list = builder.build()?;
let serialized = serde_json::to_string(&status_list)?;

// Deserialization
let decoded: StatusList = serde_json::from_str(&serialized)?;
let decoder = StatusListDecoder::new(&decoded)?;
```

```rust 
// Create a new status list with 2 bits per status
let mut builder = StatusListBuilder::new(2)?;
// Add some statuses
builder
.add_status(StatusType::Valid)
.add_status(StatusType::Invalid)
.add_status(StatusType::Suspended);
// Build the status list
let status_list = builder.build()?;
// Serialize to JSON
let json = status_list.to_json()?;
// Example output: {"bits":2,"lst":"eNpTAAEHAA"}
// Serialize to CBOR
let cbor = status_list.to_cbor()?;
// Example output: "a2646269747302636c73744978da636100070"

```

## Specification Compliance

This implementation follows the IETF draft specification for Token Status Lists, including:

- Bit-size restrictions (1, 2, 4, or 8 bits per status)
- ZLIB compression with DEFLATE
- Standard status type values
- JSON serialization format
- CBOR serialization format


## License

This project is licensed under the terms of the [LICENSE](LICENSE) file in the root directory.

[![PR Validation](https://github.com/shaileshp0110/token-status-list/actions/workflows/pr.yml/badge.svg)](https://github.com/shaileshp0110/token-status-list/actions/workflows/pr.yml)
[![codecov](https://codecov.io/gh/shaileshp0110/token-status-list/branch/main/graph/badge.svg)](https://codecov.io/gh/shaileshp0110/token-status-list)
