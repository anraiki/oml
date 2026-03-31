//! # OML — Obvious Minimal Language
//!
//! A configuration file format that extends TOML with **semantic field awareness**.
//! Where TOML treats all string values as opaque, OML resolves recognized field
//! names into structured values with addressable sub-properties.
//!
//! OML is a superset of TOML: any valid TOML document is a valid OML document.
//!
//! ## Quick Start
//!
//! ```rust
//! let src = r#"
//! [server]
//! url     = "https://api.example.com:9000/v2"
//! email   = "ops@example.com"
//! ip      = "192.168.1.10"
//!
//! [paths]
//! config_file = "/etc/myapp/config.toml"
//! "#;
//!
//! let doc = oml::parse(src).unwrap();
//!
//! // URL sub-properties
//! let server = doc.get("server").unwrap();
//! assert_eq!(server.get("url").unwrap().get("host").unwrap().as_str(), Some("api.example.com"));
//! assert_eq!(server.get("url").unwrap().get("port").unwrap().as_int(), Some(9000));
//!
//! // Email sub-properties
//! assert_eq!(server.get("email").unwrap().get("domain").unwrap().as_str(), Some("example.com"));
//!
//! // IP sub-properties
//! assert_eq!(server.get("ip").unwrap().get("version").unwrap().as_int(), Some(4));
//! assert_eq!(server.get("ip").unwrap().get("private").unwrap().as_bool(), Some(true));
//!
//! // Path sub-properties (heuristic: _file suffix)
//! let paths = doc.get("paths").unwrap();
//! assert_eq!(paths.get("config_file").unwrap().get("extension").unwrap().as_str(), Some("toml"));
//! ```
//!
//! ## Semantic Resolution
//!
//! | Key form        | Example          | Contract                                      |
//! |-----------------|------------------|-----------------------------------------------|
//! | Canonical name  | `url = "..."`    | **Explicit** — invalid value is a parse error |
//! | Suffix match    | `primary_url = "..."` | **Heuristic** — invalid value falls back to plain string |
//!
//! See the [OML specification](https://github.com/anraiki/oml/blob/main/SPEC.md) for the
//! full type catalog, sub-property reference, and conformance rules.

mod classifier;
mod error;
mod types;
mod value;

pub use error::OmlError;
pub use types::{OmlEmail, OmlIp, OmlPath, OmlUrl};
pub use value::{OmlTable, OmlValue};

use classifier::{classify, classify_for_array, ClassificationKind, SemanticType};

/// Parse an OML document string into an [`OmlValue::Table`].
///
/// Returns an error if the input is not valid TOML, or if any explicit
/// semantic key contains an invalid value (§3.1 of the spec).
pub fn parse(input: &str) -> Result<OmlValue, OmlError> {
    let toml_value: toml::Value = toml::from_str(input)?;
    convert(toml_value, None)
}

// --- Internal conversion ---

fn convert(value: toml::Value, key: Option<&str>) -> Result<OmlValue, OmlError> {
    match value {
        toml::Value::String(s) => {
            if let Some(k) = key {
                if let Some((sem_type, kind)) = classify(k) {
                    return resolve_string(s, sem_type, kind, k);
                }
            }
            Ok(OmlValue::String(s))
        }

        toml::Value::Integer(i) => Ok(OmlValue::Integer(i)),
        toml::Value::Float(f) => Ok(OmlValue::Float(f)),
        toml::Value::Boolean(b) => Ok(OmlValue::Boolean(b)),
        toml::Value::Datetime(dt) => Ok(OmlValue::Datetime(dt.to_string())),

        toml::Value::Array(arr) => {
            // Check if the key is semantically classified (including plural forms).
            let sem_class = key.and_then(classify_for_array);
            if let Some((sem_type, kind)) = sem_class {
                let resolved = arr
                    .into_iter()
                    .map(|elem| match elem {
                        toml::Value::String(s) => resolve_array_elem(s, sem_type, kind, key.unwrap()),
                        other => convert(other, None),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(OmlValue::Array(resolved));
            }
            let items = arr
                .into_iter()
                .map(|v| convert(v, None))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(OmlValue::Array(items))
        }

        toml::Value::Table(table) => {
            let mut map = OmlTable::new();
            for (k, v) in table {
                let val = convert(v, Some(&k))?;
                map.insert(k, val);
            }
            Ok(OmlValue::Table(map))
        }
    }
}

fn resolve_string(
    s: String,
    sem_type: SemanticType,
    kind: ClassificationKind,
    key: &str,
) -> Result<OmlValue, OmlError> {
    match (sem_type, kind) {
        (SemanticType::Url, ClassificationKind::Explicit) => {
            OmlUrl::parse(&s, key).map(OmlValue::Url)
        }
        (SemanticType::Url, ClassificationKind::Heuristic) => Ok(OmlUrl::try_parse(&s)
            .map(OmlValue::Url)
            .unwrap_or(OmlValue::String(s))),

        // Path never fails — any string is a valid path.
        (SemanticType::Path, _) => Ok(OmlValue::Path(OmlPath::new(&s))),

        (SemanticType::Email, ClassificationKind::Explicit) => {
            OmlEmail::parse(&s, key).map(OmlValue::Email)
        }
        (SemanticType::Email, ClassificationKind::Heuristic) => Ok(OmlEmail::try_parse(&s)
            .map(OmlValue::Email)
            .unwrap_or(OmlValue::String(s))),

        (SemanticType::Ip, ClassificationKind::Explicit) => {
            OmlIp::parse(&s, key).map(OmlValue::Ip)
        }
        (SemanticType::Ip, ClassificationKind::Heuristic) => Ok(OmlIp::try_parse(&s)
            .map(OmlValue::Ip)
            .unwrap_or(OmlValue::String(s))),
    }
}

/// For arrays, heuristic invalid elements fall back to string (consistent with §3.2).
/// Explicit invalid elements are still parse errors (§3.1).
fn resolve_array_elem(
    s: String,
    sem_type: SemanticType,
    kind: ClassificationKind,
    key: &str,
) -> Result<OmlValue, OmlError> {
    match kind {
        ClassificationKind::Explicit => resolve_string(s, sem_type, kind, key),
        ClassificationKind::Heuristic => {
            Ok(match sem_type {
                SemanticType::Url => OmlUrl::try_parse(&s)
                    .map(OmlValue::Url)
                    .unwrap_or(OmlValue::String(s)),
                SemanticType::Path => OmlValue::Path(OmlPath::new(&s)),
                SemanticType::Email => OmlEmail::try_parse(&s)
                    .map(OmlValue::Email)
                    .unwrap_or(OmlValue::String(s)),
                SemanticType::Ip => OmlIp::try_parse(&s)
                    .map(OmlValue::Ip)
                    .unwrap_or(OmlValue::String(s)),
            })
        }
    }
}
