use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::rich_text::{
    Attrs, Builtin, Chord, Commands, Doc, EditorInput, EditorState, Keymap, NodeRegistry, NodeSpec,
    NodeViewProps, NodeViews, Position, RichTextTool, Selection, use_rich_text_editor,
};
use libero::components::{Code, FieldPart, FieldStatus, Flex, Paper, RichTextEditor, Text};
use libero::sx::sx;
use libero::use_theme;

static SAMPLE: &str = r#"## Release notes

Start a line with # and a space for a heading, or wrap a word in **two stars**. Marks mix: *italic*, <u>underline</u>, ~~strike~~, `code` and [links](https://libero-ui.dev).

### Shortcuts

- Undo with Ctrl+Z
- Bold with Ctrl+B
  - Nest an item with Tab

1. Type three backticks and a language
2. Press Enter for a code block

> A quote holds its own paragraphs.
>
> - and lists

```rust
fn main() {
    println!("Click to edit");
}
```

---

Below the rule, the next section starts."#;

// What the extension switches print above the rsx; the live editor runs the functions below.
// `static`, not `const`: the snippet scan would compile these fragments alone, not in the Demo.
static MENTION: &str = r#"#[component]
fn Mention(props: NodeViewProps) -> Element {
    let user = props.attrs.get("user").and_then(|user| user.as_str()).unwrap_or("?");
    rsx! { b { "@{user}" } }
}

fn registry() -> NodeRegistry {
    let mut registry = NodeRegistry::default();
    registry.register(NodeSpec::inline("mention").attr("user", "ada")).unwrap();
    registry
}"#;

static SHOUT_COMMAND: &str = r#"    commands.register("shout", |state| {
        let text = state.selected_text().to_uppercase();
        !text.is_empty() && state.insert_text(&text)
    });"#;

static AT_COMMAND: &str = r#"    commands.register("at", |state| state.insert_text("@"));
    commands.register("mention", |state| {
        let mention = registry().new_inline("mention", Attrs::new());
        mention.is_some_and(|mention| state.insert_inline(mention))
    });"#;

static SHOUT_CHORD: &str = r#"    keymap.bind(Chord::parse("Mod+Shift+1").unwrap(), "shout");"#;
static MENTION_CHORD: &str = r#"    keymap.bind(Chord::parse("Mod+Shift+2").unwrap(), "mention");"#;
static KEYMAP_CHORDS: &str = r#"    keymap.bind(Chord::parse("Mod+Shift+h").unwrap(), Builtin::Heading2);
    keymap.unbind_command(Builtin::Rule);"#;

static MENTIONS: &str = r##"use libero::components::Paper;
use libero::sx::sx;

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
}"##;

// The mention list's state and handlers, printed before the rsx.
static MENTIONS_BODY: &str = r##"let editor = use_rich_text_editor();
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
let overlay = (!people.is_empty()).then(|| rsx! {
    Paper { shadow: "md", bordered: true, role: "listbox", "aria-label": "People",
        sx: sx().padding("xs").min_width("10rem"),
        for (index, user) in people.iter().copied().enumerate() {
            div { id: "mention-{user}", role: "option", "aria-selected": index == current,
                onclick: move |_| pick(user),
                Paper { radius: "sm", color: (index == current).then_some("primary"),
                    sx: sx().padding("0.25rem 0.5rem"),
                    "@{user}"
                }
            }
        }
    }
});"##;

/// The demo's extension switches: a command, mentions and a keymap.
#[derive(Clone, Copy)]
struct Extensions {
    shout: bool,
    mentions: bool,
    keymap: bool,
}

impl Extensions {
    fn of(values: &DemoValues) -> Self {
        Self {
            shout: values.str("commands") == "true",
            mentions: values.str("mentions") == "true",
            keymap: values.str("keymap") == "true",
        }
    }

    fn any(self) -> bool {
        self.shout || self.mentions || self.keymap
    }

