use core::fmt;

// Note the binary order is TIMESTAMP_RANDO_PREFIX+VERSION
const TIME_BIN_LEN: usize = 5;
const RANDO_BIN_LEN: usize = 8;
pub const END_RANDO_BIN: usize = 13;
const PREFIX_BIN_LEN: usize = 3; // includes version

// But the string order is PREFIX_TIME_RANDO_VERSION
const PREFIX_CHAR_LEN: usize = 4; // excluding the version char
const TIME_CHAR_LEN: usize = 8;
const END_TIME_CHAR: usize = 12;
const RANDO_CHAR_LEN: usize = 13;
const VERSION_CHAR_LEN: usize = 1;

/// Length of a string-encoded Upid
const CHAR_LEN: usize = 26;

/// 32-character alphabet modified from Crockford's
/// Numbers first for sensible sorting, but full lower-case
/// latin alphabet so any sensible prefix can be used
/// Effectively a mapping from 8 bit byte -> 5 bit int -> base32 character
pub const ENCODE: &[u8; 32] = b"234567abcdefghijklmnopqrstuvwxyz";

/// Speedy O(1) inverse lookup
/// base32 char -> ascii byte int -> base32 alphabet index
const DECODE: [u8; 256] = [
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
    31, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255,
];

/// Encodes the provided binary data to a base32 String
pub fn encode(binary: u128) -> String {
    let bytes: [u8; 16] = binary.to_be_bytes();
    let time = encode_time(&bytes[0..TIME_BIN_LEN]);
    let rando = encode_rando(&bytes[TIME_BIN_LEN..END_RANDO_BIN]);
    let (prefix, version) = encode_prefix(&bytes[END_RANDO_BIN..]);
    let out = format!("{}_{}{}{}", prefix, time, rando, version);
    out
}

/// Encodes the prefix portion of binary data to the prefix and version Strings
///
/// This process goes from 24 bits `[u8; 3]` to 25 bits (5 base32 characters)
/// so a 0 bit is implicitly padded to the lsb
pub fn encode_prefix(binary: &[u8]) -> (String, String) {
    let buffer_prefix: [u8; PREFIX_CHAR_LEN] = [
        ENCODE[((binary[0] & 248) >> 3) as usize],
        ENCODE[(((binary[0] & 7) << 2) | ((binary[1] & 192) >> 6)) as usize],
        ENCODE[((binary[1] & 62) >> 1) as usize],
        ENCODE[(((binary[1] & 1) << 4) | ((binary[2] & 240) >> 4)) as usize],
    ];
    let buffer_version: [u8; VERSION_CHAR_LEN] = [
        ENCODE[(binary[2] & 15) as usize], // implicitly "add" a 0 bit
    ];
    let prefix = String::from_utf8(buffer_prefix.to_vec())
        .expect("unexpected failure in base32 encode for upid");
    let version = String::from_utf8(buffer_version.to_vec())
        .expect("unexpected failure in base32 encode for upid");
    (prefix, version)
}

/// Encodes the time portion of binary data to a base32 String
///
/// Unlike the prefix, this has 1:1 bit mapping with 40 bits
fn encode_time(binary: &[u8]) -> String {
    let buffer: [u8; TIME_CHAR_LEN] = [
        ENCODE[((binary[0] & 248) >> 3) as usize],
        ENCODE[(((binary[0] & 7) << 2) | ((binary[1] & 192) >> 6)) as usize],
        ENCODE[((binary[1] & 62) >> 1) as usize],
        ENCODE[(((binary[1] & 1) << 4) | ((binary[2] & 240) >> 4)) as usize],
        ENCODE[(((binary[2] & 15) << 1) | ((binary[3] & 128) >> 7)) as usize],
        ENCODE[((binary[3] & 124) >> 2) as usize],
        ENCODE[(((binary[3] & 3) << 3) | ((binary[4] & 224) >> 5)) as usize],
        ENCODE[(binary[4] & 31) as usize],
    ];
    String::from_utf8(buffer.to_vec()).expect("unexpected failure in base32 encode for upid")
}

