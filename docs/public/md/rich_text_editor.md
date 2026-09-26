# RichTextEditor

Crate: `libero`
Import: `use libero::components::{RichTextEditor, rich_text::Doc};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/rich_text_editor/view.rs>
Index: [index.md](index.md) lists every other page
Description: A rich text field over a `Doc`: headings, lists, quotes, code blocks and marks, with undo, shortcuts and Markdown typing.

A rich text field over a `Doc`: paragraphs, headings, lists, quotes, code
blocks and marks, with undo, the usual shortcuts and Markdown typing
shortcuts. `Doc::to_markdown` and `Doc::from_markdown` convert it; the types
live in `libero::components::rich_text`. A code block shows its source with
fences while the caret is in it.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{RichTextEditor, rich_text::Doc};

#[component]
fn Demo() -> Element {
    let mut doc = use_signal(|| Doc::from_markdown("## Release notes"));

    rsx! {
        RichTextEditor {
            label: "Notes",
            placeholder: "Start writing",
            value: doc(),
            onchange: move |next| doc.set(next),
        }
        pre { {doc.read().to_markdown()} }
    }
}
```

Typing `# ` at the start of a paragraph makes a heading (`## ` to `###### `
for the smaller ones), `- ` or `* ` a bulleted list, `1. ` a numbered list,
`> ` a quote, `---` and a space a rule, and three backticks and a space a code
block. Closing `` `code` ``, `**bold**`, `*italic*` or `~~strike~~` turns the
span into that mark. Code blocks take none of these.

Your own toolbar: `use_rich_text_editor()` gives a handle; pass it as
`handle`, run commands with `run` and read the state reactively.

```rust
use dioxus::prelude::*;
use libero::components::{
    Button, RichTextEditor,
    rich_text::{Builtin, MarkKind, use_rich_text_editor},
};

#[component]
fn Comment() -> Element {
    let editor = use_rich_text_editor();
    rsx! {
        Button {
            aria_pressed: editor.is_active(MarkKind::Bold),
            onclick: move |_| { editor.run(Builtin::Bold); },
            "Bold"
        }
        RichTextEditor { label: "Comment", toolbar: false, handle: editor }
    }
}
```

Your own nodes: register a node type in a `NodeRegistry` for the document,
then draw it with `NodeViews`. A view gets `NodeViewProps { name, attrs,
children }`; a node with content renders `children` exactly once and marks
its own markup around it `contenteditable: "false"`. Pass the registry as
`registry` too, so a debug build catches a misspelled view name.

```rust
use dioxus::prelude::*;
use libero::components::rich_text::{NodeViewProps, NodeViews, RichTextEditor};

#[component]
fn Mention(props: NodeViewProps) -> Element {
    let user = props.attrs.get("user").and_then(|user| user.as_str()).unwrap_or("?");
    rsx! { span { "@{user}" } }
}

#[component]
fn Message() -> Element {
    rsx! {
        RichTextEditor { label: "Message", nodes: NodeViews::new().with("mention", Mention) }
    }
}
```

Built-in blocks draw through `NodeViews` too, under these names. Without a
view they keep the editor's own look.

| Name | `attrs` | `children` |
|---|---|---|
| `paragraph` | - | The text line, a `p` that holds the caret. |
| `heading` | `level` (1 to 6) | The text line, an `h1` to `h6` that holds the caret. |
| `quote` | - | The quoted blocks. |
| `list` | `ordered` (bool), `start` (number) | The items, `li` elements unless `list_item` has a view. |
| `list_item` | - | The item's blocks. |
| `rule` | - | Nothing: the view is a non-editable island. |
| `code_block` | - | Not overridable: it switches between source and `CodeBlock` itself. |

## Accessibility

### Keyboard

`Mod` in a `Keymap` is Cmd on Apple platforms and Ctrl elsewhere; the table
shows Ctrl.

| Key | Action |
|---|---|
| `Ctrl+B` | Bold. |
| `Ctrl+I` | Italic. |
| `Ctrl+U` | Underline. |
| `Ctrl+Shift+S` or `Ctrl+Shift+X` | Strikethrough. |
| `Ctrl+E` | Inline code. |
| `Ctrl+Alt+0` | Paragraph. |
| `Ctrl+Alt+1` | Heading 1. `Ctrl+Alt+2` to `Ctrl+Alt+6` make headings 2 to 6. |
| `Ctrl+Alt+C` | Code block. |
| `Ctrl+Shift+8` | Bulleted list. |
| `Ctrl+Shift+7` | Numbered list. |
| `Ctrl+Shift+B` | Quote. |
| `Ctrl+Shift+Enter` | Horizontal rule. |
| `Tab` or `Shift+Tab` | In a list item: nests it under the item before, or moves it out. Elsewhere Tab moves focus on as usual. |
| `Shift+Enter` | Line break inside the block. |
| `Ctrl+Z` | Undo. |
| `Ctrl+Shift+Z` or `Ctrl+Y` | Redo. |
| `Ctrl+K` | Opens the link dialog: links the selection, or edits or removes the link at the caret. |
| `Ctrl+Shift+U` | Removes the link at the caret. |
| `Ctrl+/` | Lists the editor's shortcuts, from the live keymap. |
| `Alt+F10` | Moves focus to the toolbar; the arrow keys move through it, Escape returns to the text. |