    /// Something binds a chord, so the snippet prints `keymap()`.
    fn binds(self) -> bool {
        self.shout || self.mentions || self.keymap
    }
}

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

/// The demo's editor; the mention list's hooks run always, its props only with `mentions` on.
#[component]
fn NotesEditor(values: DemoValues, mut doc: Signal<Doc>) -> Element {
    let on = Extensions::of(&values);
    let editor = use_rich_text_editor();
    let mut active = use_signal(|| 0usize);
    let mut dismissed = use_signal(|| None::<String>);
    let query = match on.mentions {
        true => editor.with_state(mention_query).flatten(),
        false => None,
    };
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
                    div { id: "mention-{user}", role: "option", "aria-selected": index == current,
                        onclick: move |_| pick(user),
                        Paper { radius: "sm", color: (index == current).then_some("primary"),
                            sx: sx().padding("0.25rem 0.5rem"),
                            "@{user}"
                        }
                    }
                }
            }
        }
    });
    let (nodes, tools) = match on.mentions {
        true => (
            NodeViews::new().with("mention", Mention),
            vec![RichTextTool::new("at", "Mention someone", rsx! { "@" })],
        ),
        false => (NodeViews::new(), vec![]),
    };
    rsx! {
        RichTextEditor {
            size: values.str("size"),
            radius: values.str("radius"),
            label: (values.str("label") == "true").then(|| "Notes".to_string()),
            aria_label: (values.str("label") != "true").then_some("Notes"),
            description: (values.str("description") == "true")
                .then(|| "What changed in this release.".to_string()),
            helper: (values.str("helper") == "true")
                .then(|| "Markdown shortcuts work as you type.".to_string()),
            status: match values.str("status").as_str() {
                "warning" => FieldStatus::Warning("Long notes get cut short in the feed.".to_string()),
                "error" => FieldStatus::Error("Write a few words.".to_string()),
                _ => FieldStatus::Valid,
            },
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
            intercept,
            overlay,
            active_descendant: picked.map(|user| format!("mention-{user}")),
            value: doc(),
            onchange: move |next| doc.set(next),
        }
    }
}

fn shout(state: &mut EditorState) -> bool {
    let text = state.selected_text().to_uppercase();
    !text.is_empty() && state.insert_text(&text)
}

/// A node command: a mention of the registry's default user.
fn mention(state: &mut EditorState) -> bool {
    let mention = registry().new_inline("mention", Attrs::new());
    mention.is_some_and(|mention| state.insert_inline(mention))
}

fn chord(chord: &str) -> Chord {
    Chord::parse(chord).expect("a valid demo chord")
}

fn demo_commands(on: Extensions) -> Commands {
    let mut commands = Commands::default();
    if on.shout {
        commands.register("shout", shout);
    }
    if on.mentions {
        commands.register("at", |state| state.insert_text("@"));
        commands.register("mention", mention);
    }
    commands
}

fn demo_keymap(on: Extensions) -> Keymap {
    let mut keymap = Keymap::default();
    if on.shout {
        keymap.bind(chord("Mod+Shift+1"), "shout");
    }
    if on.mentions {
        keymap.bind(chord("Mod+Shift+2"), "mention");
    }
    if on.keymap {
        keymap.bind(chord("Mod+Shift+h"), Builtin::Heading2);
        keymap.unbind_command(Builtin::Rule);
    }
    keymap
}

