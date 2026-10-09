use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::rich_text::{
        Attrs, BlockKind, Doc, Inline, NodeViewProps, NodeViews, RichTextView,
    },
};

const MARKDOWN: &str = "## Release notes

Fixed in **v2**, *see* `main.rs` and [the guide](https://libero-ui.dev).

- one
- two

> quoted

```rust
fn main() {}
```";

fn app() -> Element {
    rsx! {
        LiberoProvider {
            RichTextView { value: Doc::from_markdown(MARKDOWN) }
        }
    }
}

#[test]
fn a_view_renders_blocks_marks_and_links() {
    let html = render(app);
    let html = body(&html);

    assert!(html.contains("<h2"), "{html}");
    assert!(html.contains("Release notes"), "{html}");
    assert!(html.contains("<strong>v2</strong>"), "{html}");
    assert!(html.contains("<em>see</em>"), "{html}");
    assert!(html.contains("<code>main.rs</code>"), "{html}");
    assert!(
        html.contains(r#"<a href="https://libero-ui.dev""#),
        "{html}"
    );
    assert!(html.contains("<ul>") && html.contains("<li>"), "{html}");
    assert!(html.contains("<blockquote>"), "{html}");
    assert!(html.contains(r#"data-code="view""#), "{html}");
}

#[test]
fn a_view_is_not_an_editing_surface() {
    let html = render(app);
    let html = body(&html);

    for editing in ["contenteditable=\"true\"", "role=\"textbox\"", "tabindex"] {
        assert!(!html.contains(editing), "{editing} in {html}");
    }
}

#[test]
fn a_view_draws_a_caller_node_through_its_view() {
    fn mention(props: NodeViewProps) -> Element {
        let user = props.attrs.get("user").and_then(|user| user.as_str());
        rsx! { b { class: "mention", "@{user.unwrap_or_default()}" } }
    }
    fn app() -> Element {
        let mut doc = Doc::new();
        let node = Inline::Node {
            name: "mention".into(),
            attrs: Attrs::from([("user".into(), "ada".into())]),
        };
        doc.blocks = vec![doc.leaf(BlockKind::Paragraph, vec![Inline::text("Hi "), node])];
        rsx! {
            LiberoProvider {
                RichTextView { value: doc, nodes: NodeViews::new().with("mention", mention) }
            }
        }
    }

    let html = render(app);
    assert!(
        body(&html).contains(r#"<b class="mention">@ada</b>"#),
        "{html}"
    );
}
