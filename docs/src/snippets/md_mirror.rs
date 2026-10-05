//! The markdown mirrors against their pages: parts, props and the nav.

use super::*;

/// The `(part, data-slot)` rows of a mirror's `## Style API` tables.
fn md_parts(md: &str) -> BTreeSet<(String, String)> {
    let mut section = "";
    let mut rows = BTreeSet::new();
    for line in md.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            section = heading.trim();
        } else if section == "Style API"
            && let Some(row) = line.strip_prefix("| `")
            && let [part, _, slot, ..] = row.split('`').collect::<Vec<_>>()[..]
        {
            rows.insert((part.to_string(), slot.to_string()));
        }
    }
    rows
}

/// The page's Style API tab and its mirror's `## Style API` list the same parts and slots.
#[test]
fn md_mirrors_list_the_parts_of_their_page() {
    let public = Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    let mut problems = Vec::new();
    for Rendered { route, pages, .. } in rendered() {
        for RecordedPage {
            markdown,
            properties: groups,
            ..
        } in pages
        {
            let Some(markdown) = markdown else { continue };
            let Ok(md) = std::fs::read_to_string(public.join(markdown.trim_start_matches('/')))
            else {
                continue;
            };
            let page: BTreeSet<(String, String)> = groups
                .iter()
                .flat_map(|group| group.part_slots())
                .map(|(part, slot)| (part.to_string(), slot.to_string()))
                .collect();
            let mirrored = md_parts(&md);
            for (part, slot) in page.difference(&mirrored) {
                problems.push(format!("{markdown}: lacks `{part}` / `{slot}`"));
            }
            for (part, slot) in mirrored.difference(&page) {
                problems.push(format!(
                    "{markdown}: `{part}` / `{slot}` is not on the {route} page"
                ));
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Todo 1700: every route but the home page has a sidebar entry (so a search hit and a pager
/// link), and its one `DocPage` is titled with that entry's label (an "Overview" its section's).
#[test]
fn every_page_is_in_the_nav_under_its_title() {
    let mut problems = Vec::new();
    for Rendered { route, pages, .. } in rendered() {
        if *route == (Route::Home {}) {
            continue;
        }
        let Some(title) = crate::nav::page_title(route) else {
            problems.push(format!("{route}: not in the nav"));
            continue;
        };
        match &pages[..] {
            [page] if page.title == title => {}
            [page] => problems.push(format!(
                "{route}: titled {:?}, the nav says {title:?}",
                page.title
            )),
            _ => problems.push(format!("{route}: {} `DocPage`s", pages.len())),
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The names in a mirror's `## Props` and `## API` tables with their third cell (the default),
/// per `###` group; `""` names the
/// `## Props` ones before any. `## API` files options structs and handles by heading; a handle may also have a `## ` section
/// or a table headed by its type.
pub(super) fn md_props(md: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut groups: Vec<(String, BTreeMap<String, String>)> = Vec::new();
    let mut section = "";
    // The group the next row joins: none until a heading (or, in `## Props`, a row) opens one.
    let mut current: Option<usize> = None;
    let open = |groups: &mut Vec<(String, BTreeMap<String, String>)>, name: &str| {
        groups.push((name.to_string(), BTreeMap::new()));
        Some(groups.len() - 1)
    };
    for line in md.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            section = heading.trim();
            current = None;
            // A handle may have a `## ` section of its own, named by its type.
            if section.trim_matches('`').ends_with("Handle") {
                current = open(&mut groups, section.trim_matches('`'));
            }
        } else if current.is_none() && !matches!(section, "Props" | "API") {
            continue;
        } else if let Some(heading) = line.strip_prefix("### ") {
            if matches!(section, "Props" | "API") {
                current = open(&mut groups, heading.trim().trim_matches('`'));
            }
        } else if let Some(row) = line.strip_prefix("| `") {
            let Some((name, _)) = row.split_once('`') else {
                continue;
            };
            // A table may head its first column with its type: `| `FooHandle` | Description |`.
            if name.starts_with(|c: char| c.is_uppercase()) {
                current = open(&mut groups, name);
                continue;
            }
            if current.is_none() && section == "Props" {
                current = open(&mut groups, "");
            }
            if let Some(group) = current {
                let default = line.split(" | ").nth(2).unwrap_or_default().trim();
                groups[group]
                    .1
                    .insert(bare(name).to_string(), default.to_string());
            }
        }
    }
    groups
}

/// Todo 1038: the page's property tables and its `public/md` mirror name the same props.
#[test]
fn md_mirrors_list_the_props_of_their_page() {
    let public = Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    let mut problems = Vec::new();
    for Rendered { route, pages, .. } in rendered() {
        for RecordedPage {
            markdown,
            properties: groups,
            ..
        } in pages
        {
            let Some(markdown) = markdown.as_ref().filter(|_| !groups.is_empty()) else {
                continue;
            };
            let Ok(md) = std::fs::read_to_string(public.join(markdown.trim_start_matches('/')))
            else {
                problems.push(format!("{route}: no mirror at {markdown}"));
                continue;
            };
            let mirrored = md_props(&md);
            for group in groups {
                // A group the mirror files under another heading, such as an options
                // struct, is not compared; a page's only group may sit under none.
                let named = |heading: &str| {
                    heading.split('<').next() == group.component().split('<').next()
                };
                let listed = mirrored
                    .iter()
                    .find(|(heading, _)| named(heading))
                    .or_else(|| {
                        mirrored
                            .iter()
                            .find(|(heading, _)| groups.len() == 1 && heading.is_empty())
                    })
                    .map(|(_, names)| names);
                let Some(listed) = listed else {
                    if group.component().ends_with("Handle") {
                        problems.push(format!("{markdown}: no table for `{}`", group.component()));
                    }
                    continue;
                };
                let page: BTreeSet<&str> = group.names().map(bare).collect();
                for name in page.iter().filter(|name| !listed.contains_key(**name)) {
                    problems.push(format!(
                        "{markdown}: `{}` lacks `{name}`",
                        group.component()
                    ));
                }
                for name in listed.keys().filter(|name| !page.contains(name.as_str())) {
                    problems.push(format!("{markdown}: `{name}` is not on the {route} page"));
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
