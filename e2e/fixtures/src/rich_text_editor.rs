//! `RichTextEditor`, controlled through a parent that echoes each change a task later;
//! `#out` shows the doc as Markdown, `#changes` counts `onchange` calls.

use dioxus::prelude::*;
use libero::components::RichTextEditor;
use libero::components::rich_text::{
    Attrs, Builtin, CustomContent, Doc, Inline, MarkKind, Marks, NodeRegistry, NodeSpec,
    NodeViewProps, NodeViews, use_rich_text_editor,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/rich-text-editor", || rsx! { RichTextEditorPage {} }),
    ("/rich-text-editor/handle", || rsx! { HandlePage {} }),
    ("/rich-text-editor/nodes", || rsx! { NodesPage {} }),
    ("/rich-text-editor/code", || rsx! { CodePage {} }),
    ("/rich-text-editor/sample", || rsx! { SamplePage {} }),
    ("/rich-text-editor/builtins", || rsx! { BuiltinsPage {} }),
    ("/rich-text-editor/trailing", || rsx! { TrailingPage {} }),
];

/// A doc ending in a code block, for leaving it and the language menu.
#[component]
fn TrailingPage() -> Element {
    let mut doc = use_signal(|| Doc::from_markdown("intro\n\n```rust\nlet x = 1;\n```\n"));

    rsx! {
        RichTextEditor {
            label: "Snippet",
            value: doc(),
            onchange: move |next| doc.set(next),
        }
        pre { id: "out", {doc.read().to_markdown()} }
    }
}

const SAMPLE: &str = "# Release notes\n\nSome **bold**, *italic* and `code` with a [link](https://example.com).\n\n- First item\n- Second item\n- Third item\n\n1. One\n2. Two\n\n> A quote\n\n```\nlet x = 1;\n```\n\nThe end.\n";

/// Every block kind, for the layout checks.
#[component]
fn SamplePage() -> Element {
    let mut doc = use_signal(|| Doc::from_markdown(SAMPLE));

    rsx! {
        RichTextEditor {
            label: "Notes",
            value: doc(),
            onchange: move |next| doc.set(next),
        }
    }
}

/// A code block between two paragraphs, for the arrow keys.
#[component]
fn CodePage() -> Element {
    let mut doc = use_signal(|| Doc::from_markdown("above\n\n```\nlet x = 1;\n```\n\nbelow\n"));

    rsx! {
        RichTextEditor {
            label: "Snippet",
            value: doc(),
            onchange: move |next| doc.set(next),
        }
        pre { id: "out", {doc.read().to_markdown()} }
    }
}

#[component]
fn RichTextEditorPage() -> Element {
    let mut doc = use_signal(Doc::new);
    let mut changes = use_signal(|| 0u32);

    rsx! {
        RichTextEditor {
            id: "editor-field",
            label: "Notes",
            placeholder: "Write something",
            value: doc(),
            onchange: move |next: Doc| {
                changes += 1;
                // A late echo, as an async parent would send it.
                spawn(async move { doc.set(next) });
            },
        }
        pre { id: "out", {doc.read().to_markdown()} }
        output { id: "changes", "{changes}" }
        button { id: "replace", onclick: move |_| doc.set(Doc::new()), "Replace" }
    }
}

/// A caller's own toolbar over the editor's `handle`.
#[component]
fn HandlePage() -> Element {
    let mut doc = use_signal(Doc::new);
    let editor = use_rich_text_editor();

    rsx! {
        button {
            id: "ext-bold",
            "aria-pressed": editor.is_active(MarkKind::Bold),
            onmousedown: |event| event.prevent_default(),
            onclick: move |_| { editor.run(Builtin::Bold); },
            "Bold"
        }
        button {
            id: "ext-undo",
            disabled: !editor.can_undo(),
            onclick: move |_| { editor.run(Builtin::Undo); },
            "Undo"
        }
        RichTextEditor {
            label: "Comment",
            toolbar: false,
            handle: editor,
            value: doc(),
            onchange: move |next| doc.set(next),
        }
        pre { id: "out", {doc.read().to_markdown()} }
    }
}

#[component]
fn Mention(props: NodeViewProps) -> Element {
    let user = props
        .attrs
        .get("user")
        .and_then(|user| user.as_str())
        .unwrap_or("?");
    rsx! { b { class: "mention", "@{user}" } }
}

#[component]
fn Callout(props: NodeViewProps) -> Element {
    rsx! {
        aside { class: "callout",
            span { contenteditable: "false", "Note: " }
            {props.children}
        }
    }
}

fn registry() -> NodeRegistry {
    let mut registry = NodeRegistry::default();
    registry
        .register(NodeSpec::inline("mention").attr("user", "ada"))
        .unwrap();
    registry
        .register(NodeSpec::block("callout", CustomContent::Inline))
        .unwrap();
    registry
}

/// A doc with a caller's inline atom and a caller's block, drawn by `NodeViews`.
fn nodes_doc() -> Doc {
    let registry = registry();
    let mut doc = Doc::new();
    let mention = registry.new_inline("mention", Attrs::new()).unwrap();
    doc.blocks[0]
        .inlines_mut()
        .extend([Inline::marked("hi ", Marks::new()), mention]);
    let mut callout = registry
        .new_block(&mut doc, "callout", Attrs::new())
        .unwrap();
    callout
        .inlines_mut()
        .push(Inline::marked("careful", Marks::new()));
    doc.blocks.push(callout);
    doc
}

#[component]
fn NodesPage() -> Element {
    let mut doc = use_signal(nodes_doc);

    rsx! {
        RichTextEditor {
            label: "Message",
            nodes: NodeViews::new().with("mention", Mention).with("callout", Callout),
            registry: registry(),
            value: doc(),
            onchange: move |next| doc.set(next),
        }
        pre { id: "out", {doc.read().plain_text()} }
    }
}

#[component]
fn Heading(props: NodeViewProps) -> Element {
    let level = props
        .attrs
        .get("level")
        .and_then(|level| level.as_u64())
        .unwrap_or(0);
    rsx! {
        header { class: "fancy-heading", "data-level": "{level}", {props.children} }
    }
}

#[component]
fn Quote(props: NodeViewProps) -> Element {
    rsx! { blockquote { class: "fancy-quote", {props.children} } }
}

#[component]
fn Rule(props: NodeViewProps) -> Element {
    rsx! { span { class: "fancy-rule", "data-name": props.name, "Section break" } }
}

/// Built-in blocks drawn by the caller's `NodeViews`.
#[component]
fn BuiltinsPage() -> Element {
    let mut doc = use_signal(|| Doc::from_markdown("## Title\n\n> quoted\n\n---\n\nend\n"));

    rsx! {
        RichTextEditor {
            label: "Notes",
            nodes: NodeViews::new()
                .with("heading", Heading)
                .with("quote", Quote)
                .with("rule", Rule),
            value: doc(),
            onchange: move |next| doc.set(next),
        }
        pre { id: "out", {doc.read().to_markdown()} }
    }
}
