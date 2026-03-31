use oml::parse;

// ---------------------------------------------------------------------------
// URL
// ---------------------------------------------------------------------------

#[test]
fn url_explicit_sub_properties() {
    let doc = parse(r#"
[server]
url = "https://api.example.com:9000/v2?pretty=1#top"
"#)
    .unwrap();

    let url = doc.get("server").unwrap().get("url").unwrap();
    assert_eq!(url.get("scheme").unwrap().as_str(), Some("https"));
    assert_eq!(url.get("host").unwrap().as_str(), Some("api.example.com"));
    assert_eq!(url.get("port").unwrap().as_int(), Some(9000));
    assert_eq!(url.get("path").unwrap().as_str(), Some("/v2"));
    assert_eq!(url.get("query").unwrap().as_str(), Some("pretty=1"));
    assert_eq!(url.get("fragment").unwrap().as_str(), Some("top"));
}

#[test]
fn url_absent_optional_sub_properties() {
    let doc = parse(r#"url = "https://example.com/""#).unwrap();
    let url = doc.get("url").unwrap();
    assert!(url.get("port").is_none());
    assert!(url.get("query").is_none());
    assert!(url.get("fragment").is_none());
}

#[test]
fn url_explicit_invalid_is_parse_error() {
    let result = parse(r#"url = "not a url""#);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("URL"), "error should mention type: {msg}");
}

#[test]
fn url_heuristic_valid() {
    let doc = parse(r#"primary_url = "https://db.internal/mydb""#).unwrap();
    let url = doc.get("primary_url").unwrap();
    assert!(url.is_url());
    assert_eq!(url.get("host").unwrap().as_str(), Some("db.internal"));
}

#[test]
fn url_heuristic_invalid_falls_back_to_string() {
    let doc = parse(r#"backup_url = "not a url at all""#).unwrap();
    let val = doc.get("backup_url").unwrap();
    assert!(val.is_string());
    assert_eq!(val.as_str(), Some("not a url at all"));
}

#[test]
fn url_as_str_returns_raw() {
    let doc = parse(r#"url = "https://example.com/path""#).unwrap();
    assert_eq!(
        doc.get("url").unwrap().as_str(),
        Some("https://example.com/path")
    );
}

#[test]
fn url_array_plural_key() {
    let doc = parse(r#"
[server]
urls = ["https://primary.example.com", "https://fallback.example.com"]
"#)
    .unwrap();

    let arr = doc
        .get("server")
        .unwrap()
        .get("urls")
        .unwrap()
        .as_array()
        .unwrap()
        .clone();

    assert_eq!(arr[0].get("host").unwrap().as_str(), Some("primary.example.com"));
    assert_eq!(arr[1].get("host").unwrap().as_str(), Some("fallback.example.com"));
}

#[test]
fn url_userinfo() {
    let doc = parse(r#"url = "ftp://user:pass@ftp.example.com/pub""#).unwrap();
    let url = doc.get("url").unwrap();
    assert_eq!(url.get("userinfo").unwrap().as_str(), Some("user:pass"));
}

// ---------------------------------------------------------------------------
// Email
// ---------------------------------------------------------------------------

#[test]
fn email_explicit_sub_properties() {
    let doc = parse(r#"email = "alice@example.com""#).unwrap();
    let e = doc.get("email").unwrap();
    assert!(e.is_email());
    assert_eq!(e.get("local").unwrap().as_str(), Some("alice"));
    assert_eq!(e.get("domain").unwrap().as_str(), Some("example.com"));
}

#[test]
fn email_explicit_invalid_is_parse_error() {
    assert!(parse(r#"email = "not-an-email""#).is_err());
}

#[test]
fn email_heuristic_fallback() {
    let doc = parse(r#"contact_email = "invalid@@bad""#).unwrap();
    assert!(doc.get("contact_email").unwrap().is_string());
}

#[test]
fn email_as_str_returns_raw() {
    let doc = parse(r#"email = "bob@example.org""#).unwrap();
    assert_eq!(doc.get("email").unwrap().as_str(), Some("bob@example.org"));
}

// ---------------------------------------------------------------------------
// IP Address
// ---------------------------------------------------------------------------

#[test]
fn ip_v4_sub_properties() {
    let doc = parse(r#"ip = "192.168.1.1""#).unwrap();
    let ip = doc.get("ip").unwrap();
    assert!(ip.is_ip());
    assert_eq!(ip.get("version").unwrap().as_int(), Some(4));
    assert_eq!(ip.get("private").unwrap().as_bool(), Some(true));
    assert_eq!(ip.get("loopback").unwrap().as_bool(), Some(false));

    let octets = ip.get("octets").unwrap().as_array().unwrap().clone();
    assert_eq!(octets[0].as_int(), Some(192));
    assert_eq!(octets[1].as_int(), Some(168));
    assert_eq!(octets[2].as_int(), Some(1));
    assert_eq!(octets[3].as_int(), Some(1));
}

#[test]
fn ip_v4_loopback() {
    let doc = parse(r#"ip = "127.0.0.1""#).unwrap();
    let ip = doc.get("ip").unwrap();
    assert_eq!(ip.get("loopback").unwrap().as_bool(), Some(true));
}

#[test]
fn ip_v4_public() {
    let doc = parse(r#"ip = "8.8.8.8""#).unwrap();
    let ip = doc.get("ip").unwrap();
    assert_eq!(ip.get("private").unwrap().as_bool(), Some(false));
    assert_eq!(ip.get("version").unwrap().as_int(), Some(4));
}

#[test]
fn ip_v6_loopback() {
    let doc = parse(r#"ip = "::1""#).unwrap();
    let ip = doc.get("ip").unwrap();
    assert_eq!(ip.get("version").unwrap().as_int(), Some(6));
    assert_eq!(ip.get("loopback").unwrap().as_bool(), Some(true));
    assert!(ip.get("groups").is_some());
    assert!(ip.get("octets").is_none()); // IPv6 has no octets
}

#[test]
fn ip_explicit_invalid_is_parse_error() {
    assert!(parse(r#"ip = "not.an.ip""#).is_err());
}

#[test]
fn ip_heuristic_hostname_falls_back_to_string() {
    // _host suffix with a hostname (not an IP) silently becomes a plain string.
    let doc = parse(r#"db_host = "my-database.internal""#).unwrap();
    assert!(doc.get("db_host").unwrap().is_string());
}

#[test]
fn ip_heuristic_valid() {
    let doc = parse(r#"db_host = "10.0.0.5""#).unwrap();
    assert!(doc.get("db_host").unwrap().is_ip());
}

// ---------------------------------------------------------------------------
// Filesystem Path
// ---------------------------------------------------------------------------

#[test]
fn path_explicit_sub_properties() {
    let doc = parse(r#"file = "/etc/myapp/config.toml""#).unwrap();
    let p = doc.get("file").unwrap();
    assert!(p.is_path());
    assert_eq!(p.get("name").unwrap().as_str(), Some("config.toml"));
    assert_eq!(p.get("stem").unwrap().as_str(), Some("config"));
    assert_eq!(p.get("extension").unwrap().as_str(), Some("toml"));
    assert_eq!(p.get("parent").unwrap().as_str(), Some("/etc/myapp"));
    assert_eq!(p.get("absolute").unwrap().as_bool(), Some(true));
}

#[test]
fn path_relative() {
    let doc = parse(r#"path = "relative/dir/file.txt""#).unwrap();
    let p = doc.get("path").unwrap();
    assert_eq!(p.get("absolute").unwrap().as_bool(), Some(false));
    assert_eq!(p.get("stem").unwrap().as_str(), Some("file"));
}

#[test]
fn path_parts() {
    let doc = parse(r#"path = "/usr/local/bin""#).unwrap();
    let parts = doc.get("path").unwrap().get("parts").unwrap();
    let arr = parts.as_array().unwrap();
    assert!(arr.iter().any(|v| v.as_str() == Some("usr")));
}

#[test]
fn path_heuristic_suffix() {
    let doc = parse(r#"config_path = "/var/log/app.log""#).unwrap();
    assert!(doc.get("config_path").unwrap().is_path());
}

// ---------------------------------------------------------------------------
// Plain TOML passthrough
// ---------------------------------------------------------------------------

#[test]
fn plain_toml_primitives() {
    let doc = parse(r#"
name    = "my-app"
count   = 42
ratio   = 3.14
enabled = true
"#)
    .unwrap();

    assert_eq!(doc.get("name").unwrap().as_str(), Some("my-app"));
    assert_eq!(doc.get("count").unwrap().as_int(), Some(42));
    assert_eq!(doc.get("ratio").unwrap().as_float(), Some(3.14));
    assert_eq!(doc.get("enabled").unwrap().as_bool(), Some(true));
}

#[test]
fn plain_array_of_strings() {
    let doc = parse(r#"tags = ["alpha", "beta", "gamma"]"#).unwrap();
    let arr = doc.get("tags").unwrap().as_array().unwrap().clone();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0].as_str(), Some("alpha"));
}

#[test]
fn nested_tables() {
    let doc = parse(r#"
[a.b.c]
url = "https://deep.example.com/"
"#)
    .unwrap();

    let url = doc
        .get("a")
        .unwrap()
        .get("b")
        .unwrap()
        .get("c")
        .unwrap()
        .get("url")
        .unwrap();
    assert_eq!(url.get("host").unwrap().as_str(), Some("deep.example.com"));
}
