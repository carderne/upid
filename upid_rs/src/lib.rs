//! # upid
//!
//! Rust implementation of UPID, an alternative to UUID and ULID
//! that includes a useful four-character prefix.
//!
//! It is still stored as a `u128` binary, is still sortable by date,
//! and has 64 bits of randomness. It uses a modified form of
//! Crockford's base32 alphabet that uses lower-case and prioritises
//! letters so that any four-letter alpha prefix can be specified.
//!
//! ## Quickstart
//!
//! ```rust
//! use upid::Upid;
//! let upid = Upid::new("user");
//!
//! let text = upid.to_string();
//!
//! let same = Upid::from_string(&text);
//! assert_eq!(upid, same.unwrap());
//! ```
//!
//! If an invalid prefix is specified, it will be handled as follows:
//! - invalid letters (not in the [`ENCODE`] alphabet) replaced by 'z'
//! - too short will be right-padded with 'z'
//! - too long will be clipped to four characters
//! ```rust
//! use upid::Upid;
//! let upid = Upid::new("00");
//! assert_eq!(upid.prefix(), "zzzz");
//! ```

// The code below is derived from the following:
// https://github.com/dylanhart/ulid-rs

mod b32;
#[cfg(feature = "uuid")]
mod uuid;

pub use crate::b32::{DecodeError, ENCODE};

use std::fmt;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use rand::Rng;

const VERSION: &str = "a";

fn now() -> std::time::SystemTime {
    std::time::SystemTime::now()
}

/// A Upid is a unique 128-bit identifier is sortable and has a useful prefix.
///
/// It is encoded as a 26 character string using a custom base32 alphabet based
/// on Crockford's, but with lower-case and prioritising letters over numerals.
/// In the binary, the first 40 bits are a unix timestamp with 256ms precision,
/// the next 64 are random bits, and the last 24 are the prefix and version identifier.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Upid(pub u128);

impl Upid {
    /// Creates a new Upid with the provided prefix and current time (UTC)
    ///
    /// The prefix should only contain lower-case latin alphabet characters.
    /// # Example
    /// ```rust
    /// use upid::Upid;
    ///
    /// let my_upid = Upid::new("user");
    /// ```
    pub fn new(prefix: &str) -> Upid {
        Upid::from_prefix(prefix)
    }

    /// Creates a Upid with the provided prefix and current time (UTC)
    ///
    /// The prefix should contain four lower-case latin alphabet characters.
    /// # Example
    /// ```rust
    /// use upid::Upid;
    ///
    /// let my_upid = Upid::from_prefix("user");
    /// ```
    pub fn from_prefix(prefix: &str) -> Upid {
        Upid::from_prefix_and_datetime(prefix, now())
    }

    /// Creates a new Upid with the given prefix and datetime
    ///
    /// The prefix should only contain lower-case latin alphabet characters.
    ///
    /// This will take the maximum of the `[SystemTime]` argument and `[SystemTime::UNIX_EPOCH]`
    /// as earlier times are not valid for a Upid timestamp
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use upid::Upid;
    ///
    /// let upid = Upid::from_prefix_and_datetime("user", SystemTime::now());
    /// ```
    pub fn from_prefix_and_datetime(prefix: &str, datetime: SystemTime) -> Upid {
        let milliseconds = datetime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        Upid::from_prefix_and_milliseconds(prefix, milliseconds)
    }

    /// Creates a new Upid with the given prefix and timestamp in millisecons
    ///
    /// The prefix should only contain lower-case latin alphabet characters.
    ///
    /// # Example
    /// ```rust
    /// use upid::Upid;
    ///
    /// let ms: u128 = 1720568902000;
    /// let upid = Upid::from_prefix_and_milliseconds("user", ms);
    /// ```
    pub fn from_prefix_and_milliseconds(prefix: &str, milliseconds: u128) -> Upid {
        // cut off the 8 lsb drops precision to 256 ms
        // future version could play with this differently
        // eg drop 4 bits on each side
        let time_bits = milliseconds >> 8;

        // get 64 bits of randomness on lsb side of a u128
        let mut source = rand::thread_rng();
        let random = source.gen::<u64>() as u128;

        // pad with 'z' if shorter than 4, cut to 4 if longer
        let prefix = format!("{:z<4}", prefix);
        let prefix: String = prefix.chars().take(4).collect();
        let prefix = format!("{}{}", prefix, VERSION);

        // decode_prefix Errors if the last character is past 'j' in the b32 alphabet
        // and we control that with the VERSION variable
        // If the prefix has characters from outside the alphabet, they will be wrapped into 'z's
        // And we have ensured above that it is exactly 5 characters long
        let p = b32::decode_prefix(prefix.as_bytes())
            .expect("decode_prefix failed with version character overflow");

        let res = (time_bits << 88)
            | (random << 24)
            | ((p[0] as u128) << 16)
            | ((p[1] as u128) << 8)
            | p[2] as u128;

        Upid(res)
    }

    /// Creates a Upid from a Base32 encoded string
    ///
    /// # Example
    /// ```rust
    /// use upid::Upid;
    ///
    /// let text = "user_aaccvpp5guht4dts56je5a";
    /// let result = Upid::from_string(text);
    ///
    /// assert_eq!(&result.unwrap().to_string(), text);
    /// ```
    pub fn from_string(encoded: &str) -> Result<Upid, DecodeError> {
        match b32::decode(encoded) {
            Ok(int_val) => Ok(Upid(int_val)),
            Err(err) => Err(err),
        }
    }

