//! Renders a component to HTML and picks it apart, so a test can assert on
//! one element's attributes instead of matching against the whole document.

use std::collections::BTreeMap;

use dioxus::prelude::*;

/// Renders `app` the way a browser would end up seeing it.
///
/// Two passes are required: components register their CSS while they render,
/// which is *after* `LiberoProvider` rendered its `<style>` blocks, so the
/// first pass carries the theme CSS but none of the component CSS. The
/// registry bumps a signal, and the second pass picks it up.
pub fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(&dom)
}

/// The rendered markup with the `<style>` blocks stripped - the CSS is
/// bigger than the markup and full of words like `first-child`, so anything
/// searching for text has to look past it.
pub fn body(html: &str) -> &str {
    match html.rfind("</style>") {
        Some(end) => &html[end + "</style>".len()..],
        None => html,
    }
}

/// Every attribute of the first `<tag>` in `html`.
pub fn attributes_of(html: &str, tag: &str) -> BTreeMap<String, String> {
    let start = html
        .find(&format!("<{tag}"))
        .unwrap_or_else(|| panic!("no <{tag}> in the rendered output:\n{html}"));
    let open_tag = &html[start..][..html[start..].find('>').expect("an unterminated tag")];

    let mut attributes = BTreeMap::new();
    let mut rest = &open_tag[format!("<{tag}").len()..];

    while let Some(equals) = rest.find("=\"") {
        let name = rest[..equals].trim();
        let value_start = equals + 2;
        let value_end = value_start
            + rest[value_start..]
                .find('"')
                .expect("an unterminated attribute value");
        attributes.insert(name.to_string(), rest[value_start..value_end].to_string());
        rest = &rest[value_end + 1..];
    }

    attributes
}

/// The classes on the first `<tag>`, in the order they were assembled.
pub fn classes_of(html: &str, tag: &str) -> Vec<String> {
    attributes_of(html, tag)
        .get("class")
        .map(|class| class.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

/// Whether `class` was actually emitted as a rule, not just referenced by
/// the element - the two come from one hash, so a mismatch means the
/// stylesheet registry and the element disagree.
pub fn has_rule_for(html: &str, class: &str) -> bool {
    html.contains(&format!(".{class}"))
}
