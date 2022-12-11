//! Versioning utils

/// The version of the game's protocol.
///
/// Used to ensure that the mod and the game are using the same protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    /// The hash of the protocol name. Not perfect, but should ensure mods aren't used with the wrong game.
    pub name_hash: u16,

    /// The major version of the protocol.
    pub major: u16,

    /// The minor version of the protocol.
    pub minor: u16,

    /// The patch version of the protocol.
    pub patch: u16,
}

impl Version {
    /// Convert the Version into a u64.
    pub fn to_u64(&self) -> u64 {
        let mut result: u64 = 0;
        result |= self.name_hash as u64;
        result |= (self.major as u64) << 16;
        result |= (self.minor as u64) << 32;
        result |= (self.patch as u64) << 48;
        result
    }

    /// Convert a u64 into a Version.
    pub fn from_u64(version: u64) -> Self {
        Self {
            name_hash: (version & 0xFFFF) as u16,
            major: ((version >> 16) & 0xFFFF) as u16,
            minor: ((version >> 32) & 0xFFFF) as u16,
            patch: ((version >> 48) & 0xFFFF) as u16,
        }
    }
}

/// Generate a new Version from the current crate's version.
#[macro_export]
macro_rules! version(
    () => (
        Version {
            name_hash: $crate::version::__str_hash(env!("CARGO_PKG_NAME")),
            major: $crate::version::__str_to_u16(env!("CARGO_PKG_VERSION_MAJOR")),
            minor: $crate::version::__str_to_u16(env!("CARGO_PKG_VERSION_MINOR")),
            patch: $crate::version::__str_to_u16(env!("CARGO_PKG_VERSION_PATCH")),
        }
    )
);

#[doc(hidden)]
pub const fn __str_to_u16(s: &str) -> u16 {
    let mut result: u16 = 0;
    let s = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        let digit: u32 = s.len() as u32 - i as u32;
        result += (s[i] as u16 - 48) * 10u16.pow(digit);
        i += 1;
    }
    result
}

#[doc(hidden)]
pub const fn __str_hash(s: &str) -> u16 {
    // Based on Jenkins' one-at-a-time hash.
    let mut result: u16 = 0;
    let s = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        result = result.wrapping_add(s[i] as u16);
        result = result.wrapping_add(result << 10);
        result ^= result >> 6;
        i += 1;
    }

    result = result.wrapping_add(result << 3);
    result ^= result >> 11;
    result = result.wrapping_add(result << 15);

    result
}
