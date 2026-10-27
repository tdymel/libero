//! `Doc` to DOM. Each leaf carries `data-key`; the editing surface (`surface`) finds
//! carets by it. Shared by the editor and its read-only fallback.

use dioxus::prelude::*;

use super::model::{Attrs, Block, BlockKind, Content, Inline, Mark, Marks, NodeKey};
use super::node_view::NodeViews;
use crate::{
    components::typography::{CodeBlock, inline_side_spacers},
    sx::{Sx, sx},
    theme::{ANCHOR_COLOR, CODE_FONT_FAMILY, ColorCss, ColorShade},
};

/// What the text of a document looks like, in the editor and in the read-only view.
pub(crate) fn content_sx(base: Sx) -> Sx {
    base.white_space("pre-wrap")
        .overflow_wrap("anywhere")
        .selector(
            "& > :first-child, & > * > :first-child",
            sx().margin_top("0"),
        )
        .selector(
            "& > :last-child, & > * > :last-child",
            sx().margin_bottom("0"),
        )
        .selector("& pre", sx().font_family("monospace").margin("0"))
        // A shown code block scrolls inside itself: its longest line must not widen the editor (1466).
        .selector("& [data-code='view']", sx().with("contain", "inline-size"))
        // List items hold paragraphs: no paragraph gaps between bullets.
        .selector("& li > p", sx().margin("0"))
        // A task item's box in the marker gutter: absolute, since the item's first child is a block.
        .selector(
            "& li[data-checked]",
            sx().list_style("none").position("relative"),
        )
        .selector(
            "& [data-task]",
            sx().position("absolute")
                .left("-1.5em")
                .top("0.2em")
                .width("1em")
                .height("1em")
                .box_sizing("border-box")
                .border(format!(
                    "1px solid {}",
                    ColorCss::MUTED.value(ColorShade::S6)
                ))
                .border_radius("3px")
                .font_size("1em")
                .line_height("1em")
                .text_align("center")
                .user_select("none"),
        )
        .selector(
            "& [data-task][aria-checked='true']::before",
            // Empty alt text: the mark stays out of the name the task's text gives the box.
            sx().content("'✓' / ''"),
        )
        .selector(
            "& [data-task][aria-checked='true']",
            sx().background("primary")
                .border_color("primary")
                .color("primary-contrast"),
        )
        // A 24px press area around the 1em box (WCAG 2.5.8).
        .selector("& [data-task][data-clickable]", sx().cursor("pointer"))
        .selector(
            "& [data-task][data-clickable]::after",
            sx().content("''")
                .position("absolute")
                .inset("calc(50% - 12px)"),
        )
        // Inline code, links and quotes as `Code`, `Anchor` and `Blockquote` draw them.
        .selector(
            "& :not(pre) > code",
            sx().background("muted.2")
                .border_radius("4px")
                .padding("0 0.25em")
                .and(inline_side_spacers("0.25em", "4px"))
                .font_family(CODE_FONT_FAMILY.value())
                .font_size("0.875em"),
        )
        .selector("& a", sx().color(ANCHOR_COLOR.value()))
        .selector(
            "& blockquote",
            sx().margin("1em 0")
                .padding("0 0.75rem")
                .border_left(format!(
                    "2px solid {}",
                    ColorCss::MUTED.value(ColorShade::S4)
                )),
        )
}

#[derive(Clone, Copy)]
pub(crate) struct RenderCtx<'a> {
    /// The code block edited as source, fences shown; others render as `CodeBlock`.
    pub source_code: Option<NodeKey>,
    /// Pressing a rendered code block moves the caret into it.
    pub on_code: Option<EventHandler<NodeKey>>,
    /// Clicking a task item's box toggles it; `None` shows the box read-only.
    pub on_task: Option<EventHandler<NodeKey>>,
    /// Names a task box whose line is empty; others take their line's text.
    pub task_label: &'a str,
    /// Starts the ids the task boxes point their names at; unique per editor or view.
    pub id_prefix: &'a str,
    /// The caller's components for custom nodes.
    pub views: &'a NodeViews,
    /// The language button on the source block's opening fence.
    pub fence: Option<&'a Element>,
    /// Added to every heading's level, clamped at `h6`.
    pub heading_offset: u8,
    /// Set by a key pressed in a focusable island (a rendered code block, an atom view),
    /// so the surface's keymap leaves it to that island (todo 4488).
    pub island_key: Option<CopyValue<bool>>,
}

