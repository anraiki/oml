# OML Specification

**Version:** 0.1.0-draft  
**Status:** Draft  
**Format Name:** OML (Obvious Minimal Language)

---

## 1. Overview

OML is a configuration file format based on TOML's structural model, extended with **semantic field awareness**. Where TOML treats all string values as opaque, OML applies heuristic or explicit type resolution to recognized field names, materializing structured sub-properties from scalar values.

OML files use the `.oml` extension.

OML is a superset of TOML's value model. Any valid TOML document is a valid OML document. OML adds no new syntax — it adds meaning to existing syntax through field name conventions and explicit type declarations.

---

## 2. Foundational Types

OML inherits all TOML primitive types:

- `String`
- `Integer`
- `Float`
- `Boolean`
- `Datetime` / `Date` / `Time`
- `Array`
- `Table`

OML adds a layer of **Semantic Types** on top of `String`. A semantic type is a string with a known structure that the parser resolves into a value with addressable sub-properties.

---

## 3. Semantic Type Resolution

OML resolves semantic types through two mechanisms: **explicit declaration** and **heuristic inference**. These have different contracts.

### 3.1 Explicit Declaration

A field is explicitly typed when its key matches a recognized **canonical name** exactly (see §4). Explicit declarations MUST be valid for their type. A value that fails validation against an explicitly declared type is a **parse error** and MUST halt parsing.

```oml
[server]
url = "https://example.com:8080/api"   # explicit — key is exactly "url"
uri = "urn:isbn:0451450523"            # explicit — key is exactly "uri"
```

### 3.2 Heuristic Inference

A field is heuristically typed when its key matches a recognized **suffix pattern** (see §4). Heuristic inference is best-effort. A value that fails validation against an inferred type MUST silently fall back to a plain `String`. No warning, no error. Sub-properties will not be materialized.

```oml
[database]
primary_url = "https://db.internal"   # heuristic — suffix "_url"
backup_url  = "not a url at all"      # heuristic match, invalid value — silent string fallback
```

### 3.3 Resolution Priority

1. Explicit declaration (exact canonical key match)
1. Heuristic suffix match
1. Plain string

---

## 4. Semantic Type Catalog

The following table defines all v1 semantic types. Each entry specifies canonical key names (explicit), recognized suffix patterns (heuristic), sub-properties, and the standard or pattern governing validation.

Inclusion criterion: a semantic type MUST be backed by an IETF RFC, POSIX standard, W3C spec, or an unambiguous, widely implemented pattern. Types without a governing standard are not eligible for inclusion.

---

### 4.1 URL

**Standard:** RFC 3986  
**Canonical keys:** `url`, `uri`, `href`, `endpoint`  
**Suffix patterns:** `_url`, `_uri`, `_href`, `_endpoint`, `_link`

**Sub-properties (read):**

| Property    | Type     | Description                        |
|-------------|----------|------------------------------------|
| `.scheme`   | String   | e.g. `"https"`                     |
| `.host`     | String   | e.g. `"example.com"`               |
| `.port`     | Integer? | e.g. `8080`, absent if not present |
| `.path`     | String   | e.g. `"/api/v1"`                   |
| `.query`    | String?  | Raw query string, e.g. `"a=1&b=2"` |
| `.fragment` | String?  | e.g. `"section-2"`                 |
| `.userinfo` | String?  | e.g. `"user:pass"`                 |

**Write reconstruction:** Modifying any sub-property reconstructs the full URL string as the canonical value. Port is omitted from reconstruction if it matches the scheme default (80 for http, 443 for https).

---

### 4.2 Filesystem Path

**Standard:** POSIX.1-2017 (IEEE Std 1003.1), Win32 UNC paths  
**Canonical keys:** `path`, `dir`, `directory`, `file`, `filepath`  
**Suffix patterns:** `_path`, `_dir`, `_file`, `_filepath`, `_folder`

**Sub-properties (read):**

| Property     | Type    | Description                       |
|--------------|---------|-----------------------------------|
| `.parent`    | String  | Parent directory path             |
| `.stem`      | String  | Filename without extension        |
| `.name`      | String  | Filename with extension           |
| `.extension` | String? | Extension without leading dot     |
| `.absolute`  | Boolean | Whether the path is absolute      |
| `.parts`     | Array   | Path segments as array of strings |

**Write reconstruction:** Modifying `.parent`, `.stem`, or `.extension` reconstructs the full path. Separator is normalized to the platform convention at parse time; raw value is preserved as `.raw`.

---

### 4.3 Email Address

**Standard:** RFC 5321, RFC 5322  
**Canonical keys:** `email`, `mail`  
**Suffix patterns:** `_email`, `_mail`

**Sub-properties (read):**

| Property  | Type   | Description     |
|-----------|--------|-----------------|
| `.local`  | String | Part before `@` |
| `.domain` | String | Part after `@`  |

**Write reconstruction:** `local + "@" + domain`.

---

### 4.4 IP Address

**Standard:** RFC 791 (IPv4), RFC 4291 (IPv6)  
**Canonical keys:** `ip`, `host`, `address`, `addr`  
**Suffix patterns:** `_ip`, `_host`, `_addr`, `_address`

**Sub-properties (read):**

