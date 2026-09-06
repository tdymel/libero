//! Renders a component to HTML and picks it apart, so a test can assert on
//! one element's attributes instead of matching against the whole document.

use std::collections::BTreeMap;

use dioxus::prelude::*;

/// Renders `app` the way a browser would end up seeing it.
///
/// Two passes, because a component whose output depends on an effect or an
/// async resource has not produced it yet on the creating render - `Select`
/// only applies its `value` once `use_effect` has flipped `mounted`, and
/// `QrCode` has nothing to draw until its `use_resource` resolves. The CSS
/// itself no longer needs the second pass: `StyleOutlet` renders after
/// `{children}`, so the first pass already carries every registered sheet.
pub fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(&dom)
}

/// The rendered markup with the `<style>` blocks stripped - the CSS is
/// bigger than the markup and full of words like `first-child`, so anything
/// searching for text has to look past it. They are removed wherever they
/// sit, since `StyleOutlet` renders the registered sheets *after* the
/// markup while the static ones still come before it.
pub fn body(html: &str) -> String {
    let mut body = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find("<style") {
        let Some(end) = rest[start..].find("</style>") else {
            break;
        };
        body.push_str(&rest[..start]);
        rest = &rest[start + end + "</style>".len()..];
    }
    body.push_str(rest);

    body
}

/// Every attribute of the first `<tag>` in `html`.
///
/// SSR quotes a string value (`id="x"`) but writes a boolean bare
/// (`disabled=true`), so both forms are read. A tokenizer that only knew the
/// quoted form made every `!contains_key("disabled")` pass unconditionally
/// (review 7, S1). A valueless attribute is recorded with an empty value.
pub fn attributes_of(html: &str, tag: &str) -> BTreeMap<String, String> {
    let start = html
        .find(&format!("<{tag}"))
        .unwrap_or_else(|| panic!("no <{tag}> in the rendered output:\n{html}"));
    let mut rest = &html[start + format!("<{tag}").len()..];
    let mut attributes = BTreeMap::new();

    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            panic!("an unterminated <{tag}> tag");
        }
        if rest.starts_with('>') || rest.starts_with("/>") {
            break;
        }

        let name_end = rest
            .find(|c: char| c.is_whitespace() || matches!(c, '=' | '>' | '/'))
            .unwrap_or(rest.len());
        assert!(
            name_end > 0,
            "an attribute without a name in <{tag}>: {rest}"
        );
        let name = &rest[..name_end];
        rest = &rest[name_end..];

        let value = if let Some(after_equals) = rest.strip_prefix('=') {
            if let Some(quoted) = after_equals.strip_prefix('"') {
                let end = quoted.find('"').expect("an unterminated attribute value");
                rest = &quoted[end + 1..];
                &quoted[..end]
            } else {
                let end = after_equals
                    .find(|c: char| c.is_whitespace() || c == '>')
                    .unwrap_or(after_equals.len());
                rest = &after_equals[end..];
                &after_equals[..end]
            }
        } else {
            ""
        };
        attributes.insert(name.to_string(), value.to_string());
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

/// A picked file with a name and nothing to read, for drawing a `FileField`'s
/// selection. Nothing in SSR reads its contents.
pub struct FakeFile(pub &'static str);

impl dioxus::html::NativeFileData for FakeFile {
    fn name(&self) -> String {
        self.0.to_string()
    }
    fn size(&self) -> u64 {
        0
    }
    fn last_modified(&self) -> u64 {
        0
    }
    fn path(&self) -> std::path::PathBuf {
        self.0.into()
    }
    fn content_type(&self) -> Option<String> {
        None
    }
    fn read_bytes(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = Result<dioxus::html::bytes::Bytes, dioxus::CapturedError>,
                >,
        >,
    > {
        unimplemented!("SSR never reads a file")
    }
    fn byte_stream(
        &self,
    ) -> std::pin::Pin<
        Box<
            dyn futures_core::Stream<
                    Item = Result<dioxus::html::bytes::Bytes, dioxus::CapturedError>,
                > + Send,
        >,
    > {
        unimplemented!("SSR never reads a file")
    }
    fn read_string(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, dioxus::CapturedError>>>>
    {
        unimplemented!("SSR never reads a file")
    }
    fn inner(&self) -> &dyn std::any::Any {
        self
    }
}

/// `FakeFile`s as a `FileField` value.
pub fn fake_files(names: &[&'static str]) -> libero::components::Files {
    names
        .iter()
        .map(|name| dioxus::html::FileData::new(FakeFile(name)))
        .collect()
}

/// The helper itself: SSR writes a boolean bare, and a `!contains_key` on it
/// is only worth something if the key is seen when present.
#[test]
fn attributes_of_reads_a_bare_boolean() {
    fn app() -> Element {
        rsx! { button { disabled: true, "aria-disabled": "true", "x" } }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert!(html.contains("disabled=true"), "{html}");
    assert_eq!(
        attributes.get("disabled").map(String::as_str),
        Some("true"),
        "{html}"
    );
    assert_eq!(attributes["aria-disabled"], "true", "{html}");
}
