use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::rich_text::Doc;
use libero::components::{Code, FieldPart, FieldStatus, RichTextEditor, Text};

const SAMPLE: &str = "## Release notes\n\nStart a line with # and a space for a heading, or wrap a word in **two stars**.\n\n- Undo with Ctrl+Z\n- Bold with Ctrl+B";

#[component]
pub fn RichTextEditorPage() -> Element {
    let mut doc = use_signal(|| Doc::from_markdown(SAMPLE));

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
                    prop("size", "Size").default("md").doc("Padding and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of `size`."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the toolbar. It names the text."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. Formatting rules or limits."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
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
                .key(["Ctrl+Shift+8"], "Bulleted list.")
                .key(["Ctrl+Shift+7"], "Numbered list.")
                .key(["Ctrl+Shift+B"], "Quote.")
                .key(["Ctrl+Shift+Enter"], "Horizontal rule.")
                .key(["Ctrl+Enter"], "In a code block: leaves it for a new paragraph after it. Elsewhere the key passes on, so your own `Ctrl+Enter` (send) still runs.")
                .key(["ArrowDown"], "On the last line of a code block that ends the document: leaves it for a new paragraph. Clicking below the last block does the same.")
                .key(["Tab", "Shift+Tab"], "In a list item: nests it under the item before, or moves it out. Elsewhere Tab moves focus on as usual.")
                .key(["Shift+Enter"], "Line break inside the block.")
                .key(["Ctrl+Z"], "Undo.")
                .key(["Ctrl+Shift+Z", "Ctrl+Y"], "Redo.")
                .key(["Ctrl+K"], "Opens the link dialog: links the selection, or edits or removes the link at the caret.")
                .key(["Ctrl+Shift+U"], "Removes the link at the caret.")
                .key(["Ctrl+/"], "Lists the editor's shortcuts, from the live keymap.")
                .key(["Alt+F10"], "Moves focus to the toolbar; the arrow keys move through it, Escape returns to the text.")
                .handles([
                    "The text is a `role=\"textbox\"` with `aria-multiline`, named by `label` and described by `description` and `helper`.",
                    "The toolbar is one tab stop with arrow-key movement, named by the localization's `rich_text_editor` words. Each mark and block button reports `aria-pressed`.",
                    "The toolbar keeps focus and the selection in the text when clicked.",
                    "Each toolbar button shows its name and its chord in a tooltip on hover and keyboard focus, and carries the chord as `aria-keyshortcuts`.",
                    "In a narrow column the toolbar stays one row: the less used buttons move into a \"More formatting\" menu as `menuitemcheckbox` items, so the arrow keys only reach what is shown. Bold, italic and the text type menu always stay.",
                    "A shortcut that toggles a mark, list, quote or block type is announced through a polite live region (\"Bold on\", \"Heading 2\").",
                    "The link dialog focuses its labelled URL field; a refused scheme shows as that field's error. Closing it puts the caret back in the text.",
                    "The text type menu is a menu button whose name includes the current type, with `menuitemradio` items. So is the language menu of a code block.",
                    "Every edit goes through the document model, so undo, the `onchange` value and the screen stay in step. Input methods (IME) compose natively and are taken in when the composition ends.",
                ])
                .must([
                    "Leave `label` unset only when something else names the editor, such as an `aria_label`.",
                    "Document custom chords you bind in `keymap` for your users.",
                    "Make a `NodeViews` atom name its node in text (a mention shows `@name`): it is a non-editable island a screen reader reads as is.",
                ])
                .limits([
                    "On Blitz (native) the document is shown read-only.",
                    "Paste takes plain text only, one block per line.",
                    "Copy and cut write the selection as plain text and as `text/markdown`; custom nodes write through their `NodeSpec::markdown` (pass `registry`). Cut is one undo step.",
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
                                "status: FieldStatus::Warning(\"Consider a shorter title.\".into())".to_string(),
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
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("readonly"),
                ],
                render: move |values: DemoValues| rsx! {
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
                            "warning" => FieldStatus::Warning("Consider a shorter title.".to_string()),
                            "error" => FieldStatus::Error("Write a few words.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "Start writing".to_string()),
                        toolbar: values.str("toolbar") == "true",
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        readonly: (values.str("readonly") == "true").then_some(true),
                        value: doc(),
                        onchange: move |next| doc.set(next),
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
            ". A code block shows its source with fences while the caret is in it."
        }
    }
}
