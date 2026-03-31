#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticType {
    Url,
    Path,
    Email,
    Ip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassificationKind {
    Explicit,
    Heuristic,
}

/// Classify a key name into a semantic type and resolution kind.
/// Returns `None` for plain string keys.
pub fn classify(key: &str) -> Option<(SemanticType, ClassificationKind)> {
    // Exact canonical key matches → Explicit
    let explicit = match key {
        "url" | "uri" | "href" | "endpoint" => Some(SemanticType::Url),
        "path" | "dir" | "directory" | "file" | "filepath" => Some(SemanticType::Path),
        "email" | "mail" => Some(SemanticType::Email),
        "ip" | "host" | "address" | "addr" => Some(SemanticType::Ip),
        _ => None,
    };
    if let Some(t) = explicit {
        return Some((t, ClassificationKind::Explicit));
    }

    // Suffix patterns → Heuristic
    if ends_with_any(key, &["_url", "_uri", "_href", "_endpoint", "_link"]) {
        return Some((SemanticType::Url, ClassificationKind::Heuristic));
    }
    if ends_with_any(key, &["_path", "_dir", "_file", "_filepath", "_folder"]) {
        return Some((SemanticType::Path, ClassificationKind::Heuristic));
    }
    if ends_with_any(key, &["_email", "_mail"]) {
        return Some((SemanticType::Email, ClassificationKind::Heuristic));
    }
    if ends_with_any(key, &["_ip", "_host", "_addr", "_address"]) {
        return Some((SemanticType::Ip, ClassificationKind::Heuristic));
    }

    None
}

/// Like `classify`, but also strips a trailing `s` to handle plural array keys
/// (`urls`, `emails`, `paths`, etc.). Always resolves to Heuristic when plural.
pub fn classify_for_array(key: &str) -> Option<(SemanticType, ClassificationKind)> {
    if let Some(result) = classify(key) {
        return Some(result);
    }
    if key.ends_with('s') && key.len() > 1 {
        if let Some((sem_type, _)) = classify(&key[..key.len() - 1]) {
            return Some((sem_type, ClassificationKind::Heuristic));
        }
    }
    None
}

fn ends_with_any(key: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|s| key.ends_with(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_canonical() {
        assert_eq!(classify("url"), Some((SemanticType::Url, ClassificationKind::Explicit)));
        assert_eq!(classify("email"), Some((SemanticType::Email, ClassificationKind::Explicit)));
        assert_eq!(classify("host"), Some((SemanticType::Ip, ClassificationKind::Explicit)));
    }

    #[test]
    fn heuristic_suffix() {
        assert_eq!(classify("primary_url"), Some((SemanticType::Url, ClassificationKind::Heuristic)));
        assert_eq!(classify("db_host"), Some((SemanticType::Ip, ClassificationKind::Heuristic)));
        assert_eq!(classify("config_path"), Some((SemanticType::Path, ClassificationKind::Heuristic)));
    }

    #[test]
    fn plural_array() {
        assert_eq!(
            classify_for_array("urls"),
            Some((SemanticType::Url, ClassificationKind::Heuristic))
        );
    }

    #[test]
    fn plain_key() {
        assert!(classify("name").is_none());
        assert!(classify("version").is_none());
    }
}