/// The items the extension switches add, printed above the rsx.
fn wrap_extensions(values: &DemoValues, code: &str) -> String {
    let on = Extensions::of(values);
    if !on.any() {
        return code.to_string();
    }
    let used: Vec<&str> = [
        (on.mentions, "Attrs"),
        (on.keymap, "Builtin"),
        (on.binds(), "Chord"),
        (on.shout || on.mentions, "Commands"),
        (on.mentions, "EditorInput"),
        (on.mentions, "EditorState"),
        (on.binds(), "Keymap"),
        (on.mentions, "NodeRegistry"),
        (on.mentions, "NodeSpec"),
        (on.mentions, "NodeViewProps"),
        (on.mentions, "NodeViews"),
        (on.mentions, "Position"),
        (on.mentions, "RichTextTool"),
        (on.mentions, "Selection"),
        (on.mentions, "use_rich_text_editor"),
    ]
    .into_iter()
    .filter_map(|(on, name)| on.then_some(name))
    .collect();
    let mut items = vec![format!(
        "use libero::components::rich_text::{{{}}};",
        used.join(", ")
    )];
    if on.mentions {
        items.push(MENTION.to_string());
        items.push(MENTIONS.to_string());
    }
    if on.shout || on.mentions {
        let registered: Vec<&str> = [(on.shout, SHOUT_COMMAND), (on.mentions, AT_COMMAND)]
            .into_iter()
            .filter_map(|(on, line)| on.then_some(line))
            .collect();
        items.push(format!(
            "fn commands() -> Commands {{\n    let mut commands = Commands::default();\n{}\n    commands\n}}",
            registered.join("\n")
        ));
    }
    if on.binds() {
        let bound: Vec<&str> = [
            (on.shout, SHOUT_CHORD),
            (on.mentions, MENTION_CHORD),
            (on.keymap, KEYMAP_CHORDS),
        ]
        .into_iter()
        .filter_map(|(on, line)| on.then_some(line))
        .collect();
        items.push(format!(
            "fn keymap() -> Keymap {{\n    let mut keymap = Keymap::default();\n{}\n    keymap\n}}",
            bound.join("\n")
        ));
    }
    if on.mentions {
        items.push(MENTIONS_BODY.to_string());
    }
    items.push(code.to_string());
    items.join("\n\n")
}

/// One line per extension switch that changes what the editor does, so a reader knows what to try.
fn captions(values: &DemoValues) -> Vec<&'static str> {
    let on = Extensions::of(values);
    [
        (on.shout, "Custom command: select a word and press Ctrl+Shift+1 (Cmd on a Mac) to upper-case it, one undo step."),
        (on.mentions, "Mentions: type @ and a name, or press the @ button. While the list is open, intercept gives it the arrow keys, Enter and Escape; overlay floats it at the caret; the handle's edit swaps the typed @name for a mention node, drawn by your own component, in one undo step. Ctrl+Shift+2 runs a node command that inserts @ada."),
        (on.keymap, "Custom keymap: Ctrl+Shift+H makes a heading, and Ctrl+Shift+Enter no longer adds a rule. Ctrl+/ lists the live keymap."),
        (values.str("toolbar") != "true", "No toolbar: the keymap, or your own buttons through a handle, drive the editor."),
    ]
    .into_iter()
    .filter_map(|(on, caption)| on.then_some(caption))
    .collect()
}

