//! `Doc` to DOM. Each leaf carries `data-key`; the editing surface (`surface`) finds
//! carets by it. Shared by the editor and its read-only fallback.

use dioxus::prelude::*;

use super::model::{Attrs, Block, BlockKind, Content, Inline, Mark, Marks, NodeKey};
use super::node_view::NodeViews;
use crate::components::typography::CodeBlock;

#[derive(Clone, Copy)]
pub(crate) struct RenderCtx<'a> {
    /// The code block edited as source, fences shown; others render as `CodeBlock`.
    pub source_code: Option<NodeKey>,
    /// Pressing a rendered code block moves the caret into it.
    pub on_code: Option<EventHandler<NodeKey>>,
    /// The caller's components for custom nodes.
    pub views: &'a NodeViews,
    /// The language button on the source block's opening fence.
    pub fence: Option<&'a Element>,
}

pub(crate) fn blocks(blocks: &[Block], ctx: RenderCtx<'_>) -> Element {
    rsx! {
        for block in blocks {
            {self::block(block, ctx)}
        }
    }
}

fn block(block: &Block, ctx: RenderCtx<'_>) -> Element {
    let key = block.key.0.to_string();
    if let Some(name) = builtin_name(&block.kind)
        && ctx.views.contains(name)
    {
        return builtin_view(name, block, key, ctx);
    }
    match &block.kind {
        BlockKind::Paragraph => rsx! { p { "data-key": key, {inlines(block.inlines(), ctx)} } },
        BlockKind::Heading { level } => heading(*level, key, inlines(block.inlines(), ctx)),
        BlockKind::CodeBlock { language } => code(block, language, ctx),
        BlockKind::Quote => rsx! { blockquote { {blocks(block.children(), ctx)} } },
        BlockKind::List {
            ordered: true,
            start,
        } => rsx! {
            ol { start: (*start != 1).then(|| start.to_string()), {blocks(block.children(), ctx)} }
        },
        BlockKind::List { .. } => rsx! { ul { {blocks(block.children(), ctx)} } },
        BlockKind::ListItem => rsx! { li { {blocks(block.children(), ctx)} } },
        BlockKind::Rule => rsx! { hr { "data-key": key, contenteditable: "false" } },
        BlockKind::Custom { name, attrs, .. } => custom(name, attrs, key, &block.content, ctx),
    }
}

/// A caller's block through its view, if any; its content keeps the leaf the caret needs.
fn custom(
    name: &str,
    attrs: &Attrs,
    key: String,
    content: &Content,
    ctx: RenderCtx<'_>,
) -> Element {
    let children = match content {
        Content::Inlines(content) => {
            rsx! { div { "data-key": key.clone(), {inlines(content, ctx)} } }
        }
        Content::Blocks(children) => blocks(children, ctx),
        Content::Empty => VNode::empty(),
    };
    let atom = matches!(content, Content::Empty);
    match ctx.views.render(name, attrs, children.clone()) {
        Some(view) if atom => rsx! {
            div { "data-key": key, "data-node": "{name}", contenteditable: "false", {view} }
        },
        Some(view) => rsx! { div { "data-node": "{name}", {view} } },
        None if atom => rsx! {
            div { "data-key": key, "data-node": "{name}", contenteditable: "false", "{name}" }
        },
        None => rsx! { div { "data-node": "{name}", {children} } },
    }
}

/// The registered name of a built-in a caller's `NodeViews` may draw; code blocks stay fixed.
fn builtin_name(kind: &BlockKind) -> Option<&'static str> {
    Some(match kind {
        BlockKind::Paragraph => "paragraph",
        BlockKind::Heading { .. } => "heading",
        BlockKind::Quote => "quote",
        BlockKind::List { .. } => "list",
        BlockKind::ListItem => "list_item",
        BlockKind::Rule => "rule",
        BlockKind::CodeBlock { .. } | BlockKind::Custom { .. } => return None,
    })
}

