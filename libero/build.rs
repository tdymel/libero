//! Fetches only the `.sublime-syntax` grammars enabled via `code-lang-*`
//! Cargo features from the pinned `sublimehq/Packages` commit, verifies each
//! against a hardcoded SHA-256, and compiles them into a single trimmed
//! `SyntaxSet` dump embedded by `highlight.rs` via `include_bytes!`. This
//! keeps the binary limited to the languages actually opted into, without
//! vendoring grammar sources in this repo.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use syntect::parsing::{SyntaxDefinition, SyntaxSetBuilder};

// The commit `sublimehq/Packages` was pinned at for syntect 5.3.0's own
// `testdata/Packages` submodule (https://github.com/trishume/syntect/tree/
// v5.3.0/testdata) - current `master` grammars use newer `.sublime-syntax`
// spec features syntect 5.3.0's YAML loader doesn't understand.
const PINNED_COMMIT: &str = "fa6b8629c95041bf262d4c1dab95c456a0530122";

struct LanguageSource {
    /// The `CARGO_FEATURE_*` env var Cargo sets when the matching
    /// `code-lang-*` feature is enabled.
    feature_env: &'static str,
    /// Path within the pinned `sublimehq/Packages` commit.
    upstream_path: &'static str,
    /// SHA-256 of the file at that path and commit, checked before the
    /// download is trusted.
    sha256: &'static str,
}

const LANGUAGES: &[LanguageSource] = &[
    LanguageSource {
        feature_env: "CARGO_FEATURE_CODE_LANG_RUST",
        upstream_path: "Rust/Rust.sublime-syntax",
        sha256: "586805ace8352f932a04b3b79f9eff4e2487e9e3e4baadf0d191ff155ce73c42",
    },
    LanguageSource {
        feature_env: "CARGO_FEATURE_CODE_LANG_SHELL",
        upstream_path: "ShellScript/Bash.sublime-syntax",
        sha256: "aee7f4688f8d3bea30cbf69a82d04afd629062e573c2b68d35b31172b9e917c9",
    },
    LanguageSource {
        feature_env: "CARGO_FEATURE_CODE_LANG_MARKDOWN",
        upstream_path: "Markdown/Markdown.sublime-syntax",
        sha256: "fe045101a3be4d2ce2773095a323da4296950826e6fa4129ca34f4c7e5c58967",
    },
    LanguageSource {
        feature_env: "CARGO_FEATURE_CODE_LANG_HTML",
        upstream_path: "HTML/HTML.sublime-syntax",
        sha256: "8422f400e06b4c1054627c60a4f1109682a8aa2deadc2576004cb802e5b4b3eb",
    },
    LanguageSource {
        feature_env: "CARGO_FEATURE_CODE_LANG_CSS",
        upstream_path: "CSS/CSS.sublime-syntax",
        sha256: "377cf1623784e1896c1359d87de4167bd74ae62356a987c8d326fcaf551107e7",
    },
];

fn fetch_verified(source: &LanguageSource) -> String {
    let url = format!(
        "https://raw.githubusercontent.com/sublimehq/Packages/{PINNED_COMMIT}/{}",
        source.upstream_path
    );
    let text = ureq::get(&url)
        .call()
        .unwrap_or_else(|err| panic!("failed to fetch {url}: {err}"))
        .body_mut()
        .read_to_string()
        .unwrap_or_else(|err| panic!("failed to read response body from {url}: {err}"));

    let actual = hex_encode(&Sha256::digest(text.as_bytes()));
    assert!(
        actual == source.sha256,
        "checksum mismatch for {url}: expected {}, got {actual} - upstream file may have \
         changed or the download was tampered with",
        source.sha256,
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

/// Changes whenever `PINNED_COMMIT` moves or the enabled language set
/// changes, so a stale cache entry is simply never looked up again rather
/// than needing explicit invalidation.
fn cache_key(enabled: &[&LanguageSource]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(PINNED_COMMIT.as_bytes());
    for source in enabled {
        hasher.update(source.feature_env.as_bytes());
        hasher.update(source.sha256.as_bytes());
    }
    hex_encode(&hasher.finalize())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let enabled: Vec<&LanguageSource> = LANGUAGES
        .iter()
        .filter(|source| env::var_os(source.feature_env).is_some())
        .collect();

    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo for build scripts");
    let dump_path = Path::new(&out_dir).join("syntaxes.packdump");
    let cache_path = cache_dir().join(format!("{}.packdump", cache_key(&enabled)));

    if let Ok(cached) = fs::read(&cache_path) {
        fs::write(&dump_path, cached)
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", dump_path.display()));
        return;
    }

    let mut builder = SyntaxSetBuilder::new();
    for source in &enabled {
        let text = fetch_verified(source);
        let definition = SyntaxDefinition::load_from_str(&text, true, None).unwrap_or_else(|err| {
            panic!("failed to parse grammar {}: {err}", source.upstream_path)
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
