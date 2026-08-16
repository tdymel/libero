//! Fetches only the `.sublime-syntax` grammars enabled via `code-lang-*`
//! Cargo features (from the shared catalog in
//! `src/components/typography/code/language_catalog.rs`), verifies each
//! against its hardcoded SHA-256, and compiles them into a single trimmed
//! `SyntaxSet` dump embedded by `highlight.rs` via `include_bytes!`. This
//! keeps the binary limited to the languages actually opted into, without
//! vendoring grammar sources in this repo.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use syntect::parsing::{SyntaxDefinition, SyntaxSetBuilder};

include!("src/components/typography/code/language_catalog.rs");

fn feature_env_var(feature: &str) -> String {
    format!("CARGO_FEATURE_{}", feature.to_uppercase().replace('-', "_"))
}

/// Grammar paths can contain spaces and `#` (e.g. `"Batch File/Batch
/// File.sublime-syntax"`, `"C#/C#.sublime-syntax"`), which aren't valid raw
/// URI characters - percent-encode each path segment, preserving `/`.
fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            segment
                .bytes()
                .map(|byte| match byte {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                        (byte as char).to_string()
                    }
                    _ => format!("%{byte:02X}"),
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn fetch_verified(entry: &LanguageEntry) -> String {
    let url = format!(
        "https://raw.githubusercontent.com/sublimehq/Packages/{}/{}",
        entry.commit,
        encode_path(entry.upstream_path)
    );
    let text = ureq::get(&url)
        .call()
        .unwrap_or_else(|err| panic!("failed to fetch {url}: {err}"))
        .body_mut()
        .read_to_string()
        .unwrap_or_else(|err| panic!("failed to read response body from {url}: {err}"));

    let actual = hex_encode(&Sha256::digest(text.as_bytes()));
    assert!(
        actual == entry.sha256,
        "checksum mismatch for {url}: expected {}, got {actual} - upstream file may have \
         changed or the download was tampered with",
        entry.sha256,
    );

    text
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Outside `target/` on purpose - `OUT_DIR` gets wiped by `cargo clean` and
/// varies per profile/build (e.g. `dx serve` vs `dx serve --release`), so
/// caching only there would still refetch on every fresh dev-server start.
fn cache_dir() -> PathBuf {
    let base = env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(env::temp_dir);
    base.join("libero").join("syntect-syntaxes")
}

/// Changes whenever an entry's pin/hash changes or the enabled language set
/// changes, so a stale cache entry is simply never looked up again rather
/// than needing explicit invalidation.
fn cache_key(enabled: &[&LanguageEntry]) -> String {
    let mut hasher = Sha256::new();
    for entry in enabled {
        hasher.update(entry.feature.as_bytes());
        hasher.update(entry.commit.as_bytes());
        hasher.update(entry.sha256.as_bytes());
    }
    hex_encode(&hasher.finalize())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/components/typography/code/language_catalog.rs");

    let enabled: Vec<&LanguageEntry> = LANGUAGE_CATALOG
        .iter()
        .filter(|entry| env::var_os(feature_env_var(entry.feature)).is_some())
        .collect();

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo for build scripts");
    let dump_path = Path::new(&out_dir).join("syntaxes.packdump");
    let cache_path = cache_dir().join(format!("{}.packdump", cache_key(&enabled)));

    if let Ok(cached) = fs::read(&cache_path) {
        fs::write(&dump_path, cached)
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", dump_path.display()));
    } else {
        let mut builder = SyntaxSetBuilder::new();
        for entry in &enabled {
            let text = fetch_verified(entry);
            let definition =
                SyntaxDefinition::load_from_str(&text, true, None).unwrap_or_else(|err| {
                    panic!("failed to parse grammar {}: {err}", entry.upstream_path)
                });
            builder.add(definition);
        }
        let syntax_set = builder.build();

        syntect::dumps::dump_to_uncompressed_file(&syntax_set, &dump_path)
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", dump_path.display()));

        // Best-effort - a cache write failure (e.g. read-only `HOME`) shouldn't
        // fail the build, just miss the speedup next time.
        if let Some(parent) = cache_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::copy(&dump_path, &cache_path);
    }

    // Plain feature names, no syntect/grammar data attached - safe for
    // `Language::is_available` to read from the main bundle without pulling
    // the wasm-split-gated `SYNTAX_SET`/packdump back in.
    let enabled_features_src = format!(
        "pub(crate) static ENABLED_LANGUAGE_FEATURES: &[&str] = &[{}];\n",
        enabled
            .iter()
            .map(|entry| format!("{:?}", entry.feature))
            .collect::<Vec<_>>()
            .join(", ")
    );
    fs::write(
        Path::new(&out_dir).join("enabled_languages.rs"),
        enabled_features_src,
    )
    .expect("failed to write enabled_languages.rs");
}