| Property    | Type    | Description                              |
|-------------|---------|------------------------------------------|
| `.version`  | Integer | `4` or `6`                               |
| `.octets`   | Array?  | IPv4 only — array of 4 integers          |
| `.groups`   | Array?  | IPv6 only — array of 8 hex group strings |
| `.loopback` | Boolean | Whether address is a loopback address    |
| `.private`  | Boolean | Whether address is in a private range    |

**Write reconstruction:** Reconstruct canonical string form from octets/groups. IPv6 uses compressed form per RFC 5952.

**Note:** Hostnames (non-IP strings) matching `_host` suffix are treated as plain strings. Sub-properties are only materialized for valid IP address values.

---

### 4.5 Future Type Eligibility

Types under consideration for post-v1, contingent on stable standard backing:

- **UUID** — RFC 4122. Canonical: `id`, `uuid`. Sub: `.version`, `.variant`
- **MIME Type** — RFC 2045. Suffix: `_type`, `_mime`, `_content_type`. Sub: `.type`, `.subtype`, `.parameters`
- **Color** — CSS Color Level 4 / sRGB. Suffix: `_color`, `_colour`. Sub: `.r`, `.g`, `.b`, `.hex`, `.hsl`
- **Phone** — E.164 (ITU-T). Suffix: `_phone`, `_tel`. Sub: `.country_code`, `.national`
- **Duration** — ISO 8601 duration. Suffix: `_duration`, `_ttl`. Sub: `.seconds`, `.minutes`, `.hours`
- **Semver** — semver.org 2.0.0. Suffix: `_version`, `_ver`. Sub: `.major`, `.minor`, `.patch`, `.pre`, `.build`

---

## 5. Sub-property Access

Sub-properties are accessed using dot notation on the field key.

```oml
[server]
url = "https://api.example.com:9000/v2"
```

Resolved access:

```
server.url          → "https://api.example.com:9000/v2"
server.url.scheme   → "https"
server.url.host     → "api.example.com"
server.url.port     → 9000
server.url.path     → "/v2"
server.url.query    → nil
server.url.fragment → nil
```

Sub-properties are **read-write**. Writing to a sub-property mutates the reconstructed canonical value:

```
server.url.host = "new.example.com"
server.url → "https://new.example.com:9000/v2"
```

Sub-properties MUST NOT be set as top-level OML keys. The following is invalid:

```oml
# INVALID — sub-properties are derived, not declared
[server]
url.host = "example.com"
```

### 5.1 Arrays of Semantic Values

Arrays of strings on a semantically classified key are treated as arrays of that semantic type. Each element is resolved individually and sub-properties are accessed by index.

```oml
[server]
urls = ["https://primary.example.com", "https://fallback.example.com"]
```

Resolved access:

```
server.urls[0]          → "https://primary.example.com"
server.urls[0].host     → "primary.example.com"
server.urls[0].scheme   → "https"
server.urls[1].host     → "fallback.example.com"
```

**Type is determined by the key, not per-element.** The entire array is either semantic or it isn't. Mixed-type arrays are not supported.

**Validation follows the same explicit/heuristic contract as scalars.** An invalid element in a heuristic-matched array is a type error at that index. An invalid element in an explicit-matched array is a parse error that MUST halt parsing. This is consistent with how statically typed languages handle assignment of a mismatched value — the type is the type.

---

## 6. Nil and Absent Sub-properties

Optional sub-properties (marked `?` in §4) return `nil` when absent in the source value. Nil sub-properties are excluded from write reconstruction.

---

## 7. Conflict Resolution

If a table contains both a semantic field and a key that would shadow a sub-property, it is a parse error.

```oml
[server]
url      = "https://example.com"
url.host = "other.com"           # PARSE ERROR — conflicts with derived sub-property
```

---

## 8. Serialization

OML serializes to `.oml`. When serializing:

- Semantic values are written as their canonical string form
- Sub-properties are not written as separate keys
- Reconstructed values replace the original if sub-properties were mutated

OML MAY also serialize to `.toml` with semantic information stripped (plain strings only), for interop with TOML tooling.

---

## 9. Implementation Notes

### 9.1 Parser Phases

1. **Lexing / structural parse** — identical to TOML
1. **Key classification** — for each string value, classify its key as explicit, heuristic, or plain
1. **Semantic resolution** — attempt type parsing for classified fields; apply fallback rules per §3
1. **Sub-property materialization** — populate derived properties on resolved values

### 9.2 Rust Crate Structure (reference)

```
oml/
├── src/
│   ├── lib.rs
│   ├── parser/         # structural parse (TOML-compatible)
│   ├── classifier/     # key name → semantic type mapping
│   ├── types/
│   │   ├── url.rs
│   │   ├── path.rs
│   │   ├── email.rs
│   │   └── ip.rs
│   └── value.rs        # OmlValue enum with sub-property access
├── tests/
└── SPEC.md
```

### 9.3 RFC 2119 Conformance

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in this document are to be interpreted as described in RFC 2119.

---

## 10. Open Questions

- [ ] Should suffix matching be case-insensitive? (`_URL` == `_url`?)
- [ ] Should OML expose a schema introspection API (list all semantic fields in a document)?
- [ ] Interop: should `.oml` files be parseable by TOML parsers with graceful degradation?
