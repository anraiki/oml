use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmlError {
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("invalid {type_name} for explicit key '{key}': {detail}")]
    ExplicitTypeMismatch {
        key: String,
        type_name: &'static str,
        detail: String,
    },
}
