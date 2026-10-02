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
fences while the caret is in it; the language button on its opening fence,
the toolbar's language menu or `Ctrl+Shift+L` change its language.

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
`> ` a quote, `---` and a space a rule, and three backticks and a space (or
Enter) a code block; a name after the backticks, as in ```` ```rust ````, sets
its language. Closing `` `code` ``, `**bold**`, `*italic*` or `~~strike~~` turns the
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

Mentions: type `@` and a name. `intercept` sees each key press (before the
keymap) and typed text first and takes it over by returning `true`, so the
list gets the arrow keys, Enter and Escape while it is open. `overlay` floats
at the caret: under its line, above it near the window's bottom edge. The
handle's `with_state` reads the text before the caret and `edit` swaps the
typed `@name` for a mention node as one undo step. `tools` adds your own
toolbar button, here one that types the `@`. A command can insert a node too:
Mod+Shift+2 runs `mention`, which inserts the registry's default user.

```rust
use dioxus::prelude::*;
use libero::components::rich_text::{
    Attrs, Chord, Commands, EditorInput, EditorState, Keymap, NodeRegistry, NodeSpec,
    NodeViewProps, NodeViews, Position, RichTextTool, Selection, use_rich_text_editor,
};
use libero::components::{Paper, RichTextEditor};
use libero::sx::sx;

#[component]
fn Mention(props: NodeViewProps) -> Element {
    let user = props.attrs.get("user").and_then(|user| user.as_str()).unwrap_or("?");
    rsx! { b { "@{user}" } }
}

