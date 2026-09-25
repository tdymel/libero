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
];

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

/// A doc with a caller's inline atom and a caller's block, drawn by `NodeViews`.
fn nodes_doc() -> Doc {
    let mut registry = NodeRegistry::default();
    registry
        .register(NodeSpec::inline("mention").attr("user", "ada"))
        .unwrap();
    registry
        .register(NodeSpec::block("callout", CustomContent::Inline))
        .unwrap();
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
            value: doc(),
            onchange: move |next| doc.set(next),
        }
        pre { id: "out", {doc.read().plain_text()} }
    }
}