### Libero handles

- The text is a `role="textbox"` with `aria-multiline`, named by `label` and
  described by `description` and `helper`.
- The toolbar is one tab stop with arrow-key movement, named by the
  localization's `rich_text_editor` words. Each mark and block button reports
  `aria-pressed`.
- The toolbar keeps focus and the selection in the text when clicked.
- In a narrow column the toolbar stays one row: the less used buttons move
  into a "More formatting" menu as `menuitemcheckbox` items, so the arrow keys
  only reach what is shown. Bold, italic and the text type menu always stay.
- A shortcut that toggles a mark, list, quote or block type is announced
  through a polite live region ("Bold on", "Heading 2").
- The link dialog focuses its labelled URL field; a refused scheme shows as
  that field's error. Closing it puts the caret back in the text.
- The text type menu is a menu button whose name includes the current type,
  with `menuitemradio` items.
- Every edit goes through the document model, so undo, the `onchange` value
  and the screen stay in step. Input methods (IME) compose natively and are
  taken in when the composition ends.

### You must

- Leave `label` unset only when something else names the editor, such as an
  `aria_label`.
- Document custom chords you bind in `keymap` for your users.
- Make a `NodeViews` atom name its node in text (a mention shows `@name`): it
  is a non-editable island a screen reader reads as is.

### Limits

- On Blitz (native) the document is shown read-only.
- Paste takes plain text only, one block per line.
- Copy and cut write the selection as plain text and as `text/markdown`;
  custom nodes write through their `NodeSpec::markdown` (pass `registry`).
  Cut is one undo step.
- Drag and drop of text and spellcheck replacements are ignored, so
  `spellcheck` is off.

## Props

### `RichTextEditor`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<Doc>` | - | The document. Leave it out and the editor keeps its own. An echo of a document the editor sent is not a reset. |
| `onchange` | `EventHandler<Doc>` | - | Fires with the new document after every edit. Moving the caret does not fire. |
| `validate` | `Validators<Doc>` | - | Rules over the document, shown once the field loses focus or its form is submitted. |
| `name` | `FieldName<Doc>` | - | A path such as `Post::FIELDS.body()` binds the document to the surrounding `Form`'s value when the editor has no `onchange`. |
| `placeholder` | `String` | - | Shown while the document is one empty paragraph. |
| `toolbar` | `bool` | `true` | Shows the formatting toolbar above the text: marks, link, text type menu, lists, quote, code block, undo and redo. A narrow toolbar moves what does not fit into a More menu. |
| `keymap` | `Keymap` | `Keymap::default()` | Which chords run which commands. `Mod` is Cmd on Apple platforms and Ctrl elsewhere. |
| `commands` | `Commands` | `Commands::default()` | What the keymap and the toolbar run. Register your own command under a name and bind a chord to it. |
| `handle` | `RichTextHandle` | - | From `use_rich_text_editor()`: runs commands from your own toolbar (`run`) and reads the state reactively (`is_active`, `block_kind`, `list_kind`, `in_quote`, `can_undo`, `can_redo`). One handle drives one editor. |
| `nodes` | `NodeViews` | `NodeViews::new()` | Your component per custom node name, e.g. a mention. It gets `NodeViewProps { name, attrs, children }` and renders `children` exactly once. Built-ins (`paragraph`, `heading`, `quote`, `list`, `list_item`, `rule`) take a view under their name too; `code_block` stays fixed. |
| `registry` | `NodeRegistry` | - | Your node types. Copy and cut write your nodes through their `to_markdown`, and a debug build warns about a `nodes` name that is not registered, such as a typo. |
| `size` | `Size` | `md` | Padding and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `label` | `Caption` | - | The field's caption, above the toolbar. It names the text. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Formatting rules or limits. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Disables and dims the field and hides the toolbar. |
| `readonly` | `bool` | `false` | Shows the document, focusable but not editable, without the toolbar. |

`RichTextEditor` also takes the `<div>` HTML attributes and, like every
component, the shared props `sx`, `class`, `style`, `states`, and any extra
HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `FieldPart::Label` | `label` | The label above the toolbar. |
| `FieldPart::Required` | `required` | The required asterisk, in the label. |
| `FieldPart::Description` | `description` | The caption between the label and the control. |
| `FieldPart::Frame` | `frame` | The bordered box around the text. |
| `FieldPart::Control` | `control` | The editable text the label names. |
| `FieldPart::Helper` | `helper` | The caption under the control. |
| `FieldPart::Status` | `status` | The validation message. |

## Theme defaults

It shares `FieldDefaults` with every field and starts at `Textarea`'s `size`
and `radius`.

| Field | Type | Description |
|---|---|---|
| `textarea.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `textarea.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

The `--lsx-field-*` variables are [TextField](text_field.md)'s, shared unchanged.

## Data attributes

The same as [TextField](text_field.md). The wrapper's and the frame's
`data-state` carry `size-<size>`, `radius-<size>`, `disabled`, `required`,
`warning` and `error`, and the caption slots carry `data-slot`. The text
carries `data-empty` while the document is one empty paragraph.
