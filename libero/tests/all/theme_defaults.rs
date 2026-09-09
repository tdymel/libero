//! Todo 392: a `*Defaults` holding a literal CSS colour looks right on the
//! light page only, unless `Theme::DARK` restates it. A source scan, so a
//! component no test renders is covered too.

use std::{fs, path::Path};

/// Literal colours that are right on either page, because they are not drawn
/// on it: the image list's bar lies on a photo, and a QR code must stay dark on
/// light for a scanner.
const SCHEME_INDEPENDENT: &[&str] = &["ImageListDefaults", "QrCodeDefaults"];

/// The string literals on `line` that name a CSS colour: `#rgb`..`#rrggbbaa`,
/// or an `rgb()`/`rgba()`/`hsl()`/`hsla()`.
fn colour_literals(line: &str) -> Vec<&str> {
    line.split('"')
        .skip(1)
        .step_by(2)
        .filter(|literal| {
            let hex = literal
                .split(|c: char| !(c == '#' || c.is_ascii_hexdigit()))
                .any(|word| {
                    word.strip_prefix('#').is_some_and(|digits| {
                        matches!(digits.len(), 3 | 4 | 6 | 8)
                            && digits.chars().all(|c| c.is_ascii_hexdigit())
                    })
                });
            hex || ["rgb(", "rgba(", "hsl(", "hsla("]
                .iter()
                .any(|function| literal.contains(function))
        })
        .collect()
}

/// `Theme::DARK`'s body, where a `*Defaults::DARK` has to be named.
fn theme_dark(theme_rs: &str) -> &str {
    let start = theme_rs
        .find("pub const DARK: Theme = Theme {")
        .expect("Theme::DARK in theme.rs");
    let body = &theme_rs[start..];
    &body[..body.find("};").expect("Theme::DARK ends")]
}

/// Every colour literal in `source` that its `*Defaults`' `DARK` does not
/// restate by field, or whose `DARK` `theme_dark` does not use. A `DARK` that
/// is `Self::DEFAULT` restates nothing. Tests below `#[cfg(test)]` are skipped.
fn literals_without_dark(file: &str, source: &str, theme_dark: &str) -> Vec<String> {
    let source = source.split("#[cfg(test)]").next().unwrap_or_default();
    let first_struct = source
        .lines()
        .find_map(|line| line.trim().strip_prefix("pub struct "))
        .and_then(|rest| rest.split([' ', '{', '<', '(']).next());
    let mut owner = first_struct;
    // The `DARK` block's lines, once the scan is inside one.
    let mut dark_fields: Option<Vec<&str>> = None;
    let mut in_dark = false;
    let mut hits = Vec::new();

    for (index, line) in source.lines().enumerate() {
        let code = line.trim();
        if let Some(rest) = code.strip_prefix("impl ")
            && let Some(name) = rest.strip_suffix(" {").filter(|name| !name.contains(' '))
        {
            owner = Some(name);
            dark_fields = dark_block(source, index);
        }
        let dark_header = code.starts_with("pub const DARK: Self");
        if dark_header {
            in_dark = !code.ends_with(';');
        } else if in_dark && code == "};" {
            in_dark = false;
        }
        if dark_header || in_dark || code.starts_with("//") {
            continue;
        }
        let literals = colour_literals(code);
        if literals.is_empty() {
            continue;
        }
        let Some(owner) = owner else {
            hits.push(format!("{file}:{}: {code} (no struct)", index + 1));
            continue;
        };
        if SCHEME_INDEPENDENT.contains(&owner) {
            continue;
        }
        let field = code.split(':').next().unwrap_or_default();
        let restated = dark_fields
            .as_ref()
            .is_some_and(|fields| fields.contains(&field));
        let has_dark = restated && theme_dark.contains(&format!("{owner}::DARK"));
        if !has_dark {
            hits.push(format!(
                "{file}:{}: {owner} {}",
                index + 1,
                literals.join(" ")
            ));
        }
    }
    hits
}

/// The field names the first `DARK` after line `from` sets, or `None` when
/// there is no `DARK` at all.
fn dark_block(source: &str, from: usize) -> Option<Vec<&str>> {
    let mut lines = source.lines().skip(from).map(str::trim);
    let header = lines.find(|code| code.starts_with("pub const DARK: Self"))?;
    fn field(code: &str) -> Option<&str> {
        code.split_once(':')
            .map(|(field, _)| field.trim_start_matches(['{', ' ']))
    }
    // One line, `Self::DEFAULT` or `Self { a: .., b: .. }`.
    if header.ends_with(';') {
        let body = header.split_once('{').map_or("", |(_, body)| body);
        return Some(body.split(',').filter_map(field).collect());
    }
    Some(
        lines
            .take_while(|code| *code != "};")
            .filter_map(field)
            .collect(),
    )
}

#[test]
fn every_literal_colour_in_a_defaults_has_a_dark_counterpart() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/theme");
    let theme_rs = fs::read_to_string(root.join("theme.rs")).unwrap();
    let dark = theme_dark(&theme_rs);
    let mut hits = Vec::new();

    for entry in fs::read_dir(root.join("defaults")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        let source = fs::read_to_string(&path).unwrap();
        hits.extend(literals_without_dark(&file, &source, dark));
    }
    hits.sort();

    assert!(
        hits.is_empty(),
        "a literal colour is only right on the light page: derive it from a theme \
         colour (`var(--lsx-muted-N)`), or add `DARK` and name it in `Theme::DARK`:\n{}",
        hits.join("\n")
    );
}

/// The scan itself, so the test above cannot pass by finding nothing.
#[test]
fn a_new_literal_without_dark_is_caught() {
    let dark = "pub const DARK: Theme = Theme { paper: PaperDefaults::DARK, };";
    let source = r##"
pub struct WidgetDefaults { pub background: &'static str }
impl WidgetDefaults {
    pub const DEFAULT: Self = Self {
        background: "#f6f8fa",
    };
}
"##;
    let caught = ["widget.rs:5: WidgetDefaults #f6f8fa"];
    assert_eq!(literals_without_dark("widget.rs", source, dark), caught);

    let with_dark = |dark_const: &str| source.replace("\n}\n", &format!("\n{dark_const}\n}}\n"));
    let dark = "pub const DARK: Theme = Theme { widget: WidgetDefaults::DARK, };";

    // A `DARK` that only repeats `DEFAULT` keeps the light literal.
    let inherited = with_dark("    pub const DARK: Self = Self::DEFAULT;");
    assert_eq!(literals_without_dark("widget.rs", &inherited, dark), caught);

    let restated =
        with_dark("    pub const DARK: Self = Self {\n        background: \"#161b22\",\n    };");
    assert!(literals_without_dark("widget.rs", &restated, dark).is_empty());
}

#[test]
fn a_var_is_not_a_colour_but_a_hex_in_a_shorthand_is() {
    assert!(colour_literals(r##"background: "var(--lsx-muted-1)","##).is_empty());
    assert!(colour_literals(r##"id: "#main","##).is_empty());
    assert_eq!(
        colour_literals(r##"border: "1px solid #d0d7de","##),
        ["1px solid #d0d7de"]
    );
    assert_eq!(
        colour_literals(r##"hover: "rgba(31, 35, 40, 0.08)","##),
        ["rgba(31, 35, 40, 0.08)"]
    );
}
