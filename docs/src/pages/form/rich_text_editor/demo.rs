use super::NotesCopy;
use crate::components::{DemoValues, field_props};
use dioxus::prelude::*;
use libero::components::rich_text::{
    Attrs, Builtin, Chord, Commands, Doc, EditorInput, EditorState, Keymap, NodeRegistry, NodeSpec,
    NodeViewProps, NodeViews, Position, RichTextTool, Selection, use_rich_text_editor,
};
use libero::components::{ComboboxOption, Flex, Paper, RichTextEditor, Text};
use libero::sx::sx;

/// The demo's extension switches: a command, mentions and a keymap.
#[derive(Clone, Copy)]
pub struct Extensions {
    pub shout: bool,
    pub mentions: bool,
    pub keymap: bool,
}

impl Extensions {
    pub fn of(values: &DemoValues) -> Self {
        Self {
            shout: values.str("commands") == "true",
            mentions: values.str("mentions") == "true",
            keymap: values.str("keymap") == "true",
        }
    }

    pub fn any(self) -> bool {
        self.shout || self.mentions || self.keymap
    }

    /// Something binds a chord, so the snippet prints `keymap()`.
    pub fn binds(self) -> bool {
        self.shout || self.mentions || self.keymap
    }
}

// demo-code: mention start
#[component]
fn Mention(props: NodeViewProps) -> Element {
    let user = props
        .attrs
        .get("user")
        .and_then(|user| user.as_str())
        .unwrap_or("?");
    rsx! { b { "@{user}" } }
}

fn registry() -> NodeRegistry {
    let mut registry = NodeRegistry::default();
    registry
        .register(NodeSpec::inline("mention").attr("user", "ada"))
        .unwrap();
    registry
}
// demo-code: mention end

// demo-code: mentions start
const PEOPLE: [&str; 4] = ["ada", "alan", "grace", "linus"];

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
// demo-code: mentions end

/// The demo's editor; the mention list's hooks run always, its props only with `mentions` on.
#[component]
pub fn NotesEditor(values: DemoValues, mut doc: Signal<Doc>) -> Element {
    let on = Extensions::of(&values);
    // demo-code: mentions-body start
    let editor = use_rich_text_editor();
    let mut active = use_signal(|| 0usize);
    let mut dismissed = use_signal(|| None::<String>);
    let query = editor.with_state(mention_query).flatten();
    let people: Vec<&'static str> = match &query {
        Some(query) if dismissed.read().as_ref() != Some(query) => PEOPLE
            .into_iter()
            .filter(|user| user.starts_with(query.as_str()))
            .collect(),
        _ => Vec::new(),
    };
    let current = active() % people.len().max(1);
    let picked = people.get(current).copied();
    let count = people.len();
    let pick = move |user: &'static str| {
        editor.edit(move |state| insert_mention(state, user));
        let mut active = active;
        active.set(0);
    };
    // The list takes the arrows, Enter and Escape while it is open; the rest types.
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
    let overlay = (!people.is_empty()).then(|| {
        rsx! {
            Paper { shadow: "md", bordered: true, role: "listbox", "aria-label": "People",
                sx: sx().padding("xs").min_width("10rem"),
                for (index, user) in people.iter().copied().enumerate() {
                    ComboboxOption { id: "mention-{user}", active: index == current,
                        onpick: move |_| pick(user),
                        "@{user}"
                    }
                }
            }
        }
    });
    // demo-code: mentions-body end
    let (nodes, tools) = match on.mentions {
        true => (
            NodeViews::new().with("mention", Mention),
            vec![RichTextTool::new("at", "Mention someone", rsx! { "@" })],
        ),
        false => (NodeViews::new(), vec![]),
    };
    // Without the switch the list never opens, so `@` stays plain text.
    let picked = picked.filter(|_| on.mentions);
    let mut intercept = intercept;
    let field = field_props::<NotesCopy>(&values);
    rsx! {
        RichTextEditor {
            size: values.str("size"),
            radius: values.str("radius"),
            label: field.label,
            aria_label: field.aria_label,
            description: field.description,
            helper: field.helper,
            status: field.status,
            placeholder: (values.str("placeholder") == "true").then(|| "Start writing".to_string()),
            toolbar: values.str("toolbar") == "true",
            required: (values.str("required") == "true").then_some(true),
            disabled: (values.str("disabled") == "true").then_some(true),
            readonly: (values.str("readonly") == "true").then_some(true),
            commands: demo_commands(on),
            keymap: demo_keymap(on),
            handle: editor,
            tools,
            nodes,
            registry: on.mentions.then(registry),
            intercept: move |input| on.mentions && intercept(input),
            overlay: overlay.filter(|_| on.mentions),
            active_descendant: picked.map(|user| format!("mention-{user}")),
            overlay_results: count,
            value: doc(),
            onchange: move |next| doc.set(next),
        }
    }
}

const MESSAGE_LIMIT: usize = 140;

/// A chat box: Enter sends, `max_length` caps it and the handle clears it.
#[component]
pub fn MessageBox() -> Element {
    let editor = use_rich_text_editor();
    let mut messages = use_signal(Vec::<String>::new);
    let left = MESSAGE_LIMIT.saturating_sub(editor.plain_text().chars().count());
    rsx! {
        Flex { direction: "column", gap: "sm",
            Text { size: "sm",
                "A chat box: onsubmit sends on Enter, max_length caps the text and the handle's clear empties the box."
            }
            RichTextEditor {
                label: "Message",
                helper: format!("Enter sends, Shift+Enter breaks the line. {left} characters left."),
                placeholder: "Write a message",
                toolbar: false,
                handle: editor,
                max_length: MESSAGE_LIMIT,
                onsubmit: move |doc: Doc| {
                    messages.write().push(doc.to_markdown());
                    editor.clear();
                },
            }
            for (index, message) in messages.read().iter().enumerate() {
                Text { key: "{index}", size: "sm", white_space: "pre-wrap", "{message}" }
            }
        }
    }
}

fn demo_commands(on: Extensions) -> Commands {
    let mut commands = Commands::default();
    if on.shout {
        // demo-code: shout-command start
        commands.register("shout", |state| {
            let text = state.selected_text().to_uppercase();
            !text.is_empty() && state.insert_text(&text)
        });
        // demo-code: shout-command end
    }
    if on.mentions {
        // demo-code: at-command start
        commands.register("at", |state| state.insert_text("@"));
        commands.register("mention", |state| {
            let mention = registry().new_inline("mention", Attrs::new());
            mention.is_some_and(|mention| state.insert_inline(mention))
        });
        // demo-code: at-command end
    }
    commands
}

fn demo_keymap(on: Extensions) -> Keymap {
    let mut keymap = Keymap::default();
    if on.shout {
        // demo-code: shout-chord start
        keymap.bind(Chord::parse("Mod+Shift+1").unwrap(), "shout");
        // demo-code: shout-chord end
    }
    if on.mentions {
        // demo-code: mention-chord start
        keymap.bind(Chord::parse("Mod+Shift+2").unwrap(), "mention");
        // demo-code: mention-chord end
    }
    if on.keymap {
        // demo-code: keymap-chords start
        keymap.bind(Chord::parse("Mod+Shift+h").unwrap(), Builtin::Heading2);
        keymap.unbind_command(Builtin::Rule);
        // demo-code: keymap-chords end
    }
    keymap
}
