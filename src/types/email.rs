use crate::error::OmlError;

/// A resolved email address with addressable sub-properties.
///
/// Backed by RFC 5321 / RFC 5322.
#[derive(Debug, Clone, PartialEq)]
pub struct OmlEmail {
    raw: String,
    local: String,
    domain: String,
}

impl OmlEmail {
    /// Parse for an **explicit** canonical key. Returns an error on invalid input.
    pub fn parse(input: &str, key: &str) -> Result<Self, OmlError> {
        Self::try_parse(input).ok_or_else(|| OmlError::ExplicitTypeMismatch {
            key: key.to_string(),
            type_name: "email address",
            detail: format!("'{input}' is not a valid email address"),
        })
    }

    /// Parse for a **heuristic** key. Returns `None` on invalid input (silent fallback).
    pub fn try_parse(input: &str) -> Option<Self> {
        // RFC 5321: local-part "@" domain
        // We use the last '@' to allow quoted local parts containing '@'.
        let at = input.rfind('@')?;
        if at == 0 || at == input.len() - 1 {
            return None;
        }
        let local = &input[..at];
        let domain = &input[at + 1..];

        // Local part: non-empty, no bare spaces (quoted forms not supported here)
        if local.is_empty() || local.contains(' ') {
            return None;
        }

        // Domain: must contain at least one dot, no leading/trailing dots,
        // no empty labels, only valid characters.
        if domain.is_empty()
            || !domain.contains('.')
            || domain.starts_with('.')
            || domain.ends_with('.')
            || domain.contains("..")
        {
            return None;
        }

        // Labels must be non-empty and contain only [a-zA-Z0-9-]
        for label in domain.split('.') {
            if label.is_empty()
                || label.starts_with('-')
                || label.ends_with('-')
                || !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                return None;
            }
        }

        Some(OmlEmail {
            raw: input.to_string(),
            local: local.to_string(),
            domain: domain.to_string(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// The part before `@`.
    pub fn local(&self) -> &str {
        &self.local
    }

    /// The part after `@`.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    // --- Mutation ---

    pub fn set_local(&mut self, local: &str) {
        self.local = local.to_string();
        self.raw = format!("{}@{}", self.local, self.domain);
    }

    pub fn set_domain(&mut self, domain: &str) {
        self.domain = domain.to_string();
        self.raw = format!("{}@{}", self.local, self.domain);
    }
}

impl std::fmt::Display for OmlEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}
