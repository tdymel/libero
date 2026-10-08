use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, FieldCopy, Wrap, a11y,
    field_controls, indent, prop, props, status_prop,
};
use dioxus::prelude::*;
use libero::components::rich_text::Doc;
use libero::components::{Code, FieldPart, Flex, Text};
use libero::use_theme;

mod demo;
use demo::{Extensions, NotesEditor};

struct NotesCopy;

impl FieldCopy for NotesCopy {
    const LABEL: &'static str = "Notes";
    const DESCRIPTION: &'static str = "What changed in this release.";
    const HELPER: &'static str = "Markdown shortcuts work as you type.";
    const WARNING: &'static str = "Long notes get cut short in the feed.";
    const ERROR: &'static str = "Write a few words.";
}

/// The live editor and its extensions, printed from its text by `wrap_extensions`.
const FILE: DemoFile = DemoFile(include_str!("rich_text_editor/demo.rs"));

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

/// A section of the demo file one level in, for a function body.
fn indented(name: &str) -> String {
    indent(&FILE.section(name)).trim_end().to_string()
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
        items.push(FILE.section("mention"));
        items.push(format!(
            "use libero::components::{{ComboboxOption, Paper}};\nuse libero::sx::sx;\n\n{}",
            FILE.section("mentions")
        ));
    }
    if on.shout || on.mentions {
        let registered: Vec<String> = [(on.shout, "shout-command"), (on.mentions, "at-command")]
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, name)| indented(name))
            .collect();
        items.push(format!(
            "fn commands() -> Commands {{\n    let mut commands = Commands::default();\n{}\n    commands\n}}",
            registered.join("\n")
        ));
    }
    if on.binds() {
        let bound: Vec<String> = [
            (on.shout, "shout-chord"),
            (on.mentions, "mention-chord"),
            (on.keymap, "keymap-chords"),
        ]
        .into_iter()
        .filter(|(on, _)| *on)
        .map(|(_, name)| indented(name))
        .collect();
        items.push(format!(
            "fn keymap() -> Keymap {{\n    let mut keymap = Keymap::default();\n{}\n    keymap\n}}",
            bound.join("\n")
        ));
    }
    if on.mentions {
        items.push(FILE.section("mentions-body"));
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
                        .doc("Floats at the caret while `Some`, such as a mention list: under the caret's line, above it near the window's bottom edge, mirrored in right-to-left text. It renders in the page's portal, so a parent's `overflow` does not clip it and it does not inherit the editor's CSS context. Focus stays in the text; pressing the overlay does not take it."),
                    prop("active_descendant", "String")
                        .doc("The id of the overlay's highlighted option. While `overlay` is `Some` the text carries it as `aria-activedescendant`, with `aria-controls` naming the overlay and `aria-autocomplete=\"list\"`."),
                    prop("overlay_results", "usize")
                        .doc("How many options the overlay lists. While `overlay` is `Some`, a change is announced through the editor's polite live region (\"2 results\", or \"No results\" at 0; `RichTextEditorLabels::results` and `nothing_found`)."),
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
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label. Inside a `Form`, an empty one fails the submit."),
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
                    "The text is a multiline `role=\"textbox\"`, named by `label` and described by `description`, `helper` and a hint read on focus: in a list Tab indents, and Escape, then Tab leaves (`RichTextEditorLabels::leave_hint`).",
                    "The toolbar is one tab stop, moved through with the arrows. Clicking it keeps focus and the selection in the text. Each mark and block button reports `aria-pressed`, shows its name and chord in a tooltip on hover and focus, and carries the chord as `aria-keyshortcuts`.",
                    "In a narrow column the toolbar stays one row: the less used buttons move into a \"More formatting\" menu, so the arrows only reach what is shown. Bold, italic and the text type menu always stay.",
                    "A shortcut that toggles a mark, list, quote or block type is said in a polite live region, \"Bold on\", \"Heading 2\".",
                    "The text type menu and a code block's language menu are menu buttons whose name includes the current choice, \"Code language: Rust\". The language button on the fence is not a tab stop; `Ctrl+Shift+L` reaches it.",
                    "The link dialog focuses its labelled URL field and shows a refused scheme as that field's error. Closing it puts the caret back in the text.",
                    "Every edit goes through the document model, so undo, the `onchange` value and the screen stay in step. Input methods (IME) compose natively.",
                    "While `overlay` is set, the text carries `aria-controls`, `aria-autocomplete=\"list\"` and `aria-activedescendant` from `active_descendant`, so a screen reader reads the highlighted option. `overlay_results` says the option count, \"2 results\", in the live region when it changes.",
                ])
                .must([
                    "Leave `label` unset only when something else names the editor, such as an `aria_label`.",
                    "Tell your users about the chords you bind in `keymap`.",
                    "Bind only chords the browser leaves to the page. `Ctrl+N`, `Ctrl+T`, `Ctrl+W`, `Ctrl+Tab` and their Shift forms never reach it (on a Mac also `Cmd+Q`, `Cmd+H`, `Cmd+M`), and `Ctrl+P`, `Ctrl+S`, `Ctrl+D`, `Ctrl+F`, `Ctrl+L` and `Ctrl+Shift+I`/`J`/`C` belong to the browser too. A chord you unbind goes back to the browser: without Underline, `Ctrl+U` opens the page source. On Windows `Ctrl+Alt` is AltGr, and where it types a character the character wins (`Ctrl+Alt+2` types `²` on a German layout).",
                    "Make a `NodeViews` atom show its node's name as text (a mention shows `@name`). A screen reader reads the non-editable island as it is.",
                    "Give an `overlay` list `role=\"listbox\"` with an id per `role=\"option\"`, pass the highlighted one as `active_descendant` and the count as `overlay_results`, and steer it with the keyboard through `intercept`. Escape should close it.",
                    "Name each `RichTextTool` with its `label`: the button shows only its icon.",
                ])
                .example("Release notes with `label: \"Notes\"`: a screen reader reads a multiline \"Notes\" text box. `Ctrl+B` says \"Bold on\", and `Alt+F10` moves to the toolbar, where the Bold button reads as pressed.")
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
                wide_preview: true,
                children_text: "",
                fixed: vec![
                    "value: doc()".to_string(),
                    "onchange: move |next| doc.set(next)".to_string(),
                ],
                controls: [vec![
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
                        .default("sm"),
                ], field_controls::<NotesCopy>(), vec![
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
                                "overlay_results: count".to_string(),
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
                ]].concat(),
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

            DocSection {
                title: "Code blocks",
                Text {
                    "A code block shows its source with fences while the caret is in it; the language button on its opening fence, the toolbar's language menu or Ctrl+Shift+L change its language."
                }
            }

            DocSection {
                title: "Mentions",
                Text {
                    "The mentions switch adds an @ list at the caret, built from "
                    Code { source: "intercept" }
                    ", "
                    Code { source: "overlay" }
                    " and a custom node."
                }
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
            "."
        }
    }
}
