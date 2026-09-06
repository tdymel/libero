//! Every prop that writes an `-override` var has a reader.
//!
//! A prop writes `--lsx-x-override`, and the component's CSS has to read it
//! through `overridable()`: `var(--lsx-x-override, var(--lsx-x))`. A static
//! that reads `value()` instead ignores the twin, so the prop compiles,
//! renders and does nothing. `Carousel`'s `per_view` and `gap` shipped like
//! that (todo 49, and the third trap in the brain's `css-vars` note).
//!
//! Two checks close the class. The fixture renders every writer with its prop
//! set, and every `-override` var it writes must be read by the emitted CSS.
//! And every var the library source calls `.override_var()` on must be
//! written by the fixture, so a new writer fails here until it is added.
//!
//! It checks names, not placement: a reader on an element the writer's
//! `style` does not reach (a sibling, a portal) still passes.

use crate::common::render;

use std::collections::BTreeSet;
use std::path::Path;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Alert, AspectRatio, Avatar, Badge, Burger, Carousel, Center, Collapse,
        Container, Dialog, Float, Header, Icon, Image, ImageItem, ImageList, Indicator, Marquee,
        Overlay, Scroller, Tooltip,
    },
};

fn items() -> Vec<ImageItem> {
    vec![
        ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } }),
        ImageItem::new(rsx! { Image { src: "/b.svg", alt: "B" } }),
    ]
}

fn quilted_items() -> Vec<ImageItem> {
    vec![ImageItem::new(rsx! { Image { src: "/a.svg", alt: "A" } }).rows(2)]
}

/// One of each writer, every override prop set.
fn app() -> Element {
    rsx! {
        LiberoProvider {
            ActionIcon { aria_label: "Edit", size: "lg", radius: "sm", "e" }
            Alert { radius: "sm", "alert" }
            AspectRatio { ratio: 1.5, "ratio" }
            Avatar { name: "Ada Lovelace", initials: "AL", radius: "sm" }
            Badge { radius: "sm", "badge" }
            Burger { size: "lg", "aria-label": "Menu" }
            Carousel {
                aria_label: "Offers",
                per_view: 2.0,
                gap: "sm",
                slides: vec![rsx! { "a" }, rsx! { "b" }],
            }
            Center { inline: true, "center" }
            Collapse { open: true, duration: 350, "panel" }
            Container { size: "sm", gutters: "lg", "container" }
            Dialog { aria_label: "Inline", size: "lg", "dialog" }
            Float { z_index: "5", "float" }
            Header { size: "lg", z_index: "5", "header" }
            Icon { size: "lg", "i" }
            Image { src: "/a.svg", alt: "A", radius: "sm" }
            ImageList { ratio: 1.5, items: items() }
            ImageList { variant: "quilted", items: quilted_items() }
            Indicator { radius: "sm" }
            Marquee { duration: 9000, "marquee" }
            Overlay { opacity: "0.5", z_index: "5", blur: "2px" }
            Scroller { aria_label: "Tags", fade_color: "red", span { "one" } }
            Tooltip { label: rsx! { "t" }, z_index: "5", "x" }
        }
    }
}

/// Every `--lsx-*-override` name written in `text` as a declaration
/// (`--x-override:`), in a `style` attribute or in a CSS rule.
fn written_in(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut rest = text;
    while let Some(end) = rest.find("-override:") {
        let head = &rest[..end];
        if let Some(start) = head.rfind("--lsx-") {
            let name = &head[start..];
            if name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            {
                names.insert(format!("{name}-override"));
            }
        }
        rest = &rest[end + "-override:".len()..];
    }
    names
}

/// The `<style>` blocks only: the CSS, where a reader has to be.
fn stylesheet(html: &str) -> String {
    html.split("<style")
        .skip(1)
        .filter_map(|block| block.split("</style>").next())
        .collect()
}

/// Every name the library calls `.override_var()` on, from the source: the
/// identifier before the call, looked up in its `CssVar::new` or
/// `SizeCss::new` declaration, and turned into its twin the way `tokens`
/// does.
fn override_vars_in_source() -> BTreeSet<String> {
    let mut sources = Vec::new();
    collect_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut sources,
    );

    let mut idents = BTreeSet::new();
    for source in &sources {
        let mut rest = source.as_str();
        while let Some(at) = rest.find(".override_var()") {
            let ident: String = rest[..at]
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if !ident.is_empty() {
                idents.insert(ident);
            }
            rest = &rest[at + 1..];
        }
    }

    idents
        .into_iter()
        .map(|ident| {
            let name = sources
                .iter()
                .find_map(|source| declared_name(source, &ident))
                .unwrap_or_else(|| panic!("no `const {ident}` declaring a `--lsx-*` name"));
            format!("{}-override", name.trim_end_matches('-'))
        })
        .collect()
}

/// `"--lsx-x"` out of `const IDENT: CssVar = CssVar::new("--lsx-x");`.
fn declared_name(source: &str, ident: &str) -> Option<String> {
    let decl = format!("const {ident}: ");
    let after = &source[source.find(&decl)? + decl.len()..];
    let line = after.lines().next()?;
    let start = line.find("::new(\"")? + "::new(\"".len();
    let end = line[start..].find('"')?;
    Some(line[start..start + end].to_string())
}

fn collect_sources(dir: &Path, sources: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("readable src dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_sources(&path, sources);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            sources.push(std::fs::read_to_string(&path).expect("readable source"));
        }
    }
}

#[test]
fn every_override_var_written_is_read_by_the_css() {
    let html = render(app);
    let css = stylesheet(&html);
    let written = written_in(&html);

    let unread: Vec<_> = written
        .iter()
        .filter(|name| !css.contains(&format!("var({name}")))
        .collect();
    assert!(
        unread.is_empty(),
        "a prop writes these and no CSS reads them, so it does nothing: {unread:?}"
    );
}

#[test]
fn every_override_writer_in_the_source_is_in_the_fixture() {
    let html = render(app);
    let written = written_in(&html);
    let in_source = override_vars_in_source();

    let missing: Vec<_> = in_source.difference(&written).collect();
    assert!(
        missing.is_empty(),
        "the library writes these, but the fixture above never renders them - \
         add the component with its prop set: {missing:?}\nwritten: {written:?}"
    );
}