    /// Gets the datetime of when this Upid was created accurate to around 256ms
    ///
    /// # Example
    /// ```rust
    /// use std::time::{SystemTime, Duration};
    /// use upid::Upid;
    ///
    /// let dt = SystemTime::now();
    /// let upid = Upid::from_prefix_and_datetime("user", dt);
    ///
    /// assert!(dt + Duration::from_millis(257) >= upid.datetime());
    /// ```
    pub fn datetime(&self) -> SystemTime {
        let stamp = self.milliseconds();
        SystemTime::UNIX_EPOCH + Duration::from_millis(stamp)
    }

    /// Gets the prefix of this upid
    ///
    /// # Example
    /// ```rust
    /// use upid::Upid;
    ///
    /// let prefix = "user";
    /// let upid = Upid::from_prefix(prefix);
    ///
    /// assert_eq!(upid.prefix(), prefix);
    /// ```
    pub fn prefix(&self) -> String {
        let bytes: [u8; 16] = self.0.to_be_bytes();
        let (prefix, _) = b32::encode_prefix(&bytes[b32::END_RANDO_BIN..]);
        prefix
    }

    /// Gets the timestamp section of this upid
    ///
    /// # Example
    /// ```rust
    /// use upid::Upid;
    ///
    /// let ms: u128 = 1720568902000;
    /// let upid = Upid::from_prefix_and_milliseconds("user", ms);
    ///
    /// assert!(ms - u128::from(upid.milliseconds()) < 257);
    /// ```
    pub const fn milliseconds(&self) -> u64 {
        ((self.0 >> 88) << 8) as u64
    }

    /// Creates a Base32 encoded string that represents this Upid
    ///
    /// # Example
    /// ```rust
    /// use upid::Upid;
    ///
    /// let text = "user_aaccvpp5guht4dts56je5a";
    /// let upid = Upid::from_string(text).unwrap();
    ///
    /// assert_eq!(&upid.to_string(), text);
    /// ```
    #[allow(clippy::inherent_to_string_shadow_display)] // Significantly faster than Display::to_string
    pub fn to_string(&self) -> String {
        b32::encode(self.0)
    }

    /// Creates a Upid using the provided bytes array.
    ///
    /// # Example
    /// ```rust
    /// use upid::Upid;
    /// let bytes = [0xFF; 16];
    ///
    /// let upid = Upid::from_bytes(bytes);
    /// ```
    pub const fn from_bytes(bytes: [u8; 16]) -> Upid {
        Self(u128::from_be_bytes(bytes))
    }

    /// Returns the bytes of the Upid in big-endian order.
    ///
    /// # Example
    /// ```rust
    /// use upid::Upid;
    ///
    /// let text = "user_aaccvpp5guht4dts56je5a";
    /// let upid = Upid::from_string(text).unwrap();
    /// ```
    pub const fn to_bytes(&self) -> [u8; 16] {
        self.0.to_be_bytes()
    }
}

impl Default for Upid {
    fn default() -> Self {
        Upid::new("")
    }
}

impl From<Upid> for String {
    fn from(upid: Upid) -> String {
        upid.to_string()
    }
}

impl From<u128> for Upid {
    fn from(value: u128) -> Upid {
        Upid(value)
    }
}

impl From<Upid> for u128 {
    fn from(upid: Upid) -> u128 {
        upid.0
    }
}

impl FromStr for Upid {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Upid::from_string(s)
    }
}

impl fmt::Display for Upid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: u128 = 257;

    #[test]
    fn can_into_thing() {
        let want = Upid::from_str("user_aaccvpp5guht4dts56je5a").unwrap();
        let s: String = want.into();
        let u: u128 = want.into();

        assert_eq!(Upid::from_str(&s).unwrap(), want);
        assert_eq!(Upid::from(u), want);
    }

    #[test]
    fn can_display_things() {
        println!("{}", DecodeError::InvalidLength);
        println!("{}", DecodeError::InvalidChar);
    }

    #[test]
    fn test_dynamic() {
        let upid = Upid::new("user");
        let encoded = upid.to_string();
        let upid2 = Upid::from_string(&encoded).expect("failed to deserialize");
        assert_eq!(upid, upid2);
    }

    #[test]
    fn test_order() {
        let dt = SystemTime::now();
        let upid1 = Upid::from_prefix_and_datetime("user", dt);
        let upid2 = Upid::from_prefix_and_datetime("user", dt + Duration::from_millis(EPS as u64));
        assert!(upid1 < upid2);
    }

    #[test]
    fn test_timestamp() {
        let dt = SystemTime::now();
        let want = dt
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let upid = Upid::from_prefix_and_milliseconds("user", want);
        let got = u128::from(upid.milliseconds());

        assert!(want - got < EPS);
    }

    #[test]
    fn test_datetime() {
        let dt = SystemTime::now();
        let upid = Upid::from_prefix_and_datetime("user", dt);

        assert!(upid.datetime() <= dt);
        assert!(upid.datetime() + Duration::from_millis(EPS as u64) >= dt);
    }

    #[test]
    fn test_invalid_prefix() {
        // Invalid characters just become 'zzzz'
        let want = "zzzz";

        // even if too long
        let got = Upid::from_prefix("[0#/]]1,").prefix();
        assert_eq!(got, want);

        // or too short
        let got = Upid::from_prefix("[0").prefix();
        assert_eq!(got, want);
    }
}