impl RenderCtx<'_> {
    fn heading_level(&self, level: u8) -> u8 {
        level.saturating_add(self.heading_offset).min(6)
    }

    fn on_island_key(&self) -> impl FnMut(KeyboardEvent) + 'static {
        let island_key = self.island_key;
        move |_| {
            if let Some(mut island_key) = island_key {
                island_key.set(true);
            }
        }
    }
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
        BlockKind::Paragraph => paragraph(block, None, None, ctx),
        BlockKind::Heading { level } => heading(
            ctx.heading_level(*level),
            key,
            inlines(block.inlines(), ctx),
        ),
        BlockKind::CodeBlock { language } => code(block, language, ctx),
        BlockKind::Quote => rsx! { blockquote { {blocks(block.children(), ctx)} } },
        BlockKind::List {
            ordered: true,
            start,
        } => rsx! {
            ol { start: (*start != 1).then(|| start.to_string()), {blocks(block.children(), ctx)} }
        },
        BlockKind::List { .. } => rsx! { ul { {blocks(block.children(), ctx)} } },
        BlockKind::ListItem { checked } => rsx! {
            li { "data-checked": checked.map(|checked| checked.to_string()),
                {item_content(block, *checked, ctx)}
            }
        },
        BlockKind::Rule => rsx! { hr { "data-key": key, contenteditable: "false" } },
        BlockKind::Custom { name, attrs, .. } => custom(name, attrs, key, &block.content, ctx),
    }
}

/// A list item's task box, if any, and its blocks.
fn item_content(block: &Block, checked: Option<bool>, ctx: RenderCtx<'_>) -> Element {
    let children = block.children();
    // Inside the first line's leaf, so a caret beside the box is still in the text.
    let lead = children.first().filter(|first| {
        checked.is_some() && first.kind == BlockKind::Paragraph && !ctx.views.contains("paragraph")
    });
    // The box is named by its task's text; an empty line keeps the generic label.
    let text_id = lead
        .filter(|first| !first.is_empty())
        .map(|first| format!("{}-task-{}", ctx.id_prefix, first.key.0));
    let task = checked.map(|checked| task_box(checked, block.key, text_id.clone(), ctx));
    let rest = if lead.is_some() {
        &children[1..]
    } else {
        children
    };
    rsx! {
        match lead {
            Some(first) => paragraph(first, text_id, task, ctx),
            None => rsx! { {task} },
        }
        {blocks(rest, ctx)}
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
            div {
                "data-key": key,
                "data-node": "{name}",
                contenteditable: "false",
                onkeydown: ctx.on_island_key(),
                {view}
            }
        },
        // Typed text targets the surface above, so a key seen here is from a focused control.
        Some(view) => rsx! {
            div { "data-node": "{name}", onkeydown: ctx.on_island_key(), {view} }
        },
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
        BlockKind::ListItem { .. } => "list_item",
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
            let level = ctx.heading_level(*level);
            attrs.insert("level".into(), level.into());
            heading(level, key.clone(), inlines(block.inlines(), ctx))
        }
        BlockKind::Rule => VNode::empty(),
        BlockKind::ListItem { checked } => {
            if let Some(checked) = checked {
                attrs.insert("checked".into(), (*checked).into());
            }
            item_content(block, *checked, ctx)
        }
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
            div {
                "data-key": key,
                "data-node": "{name}",
                contenteditable: "false",
                onkeydown: ctx.on_island_key(),
                {view}
            }
        },
        _ => view.unwrap_or_else(VNode::empty),
    }
}