/// Encodes the randomness portion of binary data to a base32 String
///
/// This process goes from 64 bits `[u8; 8]` to 65 bits (13 base32 characters)
/// so a 0 bit is implicitly padded to the lsb
fn encode_rando(binary: &[u8]) -> String {
    let buffer: [u8; RANDO_CHAR_LEN] = [
        ENCODE[((binary[0] & 248) >> 3) as usize],
        ENCODE[(((binary[0] & 7) << 2) | ((binary[1] & 192) >> 6)) as usize],
        ENCODE[((binary[1] & 62) >> 1) as usize],
        ENCODE[(((binary[1] & 1) << 4) | ((binary[2] & 240) >> 4)) as usize],
        ENCODE[(((binary[2] & 15) << 1) | ((binary[3] & 128) >> 7)) as usize],
        ENCODE[((binary[3] & 124) >> 2) as usize],
        ENCODE[(((binary[3] & 3) << 3) | ((binary[4] & 224) >> 5)) as usize],
        ENCODE[(binary[4] & 31) as usize],
        ENCODE[((binary[5] & 248) >> 3) as usize],
        ENCODE[(((binary[5] & 7) << 2) | ((binary[6] & 192) >> 6)) as usize],
        ENCODE[((binary[6] & 62) >> 1) as usize],
        ENCODE[(((binary[6] & 1) << 4) | ((binary[7] & 240) >> 4)) as usize],
        ENCODE[(binary[7] & 15) as usize], // implicitly "add" a 0 bit
    ];
    String::from_utf8(buffer.to_vec()).expect("unexpected failure in base32 encode for upid")
}

/// An error that can occur when decoding a base32 string
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum DecodeError {
    /// The length of the string does not match the expected length
    InvalidLength,
    /// A non-base32 character was found
    InvalidChar,
    /// Text representation overflows random or prefix chunks
    Overflow,
}

impl std::error::Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match *self {
            DecodeError::InvalidLength => "invalid length",
            DecodeError::InvalidChar => "invalid character",
            DecodeError::Overflow => "overflow",
        };
        write!(f, "{}", text)
    }
}

/// Decodes the encoded string to u128 binary
///
/// Decoding is fallible and will return a `DecodeError` if the string
/// is too long or includes characters outside the alphabet. This means
/// all upstream functions also need to return `Result`.
///
/// A future API might add an infallible version.
pub fn decode(encoded: &str) -> Result<u128, DecodeError> {
    let encoded = encoded.replace('_', "");
    if encoded.len() != CHAR_LEN {
        return Err(DecodeError::InvalidLength);
    }

    if encoded.bytes().any(|b| !ENCODE.contains(&b)) {
        return Err(DecodeError::InvalidChar);
    }
    let bytes: &[u8] = encoded.as_bytes();

    let prefix_bytes: Vec<u8> = [&bytes[0..PREFIX_CHAR_LEN], &[bytes[bytes.len() - 1]]].concat();

    let prefix = decode_prefix(&prefix_bytes)?;
    let time = decode_time(&bytes[PREFIX_CHAR_LEN..END_TIME_CHAR])?;
    let rando = decode_rando(&bytes[END_TIME_CHAR..bytes.len() - 1])?;

    let mut result: u128 = 0;
    for (shift, &byte) in time
        .iter()
        .chain(rando.iter())
        .chain(prefix.iter())
        .enumerate()
    {
        result |= (byte as u128) << ((15 - shift) * 8);
    }
    Ok(result)
}

/// Decodes the prefix and version character bytes into binary
///
/// As this process goes from 25 -> 24 bits, there can be overflow.
/// For the last character, only the first half of the alphabet is allowed
/// (4 bits rather than the usual 5).
pub fn decode_prefix(encoded: &[u8]) -> Result<[u8; PREFIX_BIN_LEN], DecodeError> {
    if DECODE[encoded[encoded.len() - 1] as usize] > 15 {
        return Err(DecodeError::Overflow);
    }

    let buffer: [u8; PREFIX_BIN_LEN] = [
        ((DECODE[encoded[0] as usize] << 3) | (DECODE[encoded[1] as usize] >> 2)),
        ((DECODE[encoded[1] as usize] << 6)
            | (DECODE[encoded[2] as usize] << 1)
            | (DECODE[encoded[3] as usize] >> 4)),
        ((DECODE[encoded[3] as usize] << 4) | (DECODE[encoded[4] as usize] & 15)),
        // lose 1 bit of data
    ];
    Ok(buffer)
}