fn registry() -> NodeRegistry {
    let mut registry = NodeRegistry::default();
    registry.register(NodeSpec::inline("mention").attr("user", "ada")).unwrap();
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

#[component]
fn MentionEditor() -> Element {
    let editor = use_rich_text_editor();
    let mut active = use_signal(|| 0usize);
    let mut dismissed = use_signal(|| None::<String>);
    let commands = use_hook(|| {
        let mut commands = Commands::default();
        commands.register("at", |state| state.insert_text("@"));
        commands.register("mention", |state| {
            let mention = registry().new_inline("mention", Attrs::new());
            mention.is_some_and(|mention| state.insert_inline(mention))
        });
        commands
    });
    let keymap = use_hook(|| {
        let mut keymap = Keymap::default();
        keymap.bind(Chord::parse("Mod+Shift+2").unwrap(), "mention");
        keymap
    });
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
    });
    rsx! {
        RichTextEditor {
            label: "Message",
            placeholder: "Type @ to mention someone",
            handle: editor,
            commands,
            keymap,
            tools: vec![RichTextTool::new("at", "Mention someone", rsx! { "@" })],
            nodes: NodeViews::new().with("mention", Mention),
            registry: registry(),
            intercept,
            overlay,
            active_descendant: picked.map(|user| format!("mention-{user}")),
            overlay_results: count,
        }
    }
}
```

On Android a soft keyboard reports most keys as `Unidentified` and composes
its text, so `intercept` sees Enter but rarely the typed text; `with_state`
still sees every word once it is composed, which is what the example reads.

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
| `Ctrl+Shift+L` | In a code block: opens its language menu. Choosing or Escape returns to the caret. |
| `Ctrl+Shift+8` | Bulleted list. |
| `Ctrl+Shift+7` | Numbered list. |
| `Ctrl+Shift+B` | Quote. |
| `Ctrl+Shift+Enter` | Horizontal rule. |
| `Ctrl+Enter` | In a code block: leaves it for a new paragraph after it. Elsewhere the key passes on, so your own `Ctrl+Enter` (send) still runs. |
| `ArrowDown` | On the last line of a code block that ends the document: leaves it for a new paragraph. Clicking below the last block does the same. |
| `Enter` | Twice at the end of a code block: the second drops the empty line and leaves the block, also on a touch keyboard. |
| `Tab` or `Shift+Tab` | In a list item: nests it under the item before, or moves it out. Elsewhere Tab moves focus on as usual. |
| `Escape`, then `Tab` | Escape, then Tab or Shift+Tab: leaves the editor, also from a list item. |
| `Shift+Enter` | Line break inside the block. |
| `Ctrl+Backspace` or `Ctrl+Delete` | Deletes the word before or after the caret (`Alt` on a Mac). |
| `Cmd+Backspace` or `Cmd+Delete` | On a Mac: deletes to the start or end of the line, which ends at a line break or the block's edge. |
| `Ctrl+Z` | Undo. |
| `Ctrl+Shift+Z` or `Ctrl+Y` | Redo. |
| `Ctrl+K` | Opens the link dialog: links the selection, or edits or removes the link at the caret. |
| `Ctrl+Shift+U` | Removes the link at the caret. |
| `Ctrl+/` | Lists the editor's shortcuts, from the live keymap. |
| `Alt+F10` | Moves focus to the toolbar; the arrow keys move through it, Escape returns to the text. |

### Libero handles

- The text is a `role="textbox"` with `aria-multiline`, named by `label`
  through `aria-labelledby` and described by `description` and `helper`.
  Clicking the label focuses the text. A debug build warns when nothing names
  it.
- The text stays a tab stop while `readonly`; while `disabled` it carries
  `aria-disabled` and is skipped.
- The text is described by a hint read on focus: in a list Tab indents, and
  Escape, then Tab leaves (`RichTextEditorLabels::leave_hint`).
- The toolbar is one tab stop with arrow-key movement, named by the
  localization's `rich_text_editor` words. Each mark and block button reports
  `aria-pressed`.
- The toolbar keeps focus and the selection in the text when clicked.
- Each toolbar button shows its name and its chord in a tooltip on hover and
  keyboard focus, and carries the chord as `aria-keyshortcuts`.
- In a narrow column the toolbar stays one row: the less used buttons move
  into a "More formatting" menu as `menuitemcheckbox` items, so the arrow keys
  only reach what is shown. Bold, italic and the text type menu always stay.
- A shortcut, toolbar button or menu item that toggles a mark, list, quote or
  block type is announced through a polite live region ("Bold on",
  "Heading 2").
- The link dialog focuses its labelled URL field; a refused scheme shows as
  that field's error. Closing it puts the caret back in the text.
- The text type menu is a menu button whose name includes the current type,
  with `menuitemradio` items. So is the language menu of a code block, in
  the toolbar and on its opening fence, whose name starts with the fence text
  it shows ("rust, Code language"); the fence button is not a tab stop,
  `Ctrl+Shift+L` reaches it.
- Every edit goes through the document model, so undo, the `onchange` value
  and the screen stay in step. Input methods (IME) compose natively and are
  taken in when the composition ends.
- While `overlay` is set, the text carries `aria-controls` naming it,
  `aria-autocomplete="list"` and `aria-activedescendant` from
  `active_descendant`, so a screen reader announces the highlighted option.
- With `overlay_results`, the overlay's option count is announced through the
  polite live region whenever it changes while the overlay shows ("2
  results").

### You must

- Leave `label` unset only when something else names the editor, such as an
  `aria-label` or `aria-labelledby` attribute.
- Document custom chords you bind in `keymap` for your users.
- Bind chords the browser leaves to the page. `Ctrl+N`, `Ctrl+T`, `Ctrl+W`,
  `Ctrl+Tab` and their Shift forms never reach it (on a Mac also `Cmd+Q`,
  `Cmd+H`, `Cmd+M`). A chord you unbind goes back to the browser: without
  Underline, `Ctrl+U` opens the page source. `Ctrl+P`, `Ctrl+S`, `Ctrl+D`,
  `Ctrl+F`, `Ctrl+L` and `Ctrl+Shift+I`/`J`/`C` belong to the browser too. On
  Windows `Ctrl+Alt` is AltGr, which types characters on many layouts: where it
  types one, the character wins over the chord (`Ctrl+Alt+2` types `²` on a
  German layout).
- Make a `NodeViews` atom name its node in text (a mention shows `@name`): it
  is a non-editable island a screen reader reads as is.
- Give an `overlay` list `role="listbox"` with an id per `role="option"`, pass
  the highlighted one as `active_descendant` and the count as
  `overlay_results`, and steer it with the keyboard through `intercept`;
  Escape should close it.
- Name each `RichTextTool` with its `label`: the button shows only its icon.

### Limits

- On Blitz (native) the document is shown read-only.
- Copy and cut write the selection as Markdown, as plain text and as
  `text/markdown`: a quote as `>`, a code block fenced with its language,
  lists, headings, rules, marks and links as written. Custom nodes write
  through their `NodeSpec::markdown` (pass `registry`). Cut is one undo step.
- Paste reads Markdown back into formatted blocks; custom nodes stay text.
  Text from elsewhere is read the same way, one paragraph per line. In a code
  block it is pasted as is.
- Drag and drop of text and replacements (spellcheck, autocorrect, macOS text
  substitutions) are ignored, so `spellcheck` is off.

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
| `intercept` | `Callback<EditorInput, bool>` | - | Sees each key press (`EditorInput::Key`, before the keymap) and typed text (`EditorInput::Text`) first; return `true` to take it over, and the editor does nothing with it. Android soft keyboards report most keys as `Unidentified` and compose their text, which never arrives as `Text`; Enter still arrives as a key. Read typed text through the handle's `with_state` instead. |
| `overlay` | `Element` | - | Floats at the caret while `Some`, such as a mention list: under the caret's line, above it near the window's bottom edge, mirrored in right-to-left text. Focus stays in the text; pressing the overlay does not take it. |
| `active_descendant` | `String` | - | The id of the overlay's highlighted option. While `overlay` is `Some` the text carries it as `aria-activedescendant`, with `aria-controls` naming the overlay and `aria-autocomplete="list"`. |
| `overlay_results` | `usize` | - | How many options the overlay lists. While `overlay` is `Some`, a change is announced through the editor's polite live region ("2 results", or "No results" at 0; `RichTextEditorLabels::results` and `nothing_found`). |
| `tools` | `Vec<RichTextTool>` | `vec![]` | Your toolbar buttons, after the block buttons: `RichTextTool::new(command, label, icon)` runs the command by name; `.active(fn)` makes it a toggle with `aria-pressed`. They never move into the More menu. |
| `size` | `Size` | `md` | Padding and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `label` | `Caption` | - | The field's caption, above the toolbar. It names the text. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Formatting rules or limits. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`. |
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