#[component]
pub fn RichTextEditorPage() -> Element {
    let theme = use_theme();
    let doc = use_signal(|| Doc::from_markdown(SAMPLE));

    rsx! {
        DocPage {
            title: "RichTextEditor",
            source: "libero/src/components/form/rich_text_editor/view.rs",
            markdown: "/md/rich_text_editor.md",
            properties: vec![
                props("RichTextEditor", vec![
                    prop("value", "Option<Doc>")
                        .doc("The document. Leave it out and the editor keeps its own. An echo of a document the editor sent is not a reset."),
                    prop("onchange", "EventHandler<Doc>")
                        .doc("Fires with the new document after every edit. Moving the caret does not fire."),
                    prop("validate", "Validators<Doc>")
                        .doc("Rules over the document, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<Doc>")
                        .doc("A path such as `Post::FIELDS.body()` binds the document to the surrounding `Form`'s value when the editor has no `onchange`."),
                    prop("placeholder", "String")
                        .doc("Shown while the document is one empty paragraph."),
                    prop("toolbar", "bool")
                        .default("true")
                        .doc("Shows the formatting toolbar above the text: marks, link, text type menu, lists, quote, code block, undo and redo. A narrow toolbar moves what does not fit into a More menu."),
                    prop("keymap", "Keymap")
                        .default("Keymap::default()")
                        .doc("Which chords run which commands. `Mod` is Cmd on Apple platforms and Ctrl elsewhere."),
                    prop("commands", "Commands")
                        .default("Commands::default()")
                        .doc("What the keymap and the toolbar run. Register your own command under a name and bind a chord to it."),
                    prop("handle", "RichTextHandle")
                        .doc("From `use_rich_text_editor()`: runs commands from your own toolbar (`run`) and reads the state reactively (`is_active`, `block_kind`, `list_kind`, `in_quote`, `can_undo`, `can_redo`). One handle drives one editor."),
                    prop("nodes", "NodeViews")
                        .default("NodeViews::new()")
                        .doc("Your component per custom node name, e.g. a mention. It gets `NodeViewProps { name, attrs, children }` and renders `children` exactly once. Built-ins (`paragraph`, `heading`, `quote`, `list`, `list_item`, `rule`) take a view under their name too; `code_block` stays fixed."),
                    prop("registry", "NodeRegistry")
                        .doc("Your node types. Copy and cut write your nodes through their `to_markdown`, and a debug build warns about a `nodes` name that is not registered, such as a typo."),
                    prop("intercept", "Callback<EditorInput, bool>")
                        .doc("Sees each key press (`EditorInput::Key`, before the keymap) and typed text (`EditorInput::Text`) first; return `true` to take it over, and the editor does nothing with it. Android soft keyboards report most keys as `Unidentified` and compose their text, which never arrives as `Text`; Enter still arrives as a key. Read typed text through the handle's `with_state` instead."),
                    prop("overlay", "Element")
                        .doc("Floats at the caret while `Some`, such as a mention list: under the caret's line, above it near the window's bottom edge, mirrored in right-to-left text. Focus stays in the text; pressing the overlay does not take it."),
                    prop("active_descendant", "String")
                        .doc("The id of the overlay's highlighted option. While `overlay` is `Some` the text carries it as `aria-activedescendant`, with `aria-controls` naming the overlay and `aria-autocomplete=\"list\"`."),
                    prop("tools", "Vec<RichTextTool>")
                        .default("vec![]")
                        .doc("Your toolbar buttons, after the block buttons: `RichTextTool::new(command, label, icon)` runs the command by name; `.active(fn)` makes it a toggle with `aria-pressed`. They never move into the More menu."),
                    prop("size", "Size").default(theme.textarea.size.as_str()).doc("Padding and font size."),
                    prop("radius", "Size")
                        .default(theme.textarea.radius.as_str())
                        .doc("Corner radius, independent of `size`."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the toolbar. It names the text."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. Formatting rules or limits."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables and dims the field and hides the toolbar."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Shows the document, focusable but not editable, without the toolbar."),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the toolbar."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "The bordered box around the text."),
                    (FieldPart::Control, "The editable text the label names."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ]).extends("div"),
            ],
            accessibility: a11y()
                .key(["Ctrl+B"], "Bold. `Mod` in a `Keymap` is Cmd on Apple platforms, Ctrl elsewhere; the table shows Ctrl.")
                .key(["Ctrl+I"], "Italic.")
                .key(["Ctrl+U"], "Underline.")
                .key(["Ctrl+Shift+S", "Ctrl+Shift+X"], "Strikethrough.")
                .key(["Ctrl+E"], "Inline code.")
                .key(["Ctrl+Alt+0"], "Paragraph.")
                .key(["Ctrl+Alt+1"], "Heading 1. `Ctrl+Alt+2` to `Ctrl+Alt+6` make headings 2 to 6.")
                .key(["Ctrl+Alt+C"], "Code block.")
                .key(["Ctrl+Shift+L"], "In a code block: opens its language menu. Choosing or Escape returns to the caret.")
                .key(["Ctrl+Shift+8"], "Bulleted list.")
                .key(["Ctrl+Shift+7"], "Numbered list.")
                .key(["Ctrl+Shift+B"], "Quote.")
                .key(["Ctrl+Shift+Enter"], "Horizontal rule.")
                .key(["Ctrl+Enter"], "In a code block: leaves it for a new paragraph after it. Elsewhere the key passes on, so your own `Ctrl+Enter` (send) still runs.")
                .key(["ArrowDown"], "On the last line of a code block that ends the document: leaves it for a new paragraph. Clicking below the last block does the same.")
                        .key(["Enter"], "Twice at the end of a code block: the second drops the empty line and leaves the block, also on a touch keyboard.")
                .key(["Tab", "Shift+Tab"], "In a list item: nests it under the item before, or moves it out. Elsewhere Tab moves focus on as usual.")
                .key(["Escape", "Tab"], "Escape, then Tab or Shift+Tab: leaves the editor, also from a list item.")
                .key(["Shift+Enter"], "Line break inside the block.")
                .key(["Ctrl+Z"], "Undo.")
                .key(["Ctrl+Shift+Z", "Ctrl+Y"], "Redo.")
                .key(["Ctrl+K"], "Opens the link dialog: links the selection, or edits or removes the link at the caret.")
                .key(["Ctrl+Shift+U"], "Removes the link at the caret.")
                .key(["Ctrl+/"], "Lists the editor's shortcuts, from the live keymap.")
                .key(["Alt+F10"], "Moves focus to the toolbar; the arrow keys move through it, Escape returns to the text.")
                .handles([
                    "The text is a `role=\"textbox\"` with `aria-multiline`, named by `label` and described by `description` and `helper`.",
                    "The text is described by a hint read on focus: in a list Tab indents, and Escape, then Tab leaves (`RichTextEditorLabels::leave_hint`).",
                    "The toolbar is one tab stop with arrow-key movement, named by the localization's `rich_text_editor` words. Each mark and block button reports `aria-pressed`.",
                    "The toolbar keeps focus and the selection in the text when clicked.",
                    "Each toolbar button shows its name and its chord in a tooltip on hover and keyboard focus, and carries the chord as `aria-keyshortcuts`.",
                    "In a narrow column the toolbar stays one row: the less used buttons move into a \"More formatting\" menu as `menuitemcheckbox` items, so the arrow keys only reach what is shown. Bold, italic and the text type menu always stay.",
                    "A shortcut that toggles a mark, list, quote or block type is announced through a polite live region (\"Bold on\", \"Heading 2\").",
                    "The link dialog focuses its labelled URL field; a refused scheme shows as that field's error. Closing it puts the caret back in the text.",
                    "The text type menu is a menu button whose name includes the current type, with `menuitemradio` items. So is the language menu of a code block, in the toolbar and on its opening fence (\"Code language: Rust\"); the fence button is not a tab stop, `Ctrl+Shift+L` reaches it.",
                    "Every edit goes through the document model, so undo, the `onchange` value and the screen stay in step. Input methods (IME) compose natively and are taken in when the composition ends.",
                    "While `overlay` is set, the text carries `aria-controls` naming it, `aria-autocomplete=\"list\"` and `aria-activedescendant` from `active_descendant`, so a screen reader announces the highlighted option.",
                ])
                .must([
                    "Leave `label` unset only when something else names the editor, such as an `aria_label`.",
                    "Document custom chords you bind in `keymap` for your users.",
                    "Bind chords the browser leaves to the page. `Ctrl+N`, `Ctrl+T`, `Ctrl+W`, `Ctrl+Tab` and their Shift forms never reach it (on a Mac also `Cmd+Q`, `Cmd+H`, `Cmd+M`). A chord you unbind goes back to the browser: without Underline, `Ctrl+U` opens the page source. `Ctrl+P`, `Ctrl+S`, `Ctrl+D`, `Ctrl+F`, `Ctrl+L` and `Ctrl+Shift+I`/`J`/`C` belong to the browser too. On Windows `Ctrl+Alt` is AltGr, which types characters on many layouts: where it types one, the character wins over the chord (`Ctrl+Alt+2` types `²` on a German layout).",
                    "Make a `NodeViews` atom name its node in text (a mention shows `@name`): it is a non-editable island a screen reader reads as is.",
                    "Give an `overlay` list `role=\"listbox\"` with an id per `role=\"option\"`, pass the highlighted one as `active_descendant`, and steer it with the keyboard through `intercept`; Escape should close it.",
                    "Name each `RichTextTool` with its `label`: the button shows only its icon.",
                ])
                .limits([
                    "On Blitz (native) the document is shown read-only.",
                    "Copy and cut write the selection as Markdown, as plain text and as `text/markdown`: a quote as `>`, a code block fenced with its language, lists, headings, rules, marks and links as written. Custom nodes write through their `NodeSpec::markdown` (pass `registry`). Cut is one undo step.",
                    "Paste reads Markdown back into formatted blocks; custom nodes stay text. Text from elsewhere is read the same way, one paragraph per line. In a code block it is pasted as is.",
                    "Drag and drop of text and spellcheck replacements are ignored, so `spellcheck` is off.",
                ]),
            lead: rich_lead(),
            // snippet: let mut doc = use_signal(|| rich_text::Doc::from_markdown("## Release notes"));
            Demo {
                component: "RichTextEditor",
                children_text: "",
                fixed: vec![
                    "value: doc()".to_string(),
                    "onchange: move |next| doc.set(next)".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .labels(["Valid", "Warning", "Error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"Long notes get cut short in the feed.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Write a few words.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Notes\"".to_string()],
                            _ => vec!["aria_label: \"Notes\"".to_string()],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"What changed in this release.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"Markdown shortcuts work as you type.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Start writing\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("toolbar").default("true").code(|_, values| {
                        match values.str("toolbar").as_str() {
                            "true" => vec![],
                            _ => vec!["toolbar: false".to_string()],
                        }
                    }),
                    // `commands` and `keymap` print the one function each that the extension switches add to.
                    Control::switch("commands").code(|_, values| {
                        let on = Extensions::of(values);
                        match on.shout || on.mentions {
                            true => vec!["commands: commands()".to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("mentions").code(|_, values| {
                        match values.str("mentions").as_str() {
                            "true" => vec![
                                "handle: editor".to_string(),
                                "tools: vec![RichTextTool::new(\"at\", \"Mention someone\", rsx! { \"@\" })]".to_string(),
                                "nodes: NodeViews::new().with(\"mention\", Mention)".to_string(),
                                "registry: registry()".to_string(),
                                "intercept".to_string(),
                                "overlay".to_string(),
                                "active_descendant: picked.map(|user| format!(\"mention-{user}\"))".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    Control::switch("keymap").code(|_, values| {
                        match Extensions::of(values).binds() {
                            true => vec!["keymap: keymap()".to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("readonly"),
                ],
                wrap: Wrap(wrap_extensions),
                render: move |values: DemoValues| rsx! {
                    Flex { direction: "column", gap: "sm",
                        NotesEditor { values: values.clone(), doc }
                        for caption in captions(&values) {
                            Text { size: "sm", "{caption}" }
                        }
                    }
                },
            }
        }
    }
}

fn rich_lead() -> Element {
    rsx! {
        Text {
            "A rich text field over a "
            Code { source: "Doc" }
            ": paragraphs, headings, lists, quotes, code blocks and marks, with undo, the usual "
            "shortcuts and Markdown typing shortcuts. "
            Code { source: "Doc::to_markdown" }
            " and "
            Code { source: "Doc::from_markdown" }
            " convert it; the types live in "
            Code { source: "libero::components::rich_text" }
            ". A code block shows its source with fences while the caret is in it; the language button on its opening fence, the toolbar's language menu or Ctrl+Shift+L change its language. "
            "The mentions switch adds an @ list at the caret, built from "
            Code { source: "intercept" }
            ", "
            Code { source: "overlay" }
            " and a custom node."
        }
    }
}