/// A built-in through the caller's view: a leaf's text keeps its own tag and `data-key`,
/// a container passes its rendered blocks, a rule is an island.
fn builtin_view(name: &str, block: &Block, key: String, ctx: RenderCtx<'_>) -> Element {
    let mut attrs = Attrs::new();
    let children = match &block.kind {
        BlockKind::Paragraph => {
            rsx! { p { "data-key": key.clone(), {inlines(block.inlines(), ctx)} } }
        }
        BlockKind::Heading { level } => {
            attrs.insert("level".into(), (*level).into());
            heading(*level, key.clone(), inlines(block.inlines(), ctx))
        }
        BlockKind::Rule => VNode::empty(),
        kind => {
            if let BlockKind::List { ordered, start } = kind {
                attrs.insert("ordered".into(), (*ordered).into());
                attrs.insert("start".into(), (*start).into());
            }
            blocks(block.children(), ctx)
        }
    };
    let view = ctx.views.render(name, &attrs, children);
    match block.kind {
        BlockKind::Rule => rsx! {
            div { "data-key": key, "data-node": "{name}", contenteditable: "false", {view} }
        },
        _ => view.unwrap_or_else(VNode::empty),
    }
}

fn heading(level: u8, key: String, children: Element) -> Element {
    match level {
        1 => rsx! { h1 { "data-key": key, {children} } },
        2 => rsx! { h2 { "data-key": key, {children} } },
        3 => rsx! { h3 { "data-key": key, {children} } },
        4 => rsx! { h4 { "data-key": key, {children} } },
        5 => rsx! { h5 { "data-key": key, {children} } },
        _ => rsx! { h6 { "data-key": key, {children} } },
    }
}

/// Source while the caret is in it, fences around it; a highlighted `CodeBlock` otherwise.
fn code(block: &Block, language: &str, ctx: RenderCtx<'_>) -> Element {
    let key = block.key;
    let text = block.text();
    if ctx.source_code == Some(key) {
        let filler = text.is_empty() || text.ends_with('\n');
        return rsx! {
            div { "data-code": "source",
                div { contenteditable: "false", "data-fence": "",
                    {match ctx.fence {
                        Some(button) => rsx! { "```" {button.clone()} },
                        None => rsx! { "```{language}" },
                    }}
                }
                pre { "data-key": key.0.to_string(),
                    "{text}"
                    if filler {
                        br { "data-skip": "" }
                    }
                }
                div { contenteditable: "false", "data-fence": "", "```" }
            }
        };
    }
    let on_code = ctx.on_code;
    rsx! {
        div {
            "data-code": "view",
            "data-code-key": key.0.to_string(),
            contenteditable: "false",
            onmousedown: move |event: MouseEvent| {
                if let Some(on_code) = on_code {
                    event.prevent_default();
                    on_code.call(key);
                }
            },
            CodeBlock {
                language: language.to_string(),
                source: text,
                header: !language.is_empty(),
                copyable: false,
            }
        }
    }
}

fn inlines(content: &[Inline], ctx: RenderCtx<'_>) -> Element {
    // A trailing break or an empty leaf needs a line box for the caret.
    let filler = matches!(content.last(), None | Some(Inline::HardBreak));
    rsx! {
        for inline in content {
            {self::inline(inline, ctx)}
        }
        if filler {
            br { "data-skip": "" }
        }
    }
}

fn inline(inline: &Inline, ctx: RenderCtx<'_>) -> Element {
    match inline {
        Inline::Text { text, marks } => marked(text, marks),
        Inline::HardBreak => rsx! { br {} },
        Inline::Node { name, attrs } => {
            if let Some(view) = ctx.views.render(name, attrs, VNode::empty()) {
                return rsx! {
                    span { "data-atom": "", "data-node": "{name}", contenteditable: "false", {view} }
                };
            }
            let label = attrs
                .get("label")
                .and_then(|label| label.as_str())
                .unwrap_or(name)
                .to_string();
            rsx! {
                span { "data-atom": "", "data-node": "{name}", contenteditable: "false", "{label}" }
            }
        }
    }
}

/// `text` inside one element per mark, a link outermost.
fn marked(text: &str, marks: &Marks) -> Element {
    let mut element = rsx! { "{text}" };
    for mark in marks.iter() {
        element = match mark {
            Mark::Bold => rsx! { strong { {element} } },
            Mark::Italic => rsx! { em { {element} } },
            Mark::Underline => rsx! { u { {element} } },
            Mark::Strike => rsx! { s { {element} } },
            Mark::Code => rsx! { code { {element} } },
            Mark::Link { href, title } => rsx! {
                a { href: href.as_str().to_string(), title: title.clone(), {element} }
            },
        };
    }
    element
}
