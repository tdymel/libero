# TagsField

Crate: `libero`
Import: `use libero::components::TagsField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/tags_field.rs>
Index: [index.md](index.md) lists every other page
Description: A field whose value is a `Vec<String>` of typed tags, drawn as chips around the input.

A field whose value is a list of typed strings, drawn as chips around the
input. A comma, any other `split_chars` entry, or Enter adds what was typed,
and so does leaving the field. A tag is trimmed first. To pick from a fixed set
instead, use [MultiSelect](multi_select.md), whose value is a `Vec<T>` of your
own type.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{FieldStatus, TagRejectReason, TagRejection, TagsField};

/// The on-screen message for a rejected tag.
fn why(rejection: TagRejection) -> String {
    let tag = rejection.tag;
    match rejection.reason {
        TagRejectReason::Duplicate => format!("\"{tag}\" is already a topic."),
        TagRejectReason::Full => format!("\"{tag}\" was not added: the topic limit is reached."),
        TagRejectReason::NotAllowed => format!("\"{tag}\" is too short: 3 characters at least."),
    }
}

#[component]
fn Demo() -> Element {
    let mut topics = use_signal(Vec::<String>::new);
    let mut refused = use_signal(|| None::<String>);

    rsx! {
        TagsField {
            label: "Topics",
            description: "Comma or Enter adds one.",
            placeholder: "Add a topic",
            max_tags: 5,
            suggestions: vec!["rust".into(), "dioxus".into(), "wasm".into(), "css".into()],
            tag_rules: |tag: String| tag.chars().count() >= 3,
            onreject: move |rejection: TagRejection| refused.set(Some(why(rejection))),
            status: refused().map(FieldStatus::Error).unwrap_or(FieldStatus::Valid),
            value: topics(),
            onchange: move |next| {
                refused.set(None);
                topics.set(next)
            },
        }
    }
}
```

The field tells screen readers why it refused a tag, but shows nothing. The
`status` above shows the reason on screen until the list next changes.

## Props

### `TagsField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding, font size and the chips' size. A chip is one step smaller than the field. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list, independent of `size`. |
| `value` | `Vec<String>` | `[]` | The tags, in order, strictly controlled. Pair it with `onchange`. |
| `onchange` | `EventHandler<Vec<String>>` | - | Called with the whole list the caller should hold next. |
| `suggestions` | `Vec<String>` | - | Adds a dropdown of tags to pick. Tags already held are not offered. |
| `split_chars` | `Vec<String>` | `[","]` | Each one commits the text before it, typed or pasted. |
| `allow_duplicates` | `bool` | `false` | Lets the same tag be added twice. Off, tags are compared trimmed and case-insensitive. |
| `max_tags` | `usize` | - | The most tags the field accepts. A paste fills the room that is left and refuses the rest. |
| `tag_rules` | `Callback<String, bool>` | - | Accepts or refuses one tag before it is added. A refused tag stays in the input, and the field tells screen readers why. |
| `onreject` | `EventHandler<TagRejection>` | - | A tag was rejected: its `tag` and a `TagRejectReason` of `Duplicate`, `Full` (past `max_tags`) or `NotAllowed` (by `tag_rules`). Called after the same edit's `onchange`. The field shows no message, so say why on screen, such as through `status`, and clear it on the next change. |
| `validate` | `Validators<Vec<String>>` | - | Rules over the whole list, shown once the field loses focus or its form is submitted. |
| `name` | `FieldName<Vec<String>>` | - | Posts each tag under this name. A path such as `Article::FIELDS.topics()` also binds the list to the surrounding `Form`'s value when the field has no `onchange`. |
| `placeholder` | `String` | - | Shown while there are no tags. |
| `clearable` | `bool` | `false` | Shows an x at the end of the frame that empties the field. |
| `tag` | `Callback<SelectionArgs<String>, Element>` | `Chip` | Draws one tag, remove control included. `args.remove` removes it. Make that control a `<button>` with `tabindex: "-1"`, and leave it out when `args.disabled` or `args.readonly` is set. |
| `label` | `Caption` | - | The field's caption, above the control. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Formatting rules, or what the entry affects. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Takes the input out of the tab order and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead. |
| `dropdown_parts` | `Parts<DropdownPart>` | - | Styles the portaled `suggestions` dropdown and its inner parts. |

`SelectionArgs<String>` carries `value`, `remove`, `disabled` and `readonly`, as it does for
`MultiSelect` and `FileField`.

`TagsField` also takes the `<input>` HTML attributes and, like every component,
the shared props `sx`, `class`, `style`, `states`, and any extra HTML
attributes. They land on the input.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `TagsFieldPart::Label` | `label` | The label above the control. |
| `TagsFieldPart::Required` | `required` | The required asterisk, in the label. |
| `TagsFieldPart::Description` | `description` | The caption between the label and the control. |
| `TagsFieldPart::Frame` | `frame` | The bordered box around the control. |
| `TagsFieldPart::Control` | `control` | The element the label names. |
| `TagsFieldPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `TagsFieldPart::Tag` | `tag` | One tag's chip, before the draft input. |
| `TagsFieldPart::Helper` | `helper` | The caption under the control. |
| `TagsFieldPart::Status` | `status` | The validation message. |

### Dropdown

The dropdown is portaled out of the field, so its parts take the
`dropdown_parts` prop. They match from the dropdown box at any depth.

| Part | `data-slot` | Description |
|---|---|---|
| `DropdownPart::Panel` | `dropdown` | The dropdown box itself. |
| `DropdownPart::Listbox` | `listbox` | The scrolling list of rows. |
| `DropdownPart::Option` | `option` | A row. |
| `DropdownPart::OptionLabel` | `label` | A row's `span { "data-slot": "label" }`, which ends in an ellipsis. |
| `DropdownPart::Empty` | `nothing-found` | The text shown when the query matches nothing. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Backspace` | On an empty input: removes the last tag. |
| `Left` | On an empty input, or with the caret before the typed text: moves onto the tags and keeps the text. |
| `Left` or `Right` | On the tags: walk them. `Right` past the last returns to the input. |
| `Delete`, `Backspace` or `Enter` | On a tag: removes it. |
| `Down` or `Up` | With `suggestions`: open the list and move the highlight. |
| `Enter` | With `suggestions`: picks the highlighted row. |
| `Escape` | With `suggestions`: closes the list. |

### Libero handles

- The whole field is one tab stop, plus the clear button when `clearable` shows
  it.
- With `suggestions`, Android's Back button closes the list as Escape does,
  rather than the app.
- The clear button is named by the field's `label`, "Clear Fruit", so two clear
  buttons on one form tell apart. Without a `label` it is "Clear" alone.

### You must

- A custom `tag` must make its remove control a button with `tabindex: "-1"`.
  The arrow keys focus it, and without the tabindex each tag adds a tab stop.

## Theme defaults

Most of it is `FieldDefaults`, shared by every field, and the list is
`ComboboxDefaults`. `TagsFieldDefaults` holds only the `size` and `radius` it
starts at.

| Field | Type | Description |
|---|---|---|
| `field.sizes` | `Sizes<FieldSizeLevel>` | The frame's font size, height and padding per size. |
| `combobox.max_dropdown_height` | `String` | How tall the list grows before it scrolls. |
| `tags_field.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `tags_field.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

## Data attributes

The wrapper and the frame carry the field's `data-state` tokens:
`size-<size>`, `radius-<size>`, `disabled`, `required`, `warning`, `error`.
Each tag sits in a `[data-slot='tag']` wrapper, and its x in
`[data-slot='remove']`. The rows are `ComboboxOption`s, marked `active`, never
`selected`.
