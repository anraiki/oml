fn main() {
    let src = r#"
[server]
url     = "https://api.example.com:9000/v2?debug=1"
email   = "ops@example.com"
ip      = "192.168.1.10"

[database]
primary_url = "postgres://user:secret@db.internal:5432/prod"
backup_url  = "not configured yet"   # heuristic match, invalid — silently stays a string

[paths]
config_file = "/etc/myapp/config.toml"
log_dir     = "/var/log/myapp"

[aliases]
home_link   = "https://example.com"
"#;

    let doc = oml::parse(src).expect("parse failed");

    // --- URL ---
    let server = doc.get("server").unwrap();
    let url = server.get("url").unwrap();
    println!("=== URL ===");
    println!("raw      : {url}");
    println!("scheme   : {}", url.get("scheme").unwrap());
    println!("host     : {}", url.get("host").unwrap());
    println!("port     : {}", url.get("port").unwrap());
    println!("path     : {}", url.get("path").unwrap());
    println!("query    : {}", url.get("query").unwrap());

    // --- Email ---
    let email = server.get("email").unwrap();
    println!("\n=== Email ===");
    println!("raw    : {email}");
    println!("local  : {}", email.get("local").unwrap());
    println!("domain : {}", email.get("domain").unwrap());

    // --- IP ---
    let ip = server.get("ip").unwrap();
    println!("\n=== IP ===");
    println!("raw     : {ip}");
    println!("version : {}", ip.get("version").unwrap());
    println!("private : {}", ip.get("private").unwrap());
    println!("octets  : {}", ip.get("octets").unwrap());

    // --- Heuristic fallback ---
    let db = doc.get("database").unwrap();
    let primary = db.get("primary_url").unwrap();
    let backup = db.get("backup_url").unwrap();
    println!("\n=== Heuristic URLs ===");
    println!("primary_url is URL    : {}", primary.is_url());
    println!("primary_url host      : {}", primary.get("host").unwrap());
    println!("backup_url is string  : {}", backup.is_string());
    println!("backup_url value      : {backup}");

    // --- Path ---
    let paths = doc.get("paths").unwrap();
    let cfg = paths.get("config_file").unwrap();
    println!("\n=== Path ===");
    println!("raw       : {cfg}");
    println!("name      : {}", cfg.get("name").unwrap());
    println!("stem      : {}", cfg.get("stem").unwrap());
    println!("extension : {}", cfg.get("extension").unwrap());
    println!("parent    : {}", cfg.get("parent").unwrap());
    println!("absolute  : {}", cfg.get("absolute").unwrap());

    // --- Mutation example ---
    println!("\n=== URL Mutation ===");
    let mut url_val = server.get("url").unwrap().as_url().unwrap().clone();
    url_val.set_host("new.example.com").unwrap();
    println!("after set_host: {url_val}");
}
