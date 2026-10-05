//! Prop tables against the components' real props.

use super::*;

/// `text` cut at each top-level comma: `<>` nest too, but not the `>` of a `->`.
fn top_level_split(text: &str) -> Vec<&str> {
    let (mut pieces, mut depth, mut start) = (Vec::new(), 0i32, 0);
    let bytes = text.as_bytes();
    for (i, &byte) in bytes.iter().enumerate() {
        match byte {
            b'(' | b'[' | b'{' | b'<' => depth += 1,
            b'>' if i > 0 && bytes[i - 1] == b'-' => {}
            b')' | b']' | b'}' | b'>' => depth -= 1,
            b',' if depth == 0 => {
                pieces.push(&text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    pieces.push(&text[start..]);
    pieces
}

/// The index past the bracket closing the one at `open`.
fn closing(text: &str, open: usize) -> usize {
    let (left, right) = match text.as_bytes()[open] {
        b'(' => (b'(', b')'),
        b'[' => (b'[', b']'),
        _ => (b'{', b'}'),
    };
    let mut depth = 0;
    for (i, &byte) in text.as_bytes()[open..].iter().enumerate() {
        depth += (byte == left) as i32 - (byte == right) as i32;
        if depth == 0 {
            return open + i + 1;
        }
    }
    text.len()
}

/// The fields a builder cannot leave out: no `default` or `extends`, not `Option`, not
/// `children` (Dioxus defaults those), as `dioxus-core-macro`'s props derive decides.
fn required_fields(fields: &str) -> Vec<String> {
    declared_fields(fields)
        .into_iter()
        .filter(|(name, required, _)| *required && name != "children")
        .map(|(name, ..)| name)
        .collect()
}

/// Every field's name, whether a builder cannot leave it out, and whether it is `doc(hidden)`.
fn declared_fields(fields: &str) -> Vec<(String, bool, bool)> {
    top_level_split(fields)
        .into_iter()
        .filter_map(|field| {
            let (mut attrs, mut rest) = (String::new(), field.trim());
            while let Some(attr) = rest.strip_prefix("#[") {
                let end = closing(rest, 1);
                attrs.push_str(&attr[..end - 2]);
                rest = rest[end..].trim_start();
            }
            let rest = rest.strip_prefix("pub ").unwrap_or(rest);
            let (name, ty) = rest.split_once(':')?;
            let (name, ty) = (name.trim(), ty.trim().replace(' ', ""));
            let optional = ["Option<", "ReadSignal<Option<", "ReadOnlySignal<Option<"]
                .iter()
                .any(|prefix| ty.starts_with(prefix))
                && !attrs.replace(' ', "").contains("!optional");
            let defaulted = attrs
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .any(|word| matches!(word, "default" | "extends"));
            let hidden = attrs.replace(' ', "").contains("doc(hidden)");
            (!name.is_empty()).then(|| (name.to_string(), !optional && !defaulted, hidden))
        })
        .collect()
}

/// libero's source files, comments cut, but not a URL's `//`.
fn libero_sources() -> Vec<String> {
    let mut files = Vec::new();
    rust_files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../libero/src"),
        &mut files,
    );
    files
        .iter()
        .map(|file| {
            std::fs::read_to_string(file)
                .unwrap()
                .lines()
                .map(|line| {
                    let cut = line
                        .match_indices("//")
                        .find(|(i, _)| !line[..*i].ends_with(':'))
                        .map_or(line.len(), |(i, _)| i);
                    format!("{}\n", &line[..cut])
                })
                .collect()
        })
        .collect()
}

/// Every libero component's required props, by name: a `#[component]` fn's own parameters
/// or the fields of its `<Name>Props` struct.
fn required_props() -> BTreeMap<String, Vec<String>> {
    let mut required = BTreeMap::new();
    for source in libero_sources() {
        for (at, _) in source.match_indices("#[component]") {
            let after = &source[at..];
            let Some(fn_at) = after.find("fn ") else {
                continue;
            };
            let signature = &after[fn_at + 3..];
            let name: String = signature
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let Some(open) = signature.find('(') else {
                continue;
            };
            let params = &signature[open + 1..closing(signature, open) - 1];
            let fields = required_fields(params);
            // `props: NameProps` is the struct's, found below.
            if !(params.trim().starts_with("props:") && fields.len() == 1) {
                required.insert(name, fields);
            }
        }
        for (at, _) in source.match_indices("struct ") {
            let after = &source[at + 7..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let (Some(component), Some(open)) =
                (name.strip_suffix("Props"), after.find(['{', ';']))
            else {
                continue;
            };
            if after.as_bytes()[open] == b'{' && !component.is_empty() {
                let fields = required_fields(&after[open + 1..closing(after, open) - 1]);
                required.entry(component.to_string()).or_insert(fields);
            }
        }
    }
    required
}

/// Todo 2077: a prop the component cannot do without reads `required` in its page row and
/// its mirror's Default cell.
#[test]
fn required_props_read_required() {
    let required = required_props();
    let public = Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    let mut problems = Vec::new();
    for Rendered { route, pages, .. } in rendered() {
        for RecordedPage {
            markdown,
            properties: groups,
            ..
        } in pages
        {
            let mirrored = markdown
                .as_ref()
                .and_then(|md| {
                    std::fs::read_to_string(public.join(md.trim_start_matches('/'))).ok()
                })
                .map(|md| md_props(&md))
                .unwrap_or_default();
            for group in groups {
                let component = group.component().split('<').next().unwrap_or_default();
                let Some(names) = required.get(component) else {
                    continue;
                };
                let listed = mirrored
                    .iter()
                    .find(|(heading, _)| heading.split('<').next() == Some(component))
                    .or_else(|| {
                        mirrored
                            .iter()
                            .find(|(heading, _)| groups.len() == 1 && heading.is_empty())
                    })
                    .map(|(_, rows)| rows);
                for (name, default) in group.defaults() {
                    if !names.iter().any(|required| required == bare(name)) {
                        continue;
                    }
                    if default != "required" {
                        problems.push(format!("{route}: `{component}` `{name}` reads {default:?}"));
                    }
                    if let Some(cell) = listed.and_then(|rows| rows.get(bare(name)))
                        && cell != "required"
                    {
                        problems.push(format!(
                            "{}: `{component}` `{name}` reads {cell:?}",
                            markdown.as_deref().unwrap_or_default()
                        ));
                    }
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// What `field_props!` adds to a struct: `without(readonly)` and `without(radius)` drop one.
const FIELD_PROPS: [&str; 9] = [
    "label",
    "description",
    "helper",
    "status",
    "size",
    "radius",
    "disabled",
    "required",
    "readonly",
];

/// `(component, part slot)` of a shared part enum the component never draws, so its table
/// leaves it out. A frame's `leading`/`trailing` draw only where `use_field_frame` gets one.
const KNOWN_GAPS: &[(&str, &str)] = &[
    ("Cascader", "leading"),
    ("ChronoField", "leading"),
    // Cascader's and PhoneField's dropdown parts; `Combobox` passes its core no search box.
    ("Combobox", "column"),
    ("Combobox", "dial"),
    ("Combobox", "drill-back"),
    ("Combobox", "name"),
    ("Combobox", "pick-parent"),
    ("Combobox", "search"),
    ("MultiSelect", "leading"),
    ("NativeSelect", "leading"),
    ("NumberField", "leading"),
    ("PasswordField", "leading"),
    ("PinField", "leading"),
    ("PinField", "trailing"),
    // `Slider`'s `track` and `segments`, which `RangeSlider` has no props for.
    ("RangeSlider", "bars"),
    ("RangeSlider", "segment"),
    ("RangeSlider", "segment-fill"),
    ("RangeSlider", "segments"),
    ("RichTextEditor", "leading"),
    ("RichTextEditor", "trailing"),
    // `MultiSelect`'s chips.
    ("Select", "chip"),
    ("Select", "leading"),
    ("TagsField", "leading"),
    ("Textarea", "leading"),
];

/// What `base_props!` adds; the page's tables append all but `parts` themselves.
const BASE_PROPS: [&str; 5] = ["parts", "class", "sx", "states", "attributes"];

/// The fields a page documents: all but the `doc(hidden)` ones.
fn documented(fields: &str) -> BTreeSet<String> {
    declared_fields(fields)
        .into_iter()
        .filter(|(.., hidden)| !hidden)
        .map(|(name, ..)| name)
        .collect()
}

/// Every libero component's own props by name, `field_props!`'s included: a `#[component]`
/// fn's parameters or its `<Name>Props` struct's fields.
fn component_props() -> BTreeMap<String, BTreeSet<String>> {
    let mut props = BTreeMap::new();
    for source in libero_sources() {
        for (at, _) in source.match_indices("#[component]") {
            let after = &source[at..];
            let Some(fn_at) = after.find("fn ") else {
                continue;
            };
            let signature = &after[fn_at + 3..];
            let name: String = signature
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let Some(open) = signature.find('(') else {
                continue;
            };
            let params = &signature[open + 1..closing(signature, open) - 1];
            if !params.trim().starts_with("props:") {
                props.insert(name, documented(params));
            }
        }
        for (at, _) in source.match_indices("struct ") {
            let after = &source[at + 7..];
            let name: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let (Some(component), Some(open)) =
                (name.strip_suffix("Props"), after.find(['{', ';']))
            else {
                continue;
            };
            if after.as_bytes()[open] != b'{' || component.is_empty() {
                continue;
            }
            let mut names = documented(&after[open + 1..closing(after, open) - 1]);
            // The macro's header sits between its `field_props! {` and the struct.
            let before = &source[..at];
            if let Some(macro_at) = before.rfind("field_props! {")
                && !before[macro_at..].contains("struct ")
            {
                let header = &before[macro_at..];
                names.extend(
                    FIELD_PROPS
                        .iter()
                        .filter(|name| !header.contains(&format!("without({name})")))
                        .map(|name| name.to_string()),
                );
            }
            props.entry(component.to_string()).or_insert(names);
        }
    }
    props
}

/// Todo 1926: a page's Properties table names the component's own props, no more, no fewer.
/// The descriptions and defaults stay unchecked.
#[test]
fn prop_tables_name_the_component_props() {
    let declared = component_props();
    let mut problems = Vec::new();
    for Rendered { route, pages, .. } in rendered() {
        for RecordedPage {
            properties: groups, ..
        } in pages
        {
            for group in groups {
                let component = group.component().split('<').next().unwrap_or_default();
                let Some(fields) = declared.get(component) else {
                    continue;
                };
                let page: BTreeSet<&str> = group.names().map(bare).collect();
                for field in fields.iter().filter(|field| !page.contains(field.as_str())) {
                    let known = KNOWN_GAPS.contains(&(component, field.as_str()));
                    if field != "children" && !BASE_PROPS.contains(&field.as_str()) && !known {
                        problems.push(format!("{route}: `{component}` lacks `{field}`"));
                    }
                }
                for name in page.iter().filter(|name| !fields.contains(**name)) {
                    if !BASE_PROPS.contains(name) {
                        problems.push(format!("{route}: `{component}` has no `{name}`"));
                    }
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Todo 1926: a Style API table lists every variant of its part enum.
#[test]
fn part_tables_list_every_part() {
    let mut problems = Vec::new();
    for Rendered { route, pages, .. } in rendered() {
        for RecordedPage {
            properties: groups, ..
        } in pages
        {
            for group in groups {
                let component = group.component().split('<').next().unwrap_or_default();
                for slot in group.unlisted_parts() {
                    if !KNOWN_GAPS.contains(&(component, slot)) {
                        problems.push(format!("{route}: `{component}` lacks the `{slot}` part"));
                    }
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
