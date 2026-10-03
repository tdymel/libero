//! The `docs_shell` fixture copies `docs/src/heading_focus.rs` (todo 951), so the docs'
//! route-change focus is tested on the copy; this keeps the copied functions equal (1802).

const DOCS: &str = include_str!("../../docs/src/heading_focus.rs");
const COPY: &str = include_str!("../fixtures/src/docs_shell.rs");

/// The functions both files hold; `use_fragment_landing` differs by design (the id source).
const SHARED: &[&str] = &[
    "use_scroll_reset",
    "scroll_to_section",
    "use_heading_focus",
    "focus_target",
    "restore_fragment",
];

/// The item `fn name` from its signature to its closing brace, comments and indent dropped.
fn item(source: &str, name: &str) -> Option<String> {
    let start = [format!("fn {name}("), format!("fn {name}<")]
        .iter()
        .filter_map(|signature| source.find(signature.as_str()))
        .min()?;
    let end = start + source[start..].find("\n}\n")?;
    Some(
        source[start..end]
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[test]
fn the_fixture_copy_matches_the_docs() {
    for name in SHARED {
        let docs = item(DOCS, name).unwrap_or_else(|| panic!("the docs lost `fn {name}`"));
        let copy = item(COPY, name).unwrap_or_else(|| panic!("the copy lost `fn {name}`"));
        assert_eq!(
            copy, docs,
            "`fn {name}` in e2e/fixtures/src/docs_shell.rs drifted from docs/src/heading_focus.rs"
        );
    }
}