/// `lead` goes before the text, skipped by the caret's offsets.
fn paragraph(
    block: &Block,
    id: Option<String>,
    lead: Option<Element>,
    ctx: RenderCtx<'_>,
) -> Element {
    rsx! {
        p { id, "data-key": block.key.0.to_string(), {lead} {inlines(block.inlines(), ctx)} }
    }
}

/// A task item's box: an island the caret's offsets and the text skip (`data-skip`).
/// Not a tab stop; the keyboard toggles with the `toggle_task` chord (CKEditor 5 to-do lists).
fn task_box(
    checked: bool,
    item: NodeKey,
    labelled_by: Option<String>,
    ctx: RenderCtx<'_>,
) -> Element {
    let on_task = ctx.on_task;
    let label = labelled_by.is_none().then(|| ctx.task_label.to_string());
    rsx! {
        span {
            "data-task": "",
            "data-skip": "",
            "data-clickable": on_task.is_some().then_some(""),
            contenteditable: "false",
            role: "checkbox",
            aria_checked: checked.to_string(),
            aria_readonly: on_task.is_none().then_some("true"),
            aria_labelledby: labelled_by,
            aria_label: label,
            // The caret stays where it was.
            onmousedown: move |event: MouseEvent| event.prevent_default(),
            onclick: move |event: MouseEvent| {
                if let Some(on_task) = on_task {
                    event.prevent_default();
                    on_task.call(item);
                }
            },
        }
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
            // Code is LTR: bidi would reorder its operators on an RTL page (todo 735).
            div { "data-code": "source", dir: "ltr",
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
            // Its scroll region is a tab stop once it overflows.
            onkeydown: ctx.on_island_key(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::form::rich_text_editor::{model::Doc, node_view::NodeViewProps};

    fn app() -> Element {
        let doc = Doc::from_markdown("- [x] Buy milk\n- [ ] ");
        let views = NodeViews::default();
        blocks(
            &doc.blocks,
            RenderCtx {
                source_code: None,
                on_code: None,
                on_task: None,
                task_label: "Task done",
                id_prefix: "ed",
                views: &views,
                fence: None,
                heading_offset: 0,
                island_key: None,
            },
        )
    }

    #[test]
    fn a_task_box_is_named_by_its_text_and_an_empty_one_by_the_label() {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let id = html
            .split("aria-labelledby=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("a labelled box");
        assert!(html.contains(&format!("<p id=\"{id}\"")), "{html}");
        assert!(html.contains("Buy milk"), "{html}");
        assert_eq!(html.matches("aria-labelledby=").count(), 1, "{html}");
        assert_eq!(
            html.matches("aria-label=\"Task done\"").count(),
            1,
            "{html}"
        );
    }

    fn task_item(props: NodeViewProps) -> Element {
        let checked = props
            .attrs
            .get("checked")
            .and_then(|checked| checked.as_bool());
        rsx! {
            li { "data-checked": checked.map(|checked| checked.to_string()), {props.children} }
        }
    }

    fn with_view() -> Element {
        let doc = Doc::from_markdown("- [x] Buy milk\n- plain");
        let views = NodeViews::new().with("list_item", task_item);
        blocks(
            &doc.blocks,
            RenderCtx {
                source_code: None,
                on_code: None,
                on_task: None,
                task_label: "Task done",
                id_prefix: "ed",
                views: &views,
                fence: None,
                heading_offset: 0,
                island_key: None,
            },
        )
    }

    /// Todo 3794: a caller's `list_item` view still gets the task box and its state.
    #[test]
    fn a_list_item_view_keeps_the_task_box() {
        let mut dom = VirtualDom::new(with_view);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert_eq!(html.matches("role=\"checkbox\"").count(), 1, "{html}");
        assert!(html.contains("aria-checked=\"true\""), "{html}");
        assert_eq!(html.matches("data-checked=\"true\"").count(), 1, "{html}");
        assert_eq!(html.matches("data-checked").count(), 1, "{html}");
    }
}
