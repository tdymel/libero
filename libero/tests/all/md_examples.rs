//! `src/md_examples.rs` compiles the ```rust blocks of the docs' md pages as
//! doc-tests, but only the pages it lists. A new page it does not list would
//! go unchecked, so this fails until it is listed.

use std::path::Path;

#[test]
fn every_md_page_is_compiled_as_a_doc_test() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let listed = std::fs::read_to_string(root.join("src/md_examples.rs")).unwrap();
    let pages = std::fs::read_dir(root.join("../docs/public/md")).unwrap();

    let missing: Vec<String> = pages
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
        .filter(|page| !listed.contains(&format!("=> \"{page}\",")))
        .collect();

    assert!(missing.is_empty(), "add to src/md_examples.rs: {missing:?}");
}
