//! [`RichTextView`]: a stored `Doc` shown without an editor's state, script or history.

use dioxus::prelude::*;

use super::model::Doc;
use super::node_view::NodeViews;
use super::render::{RenderCtx, blocks, content_sx};
use crate::{
    components::{
        common::{HtmlTag, Input, base_props},
        layout::use_box,
    },
    sx::{StaticSx, sx},
};

static VIEW_SX: StaticSx = StaticSx::new(|| content_sx(sx().display("block")));

base_props! {
    extends(div);
    pub struct RichTextViewProps {
        /// The document to show.
        value: Doc,
        /// Components by node name, built-ins too (not `code_block`), as
        /// [`RichTextEditor`](super::RichTextEditor)'s `nodes`; a custom node without one
        /// draws a plain fallback.
        #[props(default)]
        nodes: NodeViews,
    }
}

/// A [`Doc`] as read-only content: paragraphs, headings, lists, quotes, code blocks, marks
/// and links, drawn as the editor draws them. For a message or comment list, and the page
/// that shows a stored document on a renderer without an editable surface.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::rich_text::{Doc, RichTextView};
/// # fn app() -> Element {
/// let comment = Doc::from_markdown("Fixed in **v2**, see `main.rs`.");
/// rsx! { RichTextView { value: comment } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/rich-text-editor>
#[component]
pub fn RichTextView(props: RichTextViewProps) -> Element {
    use_box()
        .framework_sx(&VIEW_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(
            HtmlTag::Div,
            props.attributes,
            blocks(
                &props.value.blocks,
                RenderCtx {
                    source_code: None,
                    on_code: None,
                    views: &props.nodes,
                    fence: None,
                },
            ),
        )
}
