//! `RichTextView` natively, and the `RichTextEditor` that shows its document through it:
//! Blitz has no `contenteditable`, so the view is the read-only path.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{
    RichTextEditor,
    rich_text::{BlockKind, Doc, Inline, Mark, Marks, RichTextView},
};

const MARKDOWN: &str = "## Release notes

Fixed in **v2**, *see* `main.rs` and [the guide](https://libero-ui.dev).

- one
- two

```rust
fn main() {}
```";

/// Spaces at the edge of a marked run, which Blitz trims unless the text preserves them.
fn spaced() -> Doc {
    let mut doc = Doc::new();
    let bold = Marks::new().with(Mark::Bold);
    let code = Marks::new().with(Mark::Code);
    let inlines = vec![
        Inline::text("a "),
        Inline::marked("bold ", bold),
        Inline::text("then "),
        Inline::marked(" code ", code),
        Inline::text("end"),
    ];
    doc.blocks = vec![doc.leaf(BlockKind::Paragraph, inlines)];
    doc
}

fn view() -> Element {
    rsx! { RichTextView { id: "doc", value: Doc::from_markdown(MARKDOWN) } }
}

fn spaces() -> Element {
    rsx! { RichTextView { id: "doc", value: spaced() } }
}

fn editor() -> Element {
    rsx! { RichTextEditor { label: "Notes", id: "editor", value: Doc::from_markdown(MARKDOWN) } }
}

#[test]
fn a_view_lays_out_its_marks_in_running_text() {
    let page = mount(view);
    assert_eq!(
        page.laid_out_text("#doc p"),
        "Fixed in v2, see main.rs and the guide."
    );
    for (selector, text) in [
        ("#doc strong", "v2"),
        ("#doc em", "see"),
        ("#doc :not(pre) > code", "main.rs"),
        ("#doc a", "the guide"),
    ] {
        assert_eq!(page.text(selector), text, "{selector}");
    }
    assert_eq!(
        page.attr("#doc a", "href").as_deref(),
        Some("https://libero-ui.dev")
    );
}

#[test]
fn a_view_keeps_the_spaces_at_the_edge_of_a_marked_run() {
    let page = mount(spaces);
    assert_eq!(page.laid_out_text("#doc p"), "a bold then  code end");
}

#[test]
fn a_view_paints_its_code_block_and_list() {
    let page = mount(view);
    assert!(
        page.text("#doc [data-code='view']")
            .contains("fn main() {}")
    );
    let (_, _, width, height) = page.rect("#doc [data-code='view']");
    assert!(width > 0.0 && height > 0.0, "{width}x{height}");
    assert_eq!(page.query_all("#doc li").len(), 2);
    // The code block sits below the list, the list below the heading.
    let (_, heading, ..) = page.rect("#doc h2");
    let (_, list, ..) = page.rect("#doc ul");
    let (_, code, ..) = page.rect("#doc [data-code='view']");
    assert!(heading < list && list < code, "{heading} {list} {code}");
}

#[test]
fn the_editor_shows_its_document_through_the_view() {
    let page = mount(editor);
    assert_eq!(
        page.attr("#editor", "contenteditable").as_deref(),
        Some("false")
    );
    assert_eq!(
        page.laid_out_text("#editor p"),
        "Fixed in v2, see main.rs and the guide."
    );
    assert!(page.exists("#editor li"));
}
