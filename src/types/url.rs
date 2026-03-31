use crate::error::OmlError;

/// A resolved URL value with addressable sub-properties.
///
/// Backed by RFC 3986. Parsing uses the `url` crate (WHATWG URL Standard,
/// a widely adopted superset of RFC 3986).
#[derive(Debug, Clone)]
pub struct OmlUrl {
    parsed: url::Url,
}

impl OmlUrl {
    /// Parse for an **explicit** canonical key. Returns an error on invalid input.
    pub fn parse(input: &str, key: &str) -> Result<Self, OmlError> {
        url::Url::parse(input)
            .map(|parsed| OmlUrl { parsed })
            .map_err(|e| OmlError::ExplicitTypeMismatch {
                key: key.to_string(),
                type_name: "URL",
                detail: e.to_string(),
            })
    }

    /// Parse for a **heuristic** key. Returns `None` on invalid input (silent fallback).
    pub fn try_parse(input: &str) -> Option<Self> {
        url::Url::parse(input).ok().map(|parsed| OmlUrl { parsed })
    }

    pub fn as_str(&self) -> &str {
        self.parsed.as_str()
    }

    pub fn scheme(&self) -> &str {
        self.parsed.scheme()
    }

    pub fn host(&self) -> Option<&str> {
        self.parsed.host_str()
    }

    pub fn port(&self) -> Option<u16> {
        self.parsed.port()
    }

    pub fn path(&self) -> &str {
        self.parsed.path()
    }

    pub fn query(&self) -> Option<&str> {
        self.parsed.query()
    }

    pub fn fragment(&self) -> Option<&str> {
        self.parsed.fragment()
    }

    pub fn userinfo(&self) -> Option<String> {
        let username = self.parsed.username();
        let password = self.parsed.password();
        if username.is_empty() && password.is_none() {
            return None;
        }
        let mut info = username.to_string();
        if let Some(pass) = password {
            info.push(':');
            info.push_str(pass);
        }
        Some(info)
    }

    // --- Mutation ---

    pub fn set_host(&mut self, host: &str) -> Result<(), OmlError> {
        self.parsed
            .set_host(Some(host))
            .map_err(|e| OmlError::ExplicitTypeMismatch {
                key: "host".into(),
                type_name: "URL host",
                detail: e.to_string(),
            })
    }

    pub fn set_scheme(&mut self, scheme: &str) -> Result<(), OmlError> {
        self.parsed
            .set_scheme(scheme)
            .map_err(|_| OmlError::ExplicitTypeMismatch {
                key: "scheme".into(),
                type_name: "URL scheme",
                detail: format!("invalid scheme: {scheme}"),
            })
    }

    pub fn set_port(&mut self, port: Option<u16>) -> Result<(), OmlError> {
        self.parsed
            .set_port(port)
            .map_err(|_| OmlError::ExplicitTypeMismatch {
                key: "port".into(),
                type_name: "URL port",
                detail: "cannot set port on this URL".into(),
            })
    }

    pub fn set_path(&mut self, path: &str) {
        self.parsed.set_path(path);
    }

    pub fn set_query(&mut self, query: Option<&str>) {
        self.parsed.set_query(query);
    }

    pub fn set_fragment(&mut self, fragment: Option<&str>) {
        self.parsed.set_fragment(fragment);
    }

    pub fn inner(&self) -> &url::Url {
        &self.parsed
    }
}

impl PartialEq for OmlUrl {
    fn eq(&self, other: &Self) -> bool {
        self.parsed == other.parsed
    }
}

impl std::fmt::Display for OmlUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parsed)
    }
}