/// Decodes the time characters into binary.
///
/// This cannot fail (if called correctly) but returns `Result` to be consistent
/// with its peers
fn decode_time(encoded: &[u8]) -> Result<[u8; TIME_BIN_LEN], DecodeError> {
    let buffer: [u8; TIME_BIN_LEN] = [
        ((DECODE[encoded[0] as usize] << 3) | (DECODE[encoded[1] as usize] >> 2)),
        ((DECODE[encoded[1] as usize] << 6)
            | (DECODE[encoded[2] as usize] << 1)
            | (DECODE[encoded[3] as usize] >> 4)),
        ((DECODE[encoded[3] as usize] << 4) | (DECODE[encoded[4] as usize] >> 1)),
        ((DECODE[encoded[4] as usize] << 7)
            | (DECODE[encoded[5] as usize] << 2)
            | (DECODE[encoded[6] as usize] >> 3)),
        ((DECODE[encoded[6] as usize] << 5) | (DECODE[encoded[7] as usize])),
    ];
    Ok(buffer)
}

/// Decodes the randomness character bytes into binary
///
/// As this process goes from 65 -> 64 bits, there can be overflow.
/// For the last character, only the first half of the alphabet is allowed
/// (4 bits rather than the usual 5).
fn decode_rando(encoded: &[u8]) -> Result<[u8; RANDO_BIN_LEN], DecodeError> {
    if DECODE[encoded[encoded.len() - 1] as usize] > 15 {
        return Err(DecodeError::Overflow);
    }

    let buffer: [u8; RANDO_BIN_LEN] = [
        ((DECODE[encoded[0] as usize] << 3) | (DECODE[encoded[1] as usize] >> 2)),
        ((DECODE[encoded[1] as usize] << 6)
            | (DECODE[encoded[2] as usize] << 1)
            | (DECODE[encoded[3] as usize] >> 4)),
        ((DECODE[encoded[3] as usize] << 4) | (DECODE[encoded[4] as usize] >> 1)),
        ((DECODE[encoded[4] as usize] << 7)
            | (DECODE[encoded[5] as usize] << 2)
            | (DECODE[encoded[6] as usize] >> 3)),
        ((DECODE[encoded[6] as usize] << 5) | (DECODE[encoded[7] as usize])),
        ((DECODE[encoded[8] as usize] << 3) | (DECODE[encoded[9] as usize] >> 2)),
        ((DECODE[encoded[9] as usize] << 6)
            | (DECODE[encoded[10] as usize] << 1)
            | (DECODE[encoded[11] as usize] >> 4)),
        ((DECODE[encoded[11] as usize] << 4) | (DECODE[encoded[12] as usize] & 15)),
        // lose 1 bit of data
    ];
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    const EPS: u128 = 256;

    /// Generator code for `DECODE`
    #[cfg(test)]
    #[test]
    fn test_lookup_table() {
        let mut lookup = [255; 256];
        for (i, &c) in ENCODE.iter().enumerate() {
            lookup[c as usize] = i as u8;
        }
        assert_eq!(DECODE, lookup);
    }

    fn time_as128(array: &[u8; 5]) -> u128 {
        ((array[0] as u128) << (64 + 24 + 32))
            + ((array[1] as u128) << (64 + 24 + 24))
            + ((array[2] as u128) << (64 + 24 + 16))
            + ((array[3] as u128) << (64 + 24 + 8))
            + ((array[4] as u128) << (64 + 24 + 0))
    }

    #[test]
    fn test_encode_decode() {
        let timestamp: u128 = 1720560233826;
        let time_bits = timestamp >> 1;
        let random: u64 = 1218987987987368123;
        let upid = (time_bits << 88) | ((random as u128) << 24) | (5 << 16) | (5 << 8) | 5;
        let text = encode(upid);
        let end = decode(&text).unwrap();
        assert!(end == upid);
    }

    #[test]
    fn test_encode_decode_time() {
        let timestamp: u128 = 1720560233826;
        let time_bits = timestamp >> 1;
        let t_in = (time_bits << 88).to_be_bytes();
        let enc = encode_time(&t_in);
        let tout = decode_time(&enc.as_bytes()).unwrap();
        let final_t = (time_as128(&tout) >> 88) << 1;
        assert!(timestamp - final_t < EPS);
    }
}
