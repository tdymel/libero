//! `RichTextEditor`, controlled through a parent that echoes each change a task later;
//! `#out` shows the doc as Markdown, `#changes` counts `onchange` calls.

use dioxus::prelude::*;
use libero::components::RichTextEditor;
use libero::components::rich_text::{
    Attrs, Builtin, Commands, CustomContent, Doc, EditorInput, EditorState, Inline, MarkKind,
    Marks, NodeRegistry, NodeSpec, NodeViewProps, NodeViews, Position, RichTextTool, Selection,
    use_rich_text_editor,
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
    ("/rich-text-editor/mentions", || rsx! { MentionsPage {} }),
    ("/rich-text-editor/narrow", || rsx! { NarrowPage {} }),
    ("/rich-text-editor/states", || rsx! { StatesPage {} }),
];

/// A read-only and a disabled editor, for their tab stop and state.
#[component]
fn StatesPage() -> Element {
    let doc = Doc::from_markdown("Fixed text\n");
    rsx! {
        div { id: "readonly", RichTextEditor { label: "Read only", value: doc.clone(), readonly: true } }
        div { id: "disabled", RichTextEditor { label: "Disabled", value: doc, disabled: true } }
    }
}

/// Todo 1466: a long code line in a 20rem centring row, as the docs preview.
#[component]
fn NarrowPage() -> Element {
    let mut doc = use_signal(|| {
        Doc::from_markdown(
            "intro\n\n```rust\nprintln!(\"a line far longer than the twenty rem row it sits in\");\n```\n",
        )
    });

    rsx! {
        div { id: "row", display: "flex", justify_content: "center", width: "20rem",
            RichTextEditor {
                label: "Snippet",
                value: doc(),
                onchange: move |next| doc.set(next),
            }
        }
    }
}

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

const USERS: [&str; 4] = ["ada", "alan", "grace", "linus"];

/// The word after an `@` that starts a word, up to the caret.
fn mention_query(state: &EditorState) -> Option<String> {
    let before = state.text_before_caret();
    let (head, query) = before.rsplit_once('@')?;
    let starts = head.is_empty() || head.ends_with(char::is_whitespace);
    (starts && query.chars().all(char::is_alphanumeric)).then(|| query.to_string())
}

/// Replaces `@query` before the caret with a mention of `user`.
fn insert_mention(state: &mut EditorState, user: &str) -> bool {
    let Some(query) = mention_query(state) else {
        return false;
    };
    let at = state.caret();
    let from = Position::new(at.block, at.offset - query.chars().count() - 1);
    let mut attrs = Attrs::new();
    attrs.insert("user".into(), user.into());
    let Some(mention) = registry().new_inline("mention", attrs) else {
        return false;
    };
    state.select(Selection::range(from, at));
    state.insert_inline(mention) && state.insert_text(" ")
}

/// Type `@` and a name: a list under the caret, steered by the arrow keys through
/// `intercept`, and a toolbar button of the caller's that types the `@`.
#[component]
fn MentionsPage() -> Element {
    let mut doc = use_signal(Doc::new);
    let editor = use_rich_text_editor();
    let mut active = use_signal(|| 0usize);
    let mut dismissed = use_signal(|| None::<String>);
    let commands = use_hook(|| {
        let mut commands = Commands::default();
        commands.register("at", |state| state.insert_text("@"));
        commands
    });
    let query = editor.with_state(mention_query).flatten();
    let users: Vec<&'static str> = match &query {
        Some(query) if dismissed.read().as_ref() != Some(query) => USERS
            .into_iter()
            .filter(|user| user.starts_with(query.as_str()))
            .collect(),
        _ => Vec::new(),
    };
    let current = active() % users.len().max(1);
    let pick = move |user: &'static str| {
        editor.edit(move |state| insert_mention(state, user));
        let mut active = active;
        active.set(0);
    };
    let picked = users.get(current).copied();
    let count = users.len();
    let intercept = move |input: EditorInput| -> bool {
        let Some(user) = picked else {
            return false;
        };
        match input.key() {
            Some("ArrowDown") => active.set((current + 1) % count),
            Some("ArrowUp") => active.set((current + count - 1) % count),
            Some("Enter" | "Tab") => pick(user),
            Some("Escape") => dismissed.set(query.clone()),
            _ => return false,
        }
        true
    };
    let overlay = (!users.is_empty()).then(|| {
        rsx! {
            ul { role: "listbox", id: "mention-list", "aria-label": "People",
                style: "margin: 0; padding: 4px; list-style: none; background: white; border: 1px solid #888",
                for (index, user) in users.iter().copied().enumerate() {
                    li {
                        id: "mention-{user}",
                        role: "option",
                        "aria-selected": index == current,
                        style: if index == current { "background: #ddf" } else { "" },
                        onclick: move |_| pick(user),
                        "@{user}"
                    }
                }
            }
        }
    });

    rsx! {
        RichTextEditor {
            label: "Message",
            handle: editor,
            commands,
            tools: vec![RichTextTool::new("at", "Mention someone", rsx! { "@" })],
            nodes: NodeViews::new().with("mention", Mention),
            registry: registry(),
            intercept,
            overlay,
            active_descendant: picked.map(|user| format!("mention-{user}")),
            value: doc(),
            onchange: move |next| doc.set(next),
        }
        pre { id: "out", {doc.read().plain_text()} }
    }
}
