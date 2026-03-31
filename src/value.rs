use indexmap::IndexMap;

use crate::types::{OmlEmail, OmlIp, OmlPath, OmlUrl};

/// An ordered map of string keys to [`OmlValue`]s. Preserves insertion order.
pub type OmlTable = IndexMap<String, OmlValue>;

/// The core OML value type. Encompasses all TOML primitives plus semantic types.
#[derive(Debug, Clone, PartialEq)]
pub enum OmlValue {
    // TOML primitives
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    /// Datetime stored as its canonical string representation.
    Datetime(String),
    Array(Vec<OmlValue>),
    Table(OmlTable),
    // Semantic types (resolved from String)
    Url(OmlUrl),
    Path(OmlPath),
    Email(OmlEmail),
    Ip(OmlIp),
}

impl OmlValue {
    /// Look up a table key **or** a semantic sub-property by name.
    ///
    /// For tables, this is a standard key lookup.
    /// For semantic types, this materializes a sub-property:
    ///
    /// ```rust
    /// # let doc = oml::parse(r#"url = "https://example.com:8080/api""#).unwrap();
    /// assert_eq!(doc.get("url").unwrap().get("host").unwrap().as_str(), Some("example.com"));
    /// assert_eq!(doc.get("url").unwrap().get("port").unwrap().as_int(), Some(8080));
    /// ```
    pub fn get(&self, key: &str) -> Option<OmlValue> {
        match self {
            OmlValue::Table(map) => map.get(key).cloned(),
            OmlValue::Url(u) => url_subprop(u, key),
            OmlValue::Path(p) => path_subprop(p, key),
            OmlValue::Email(e) => email_subprop(e, key),
            OmlValue::Ip(ip) => ip_subprop(ip, key),
            _ => None,
        }
    }

    /// Access an array element by index.
    pub fn index(&self, i: usize) -> Option<&OmlValue> {
        match self {
            OmlValue::Array(arr) => arr.get(i),
            _ => None,
        }
    }

    // --- Type accessors ---

    pub fn as_str(&self) -> Option<&str> {
        match self {
            OmlValue::String(s) => Some(s),
            OmlValue::Url(u) => Some(u.as_str()),
            OmlValue::Email(e) => Some(e.as_str()),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            OmlValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            OmlValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            OmlValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<OmlValue>> {
        match self {
            OmlValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&OmlTable> {
        match self {
            OmlValue::Table(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_url(&self) -> Option<&OmlUrl> {
        match self {
            OmlValue::Url(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_url_mut(&mut self) -> Option<&mut OmlUrl> {
        match self {
            OmlValue::Url(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_path(&self) -> Option<&OmlPath> {
        match self {
            OmlValue::Path(p) => Some(p),
            _ => None,
        }
    }

    pub fn as_path_mut(&mut self) -> Option<&mut OmlPath> {
        match self {
            OmlValue::Path(p) => Some(p),
            _ => None,
        }
    }

    pub fn as_email(&self) -> Option<&OmlEmail> {
        match self {
            OmlValue::Email(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_email_mut(&mut self) -> Option<&mut OmlEmail> {
        match self {
            OmlValue::Email(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_ip(&self) -> Option<&OmlIp> {
        match self {
            OmlValue::Ip(ip) => Some(ip),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, OmlValue::String(_))
    }
    pub fn is_table(&self) -> bool {
        matches!(self, OmlValue::Table(_))
    }
    pub fn is_array(&self) -> bool {
        matches!(self, OmlValue::Array(_))
    }
    pub fn is_url(&self) -> bool {
        matches!(self, OmlValue::Url(_))
    }
    pub fn is_path(&self) -> bool {
        matches!(self, OmlValue::Path(_))
    }
    pub fn is_email(&self) -> bool {
        matches!(self, OmlValue::Email(_))
    }
    pub fn is_ip(&self) -> bool {
        matches!(self, OmlValue::Ip(_))
    }
}

impl std::fmt::Display for OmlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OmlValue::String(s) => write!(f, "{s}"),
            OmlValue::Integer(i) => write!(f, "{i}"),
            OmlValue::Float(fl) => write!(f, "{fl}"),
            OmlValue::Boolean(b) => write!(f, "{b}"),
            OmlValue::Datetime(d) => write!(f, "{d}"),
            OmlValue::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            OmlValue::Table(_) => write!(f, "{{...}}"),
            OmlValue::Url(u) => write!(f, "{u}"),
            OmlValue::Path(p) => write!(f, "{p}"),
            OmlValue::Email(e) => write!(f, "{e}"),
            OmlValue::Ip(ip) => write!(f, "{ip}"),
        }
    }
}

// --- Sub-property materializers ---

fn url_subprop(url: &OmlUrl, key: &str) -> Option<OmlValue> {
    match key {
        "scheme" => Some(OmlValue::String(url.scheme().to_string())),
        "host" => url.host().map(|h| OmlValue::String(h.to_string())),
        "port" => url.port().map(|p| OmlValue::Integer(p as i64)),
        "path" => Some(OmlValue::String(url.path().to_string())),
        "query" => url.query().map(|q| OmlValue::String(q.to_string())),
        "fragment" => url.fragment().map(|frag| OmlValue::String(frag.to_string())),
        "userinfo" => url.userinfo().map(OmlValue::String),
        _ => None,
    }
}

fn path_subprop(path: &OmlPath, key: &str) -> Option<OmlValue> {
    match key {
        "parent" => path.parent().map(OmlValue::String),
        "stem" => path.stem().map(OmlValue::String),
        "name" => path.name().map(OmlValue::String),
        "extension" => path.extension().map(OmlValue::String),
        "absolute" => Some(OmlValue::Boolean(path.is_absolute())),
        "raw" => Some(OmlValue::String(path.raw().to_string())),
        "parts" => Some(OmlValue::Array(
            path.parts().into_iter().map(OmlValue::String).collect(),
        )),
        _ => None,
    }
}

fn email_subprop(email: &OmlEmail, key: &str) -> Option<OmlValue> {
    match key {
        "local" => Some(OmlValue::String(email.local().to_string())),
        "domain" => Some(OmlValue::String(email.domain().to_string())),
        _ => None,
    }
}

fn ip_subprop(ip: &OmlIp, key: &str) -> Option<OmlValue> {
    match key {
        "version" => Some(OmlValue::Integer(ip.version() as i64)),
        "octets" => ip.octets().map(|octs| {
            OmlValue::Array(octs.iter().map(|&o| OmlValue::Integer(o as i64)).collect())
        }),
        "groups" => ip.groups().map(|groups| {
            OmlValue::Array(
                groups
                    .iter()
                    .map(|&g| OmlValue::String(format!("{g:x}")))
                    .collect(),
            )
        }),
        "loopback" => Some(OmlValue::Boolean(ip.is_loopback())),
        "private" => Some(OmlValue::Boolean(ip.is_private())),
        _ => None,
    }
}
