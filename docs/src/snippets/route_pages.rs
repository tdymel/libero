//! `docs/route_pages.tsv`, the head of each route's static HTML file that `route_pages.py`
//! writes at deploy, and the page descriptions it carries.

use super::*;

/// A mirror's `Description:` line.
fn md_description(md: &str) -> Option<&str> {
    md.lines()
        .find_map(|line| line.strip_prefix("Description: "))
}

/// `public/md/index.md`'s rows, `- [Label](file.md): description`, by file.
fn index_rows(index: &str) -> BTreeMap<&str, &str> {
    index
        .lines()
        .filter_map(|line| {
            let (_, rest) = line.strip_prefix("- [")?.split_once("](")?;
            let (file, rest) = rest.split_once(')')?;
            Some((file, rest.strip_prefix(": ")?))
        })
        .collect()
}

fn public() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("public")
}

/// Each mirror's `Description:` is its index row, short enough for a search result snippet.
#[test]
fn page_descriptions_match_the_index_and_fit_a_snippet() {
    let md = public().join("md");
    let index = std::fs::read_to_string(md.join("index.md")).unwrap();
    let rows = index_rows(&index);
    let mut problems = Vec::new();
    for entry in std::fs::read_dir(&md).unwrap() {
        let file = entry.unwrap().file_name().into_string().unwrap();
        if file == "index.md" {
            continue;
        }
        let text = std::fs::read_to_string(md.join(&file)).unwrap();
        let Some(description) = md_description(&text) else {
            problems.push(format!("{file}: no `Description:` line"));
            continue;
        };
        if rows.get(file.as_str()) != Some(&description) {
            problems.push(format!("{file}: `Description:` is not its index.md row"));
        }
        let length = description.replace('`', "").chars().count();
        if length > 160 {
            problems.push(format!(
                "{file}: description is {length} characters, over 160"
            ));
        }
    }
    problems.sort();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// One tab-separated row per static route: path, title, description (none: the shell's),
/// markdown mirror, sidebar section and that section's first page. Rewritten when stale.
#[test]
fn the_route_pages_manifest_is_current() {
    let mut problems = Vec::new();
    let mut expected = String::new();
    for Rendered { route, pages, .. } in rendered() {
        let path = route.to_string();
        let row = if *route == (Route::Home {}) {
            [
                path,
                crate::pages::HOME_TITLE.into(),
                String::new(),
                "/md/index.md".into(),
            ]
        } else {
            let Some(RecordedPage {
                title,
                markdown: Some(markdown),
                ..
            }) = pages.first()
            else {
                problems.push(format!("{path}: no `DocPage` with a markdown mirror"));
                continue;
            };
            let md = std::fs::read_to_string(public().join(markdown.trim_start_matches('/')))
                .unwrap_or_default();
            let Some(description) = md_description(&md) else {
                problems.push(format!("{path}: no `Description:` in {markdown}"));
                continue;
            };
            let description = description.replace('`', "");
            [
                path,
                format!("{title} - Libero"),
                description,
                markdown.clone(),
            ]
        };
        let (group, first) = crate::nav::page_group(route).unwrap_or_default();
        let row = row.join("\t");
        assert!(!row.contains('\n'), "{row}");
        let _ = writeln!(expected, "{row}\t{group}\t{first}");
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));

    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("route_pages.tsv");
    if std::fs::read_to_string(&file).ok().as_deref() != Some(&expected) {
        std::fs::write(&file, &expected).unwrap();
        panic!("docs/route_pages.tsv was stale: rewritten, commit it");
    }
}
