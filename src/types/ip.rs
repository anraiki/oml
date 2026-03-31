use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::error::OmlError;

/// A resolved IP address with addressable sub-properties.
///
/// Backed by RFC 791 (IPv4) and RFC 4291 (IPv6).
/// IPv6 display uses compressed form per RFC 5952 (provided by std).
///
/// **Note:** The canonical `host` key requires a valid IP address.
/// For hostname configuration (e.g. `host = "db.internal"`), use a key
/// with the `_host` suffix instead — invalid IPs silently fall back to
/// plain strings under heuristic resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmlIp {
    addr: IpAddr,
}

impl OmlIp {
    /// Parse for an **explicit** canonical key. Returns an error on invalid input.
    pub fn parse(input: &str, key: &str) -> Result<Self, OmlError> {
        Self::try_parse(input).ok_or_else(|| OmlError::ExplicitTypeMismatch {
            key: key.to_string(),
            type_name: "IP address",
            detail: format!("'{input}' is not a valid IPv4 or IPv6 address"),
        })
    }

    /// Parse for a **heuristic** key. Returns `None` on invalid input (silent fallback).
    pub fn try_parse(input: &str) -> Option<Self> {
        input.parse::<IpAddr>().ok().map(|addr| OmlIp { addr })
    }

    /// IP version: `4` or `6`.
    pub fn version(&self) -> u8 {
        match self.addr {
            IpAddr::V4(_) => 4,
            IpAddr::V6(_) => 6,
        }
    }

    /// IPv4 octets as `[u8; 4]`. `None` for IPv6 addresses.
    pub fn octets(&self) -> Option<[u8; 4]> {
        match self.addr {
            IpAddr::V4(v4) => Some(v4.octets()),
            IpAddr::V6(_) => None,
        }
    }

    /// IPv6 groups as `[u16; 8]`. `None` for IPv4 addresses.
    pub fn groups(&self) -> Option<[u16; 8]> {
        match self.addr {
            IpAddr::V4(_) => None,
            IpAddr::V6(v6) => Some(v6.segments()),
        }
    }

    pub fn is_loopback(&self) -> bool {
        self.addr.is_loopback()
    }

    /// Returns `true` for RFC 1918 (IPv4) or RFC 4193 / link-local (IPv6) ranges.
    pub fn is_private(&self) -> bool {
        match self.addr {
            IpAddr::V4(v4) => ipv4_is_private(&v4),
            IpAddr::V6(v6) => ipv6_is_private(&v6),
        }
    }

    pub fn inner(&self) -> &IpAddr {
        &self.addr
    }
}

fn ipv4_is_private(addr: &Ipv4Addr) -> bool {
    let o = addr.octets();
    // 10.0.0.0/8
    if o[0] == 10 {
        return true;
    }
    // 172.16.0.0/12
    if o[0] == 172 && (16..=31).contains(&o[1]) {
        return true;
    }
    // 192.168.0.0/16
    if o[0] == 192 && o[1] == 168 {
        return true;
    }
    // 127.0.0.0/8 loopback
    if o[0] == 127 {
        return true;
    }
    // 169.254.0.0/16 link-local
    if o[0] == 169 && o[1] == 254 {
        return true;
    }
    false
}

fn ipv6_is_private(addr: &Ipv6Addr) -> bool {
    let segs = addr.segments();
    // ::1 loopback
    if addr.is_loopback() {
        return true;
    }
    // fe80::/10 link-local
    if (segs[0] & 0xffc0) == 0xfe80 {
        return true;
    }
    // fc00::/7 unique-local (RFC 4193)
    if (segs[0] & 0xfe00) == 0xfc00 {
        return true;
    }
    false
}

impl std::fmt::Display for OmlIp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.addr)
    }
}
