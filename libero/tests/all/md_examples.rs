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

/// Every page's Source link, in the page and in its md copy, names a path that
/// exists; a module split left three dead (todo 1567).
#[test]
fn every_source_link_resolves() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut sources = Vec::new();
    let mut dirs = vec![repo.join("docs/src/pages")];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            sources.extend(text.lines().filter_map(|line| {
                let value = line
                    .trim()
                    .strip_prefix("source: \"")?
                    .strip_suffix("\",")?;
                (value.contains('/') && !value.contains(' '))
                    .then(|| (path.clone(), value.to_string()))
            }));
        }
    }
    for entry in std::fs::read_dir(repo.join("docs/public/md")).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        sources.extend(text.lines().filter_map(|line| {
            let value = line
                .strip_prefix("Source: <https://github.com/tdymel/libero/tree/main/")?
                .strip_suffix('>')?;
            Some((path.clone(), value.to_string()))
        }));
    }

    assert!(
        sources.len() > 200,
        "found only {} source links",
        sources.len()
    );
    let dead: Vec<String> = sources
        .iter()
        .filter(|(_, source)| !repo.join(source).exists())
        .map(|(page, source)| format!("{}: {source}", page.display()))
        .collect();
    assert!(dead.is_empty(), "dead source links: {dead:#?}");
}
