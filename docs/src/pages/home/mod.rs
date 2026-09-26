//! The docs site's landing page at `/`: one module per section, top to bottom.

mod batteries;
mod booking;
mod closing;
mod example;
mod hero;
mod stats;

use dioxus::prelude::*;
use libero::{
    components::{Flex, OptionLabel, Title, VisuallyHidden},
    sx::{Sx, sx},
    theme::{ColorCss, ColorShade},
};

#[component]
pub fn Home() -> Element {
    rsx! {
        document::Title { "Libero - accessible, themeable components for Dioxus" }
        Flex { direction: "column", gap: "xxl", sx: sx().padding_bottom("lg"),
            hero::Hero {}
            stats::Stats {}
            example::Example {}
            batteries::Batteries {}
            closing::Closing {}
        }
    }
}

/// The palette's `color` at `percent` over transparent: a tint that
/// follows the palette and the scheme.
pub(super) fn tint(color: ColorCss, percent: u8) -> String {
    format!(
        "color-mix(in srgb, {} {percent}%, transparent)",
        color.value(ColorShade::S6)
    )
}

/// A row of buttons in which one that cannot share a line grows to the full width.
pub(super) fn cta_row_sx() -> Sx {
    sx().selector("& > *", sx().flex("1 1 auto"))
}

/// A tab's label with `icon` before its name; the name is still what names the tab.
pub(super) fn icon_label(name: String, icon: Element) -> OptionLabel {
    OptionLabel::rich(
        name.clone(),
        rsx! {
            span { display: "inline-flex", align_items: "center", gap: "6px",
                span { display: "inline-flex", width: "16px", height: "16px", {icon} }
                "{name}"
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    /// The trimmed lines between `// copy: <name>` and `// copy: end`, per name.
    fn regions(source: &str) -> BTreeMap<String, Vec<String>> {
        let mut found = BTreeMap::new();
        let mut open: Option<(String, Vec<String>)> = None;
        for line in source.lines().map(str::trim) {
            match (line.strip_prefix("// copy: "), open.as_mut()) {
                (Some("end"), _) => {
                    let (name, lines) = open.take().expect("`// copy: end` without a region");
                    found.insert(name, lines);
                }
                (Some(name), _) => open = Some((name.to_string(), Vec::new())),
                (None, Some((_, lines))) => lines.push(line.to_string()),
                (None, None) => {}
            }
        }
        found
    }

    /// Todos 1031 and 1049: the e2e fixture's hand copies of the booking card, the stats heading and the code blocks.
    #[test]
    fn the_e2e_copies_match_their_docs_source() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let read = |path: &str| std::fs::read_to_string(root.join(path)).unwrap();
        let mut original = BTreeMap::new();
        for file in ["booking", "stats", "hero", "closing", "example"] {
            for (name, lines) in regions(&read(&format!("src/pages/home/{file}.rs"))) {
                // A region several files share must be the same in each.
                if let Some(seen) = original.insert(name.clone(), lines.clone()) {
                    assert_eq!(seen, lines, "region `{name}` differs between home pages");
                }
            }
        }
        let copy = regions(&read("../e2e/fixtures/src/home.rs"));
        assert!(!original.is_empty());
        assert_eq!(
            original.keys().collect::<Vec<_>>(),
            copy.keys().collect::<Vec<_>>()
        );
        for (name, lines) in &original {
            assert_eq!(
                lines, &copy[name],
                "region `{name}` differs from the e2e fixture"
            );
        }
    }
}

/// A section's `h2`, for screen readers only (todo 1206): the sections show no subtitle.
#[component]
pub(super) fn SectionTitle(id: &'static str, children: Element) -> Element {
    rsx! {
        Title { component: "h2", id, sx: sx().margin("0"),
            VisuallyHidden { {children} }
        }
    }
}
