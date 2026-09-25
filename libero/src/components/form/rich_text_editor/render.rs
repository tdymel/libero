//! `Doc` to DOM. Each leaf carries `data-key`; the editing surface (`surface`) finds
//! carets by it. Shared by the editor and its read-only fallback.

use dioxus::prelude::*;

use super::model::{Block, BlockKind, Content, Inline, Mark, Marks, NodeKey};
use crate::components::typography::CodeBlock;

#[derive(Clone, Copy)]
pub(crate) struct RenderCtx {
    /// The code block edited as source, fences shown; others render as `CodeBlock`.
    pub source_code: Option<NodeKey>,
    /// Pressing a rendered code block moves the caret into it.
    pub on_code: Option<EventHandler<NodeKey>>,
}

pub(crate) fn blocks(blocks: &[Block], ctx: RenderCtx) -> Element {
    rsx! {
        for block in blocks {
            {self::block(block, ctx)}
        }
    }
}

fn block(block: &Block, ctx: RenderCtx) -> Element {
    let key = block.key.0.to_string();
    match &block.kind {
        BlockKind::Paragraph => rsx! { p { "data-key": key, {inlines(block.inlines())} } },
        BlockKind::Heading { level } => heading(*level, key, inlines(block.inlines())),
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
        BlockKind::Custom { name, .. } => match &block.content {
            Content::Inlines(content) => rsx! {
                div { "data-key": key, "data-node": "{name}", {inlines(content)} }
            },
            Content::Blocks(children) => rsx! {
                div { "data-node": "{name}", {blocks(children, ctx)} }
            },
            Content::Empty => rsx! {
                div { "data-key": key, "data-node": "{name}", contenteditable: "false", "{name}" }
            },
        },
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
fn code(block: &Block, language: &str, ctx: RenderCtx) -> Element {
    let key = block.key;
    let text = block.text();
    if ctx.source_code == Some(key) {
        let filler = text.is_empty() || text.ends_with('\n');
        return rsx! {
            div { "data-code": "source",
                div { contenteditable: "false", "data-fence": "", "```{language}" }
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
            contenteditable: "false",
            onmousedown: move |event: MouseEvent| {
                if let Some(on_code) = on_code {
                    event.prevent_default();
                    on_code.call(key);
                }
            },
            CodeBlock { language: language.to_string(), source: text, copyable: false }
        }
    }
}

fn inlines(content: &[Inline]) -> Element {
    // A trailing break or an empty leaf needs a line box for the caret.
    let filler = matches!(content.last(), None | Some(Inline::HardBreak));
    rsx! {
        for inline in content {
            {self::inline(inline)}
        }
        if filler {
            br { "data-skip": "" }
        }
    }
}

fn inline(inline: &Inline) -> Element {
    match inline {
        Inline::Text { text, marks } => marked(text, marks),
        Inline::HardBreak => rsx! { br {} },
        Inline::Node { name, attrs } => {
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
