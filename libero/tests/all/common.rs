//! Renders a component to HTML and picks it apart, so a test can assert on
//! one element's attributes instead of matching against the whole document.

use std::collections::BTreeMap;
use std::thread;
use std::time::{Duration, Instant};

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

/// The rendered markup with the `<style>` and `<script>` blocks stripped -
/// the CSS is bigger than the markup and full of words like `first-child`,
/// and the provider's colour-scheme script carries `light`/`dark`, so
/// anything searching for text has to look past both. They are removed
/// wherever they sit, since `StyleOutlet` renders the registered sheets
/// *after* the markup while the static ones still come before it.
pub fn body(html: &str) -> String {
    strip(&strip(html, "style"), "script")
}

fn strip(html: &str, tag: &str) -> String {
    let (open, close) = (format!("<{tag}"), format!("</{tag}>"));
    let mut body = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find(&open) {
        let Some(end) = rest[start..].find(&close) else {
            break;
        };
        body.push_str(&rest[..start]);
        rest = &rest[start + end + close.len()..];
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
    nth_attributes(html, tag, 0)
}

/// Every attribute of the `n`th `<tag>` in `html`, counted from 0. `<li`
/// does not count a `<link>`.
pub fn nth_attributes(html: &str, tag: &str, n: usize) -> BTreeMap<String, String> {
    let open = format!("<{tag}");
    let start = html
        .match_indices(&open)
        .map(|(at, _)| at)
        .filter(|at| ends_name(&html[at + open.len()..]))
        .nth(n)
        .unwrap_or_else(|| panic!("no <{tag}> number {n} in the rendered output:\n{html}"));
    parse_tag(&html[start..])
}

/// The attributes of every tag that carries `marker` - an attribute as SSR
/// writes it, such as `role="slider"`, or a tag opening, `<input` - in
/// document order. A marker in text is not a tag and is passed over.
pub fn tags_with(html: &str, marker: &str) -> Vec<BTreeMap<String, String>> {
    html.match_indices(marker)
        .filter_map(|(at, _)| {
            let start = match marker.starts_with('<') {
                true if !ends_name(&html[at + marker.len()..]) => return None,
                true => at,
                false => html[..at].rfind('<')?,
            };
            (!html[start..at].contains('>')).then(|| parse_tag(&html[start..]))
        })
        .collect()
}

/// The attributes of the first tag carrying `marker`, as [`tags_with`] reads it.
pub fn tag_with(html: &str, marker: &str) -> BTreeMap<String, String> {
    tags_with(html, marker)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no tag with {marker} in the rendered output:\n{html}"))
}

/// Whether a tag name ends where `rest` starts: `<li` before `<link` does not.
fn ends_name(rest: &str) -> bool {
    rest.starts_with(|c: char| c.is_whitespace() || matches!(c, '>' | '/'))
}

/// The attributes of the tag `html` starts with.
fn parse_tag(html: &str) -> BTreeMap<String, String> {
    let name_end = html
        .find(|c: char| c.is_whitespace() || matches!(c, '>' | '/'))
        .unwrap_or(html.len());
    let tag = &html[1..name_end];
    let mut rest = &html[name_end..];
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

/// Every CSS rule whose selector names one of `element`'s classes, written
/// `selector{declarations}`, so a declaration is asserted on that element only.
pub fn rules_for(html: &str, element: &BTreeMap<String, String>) -> String {
    let classes = element.get("class").map_or("", String::as_str);
    let mut rules = String::new();
    for class in classes.split_whitespace() {
        let name = format!(".{class}");
        for (at, _) in html.match_indices(&name) {
            let after = &html[at + name.len()..];
            // `.lsx-a` is not `.lsx-ab`, and a selector runs to its `{`.
            let longer = after.starts_with(|c: char| c.is_alphanumeric() || matches!(c, '-' | '_'));
            let Some(open) = after.find('{') else {
                continue;
            };
            if longer || after[..open].contains(['}', ';', '<']) {
                continue;
            }
            let start = html[..at].rfind(['{', '}', '>']).map_or(0, |i| i + 1);
            let end = at + name.len() + open + after[open..].find('}').unwrap() + 1;
            rules.push_str(&html[start..end]);
        }
    }
    rules
}

/// Polls `dom` until `done` holds for its markup or `limit` runs out, for a
/// test on a real timer. `process_events` drains the task a timer delivers
/// through, and `render_immediate` applies what it wrote.
pub fn drive_until(dom: &mut VirtualDom, limit: Duration, done: impl Fn(&str) -> bool) -> String {
    let start = Instant::now();
    loop {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = body(&dioxus_ssr::render(dom));
        if done(&html) || start.elapsed() > limit {
            return html;
        }
        thread::sleep(Duration::from_millis(2));
    }
}

/// One pass of [`drive_until`]: what is pending now is applied, and nothing waits.
pub fn drive_once(dom: &mut VirtualDom) -> String {
    drive_until(dom, Duration::ZERO, |_| true)
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

#[test]
fn the_pickers_count_whole_tags_and_skip_text() {
    let html =
        r#"<link rel="x"><li id="a">role="slider"</li><li id="b" role="slider"><p role="slider"/>"#;

    assert_eq!(nth_attributes(html, "li", 0)["id"], "a");
    assert_eq!(nth_attributes(html, "li", 1)["id"], "b");
    let sliders = tags_with(html, r#"role="slider""#);
    assert_eq!(sliders.len(), 2, "{sliders:?}");
    assert_eq!(sliders[0]["id"], "b");
    assert!(!sliders[1].contains_key("id"), "{sliders:?}");
    assert_eq!(tags_with(html, "<li").len(), 2);
}

#[test]
fn rules_for_reads_only_the_rules_naming_the_elements_class() {
    let html = r#"<style>.a{color:red;}.ab{width:0;}@media (x){.a[data-state~="on"]{gap:0;}}.b{top:0;}</style><p class="a">"#;

    let rules = rules_for(html, &attributes_of(html, "p"));
    assert_eq!(rules, r#".a{color:red;}.a[data-state~="on"]{gap:0;}"#);
}
