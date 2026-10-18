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
    hooks::{use_id, use_localization},
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
        /// Added to every heading's level, clamped at `h6`, so a comment's `#` sits below
        /// the host page's own headings: `1` draws `h1` as `h2`.
        #[props(default)]
        heading_offset: u8,
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
    let words = use_localization().rich_text_editor;
    let id = use_id();
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
                    on_task: None,
                    task_label: words.toggle_task,
                    id_prefix: &id.peek(),
                    views: &props.nodes,
                    fence: None,
                    heading_offset: props.heading_offset,
                },
            ),
        )
}
