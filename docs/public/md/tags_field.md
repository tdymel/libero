# TagsField

Crate: `libero`
Import: `use libero::components::TagsField;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/tags_field.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A field whose value is a `Vec<String>` of free-typed tags, drawn as chips with the editor between them.

A field whose value is a list of free-typed strings, with the five slots every
field shares. A comma - or any `split_chars` entry - and Enter both commit what
was typed; Backspace on an empty input takes the last tag back.

To *choose* out of a fixed set instead, reach for
[MultiSelect](multi_select.md): its value is a `Vec<T>` over a real domain type,
and it cannot be typed into. This one holds `String`s, for the same reason
[Autocomplete](autocomplete.md) does - a free-typed tag is not a domain enum.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::TagsField;

#[component]
fn Demo() -> Element {
    let mut topics = use_signal(Vec::<String>::new);

    rsx! {
        TagsField {
            label: "Topics",
            description: "Comma or Enter adds one.",
            placeholder: "Add a topic",
            max_tags: 5,
            value: topics(),
            onchange: move |next| topics.set(next),
        }
    }
}
```

Strictly controlled: `value` is the list, `onchange` hands back the whole list
the caller should hold next.

## Committing a tag

Every path that can add a tag folds through one merge, so `max_tags` and the
duplicate rule cannot disagree between them:

| Input | Effect |
|---|---|
| any `split_chars` entry | Commits every piece the text splits into, and clears the input |
| a paste | The same path - a paste arrives as one `oninput` with the whole resulting string, so nothing reads the clipboard |
| `Enter` | Commits the typed text, unless a suggestion row is highlighted, then picks it |
| blur | Commits what was typed rather than throwing it away |
| `Backspace`, empty input | Removes the last tag |
| `Backspace`, non-empty | Edits the text. Never removes a tag |
| `ArrowDown` / `ArrowUp` | Moves the suggestion highlight, and opens the list. Dead without `suggestions` |
| `Escape` | Closes the list, keeps the text |

A tag is trimmed before it is added. Duplicates are compared trimmed and
lowercased unless `allow_duplicates` is on. Everything past `max_tags` is
refused **one tag at a time**, so a paste of five into a field with room for two
adds those two rather than being rejected whole.

## A refused tag is silent

`tag_rules` runs over a single tag before it joins the list and answers a plain
`bool`. It shows no message: "you already added that" is heavier as an error
under the field than the mistake it describes. `onrefuse` is there for a caller
who wants to say something anyway - it is the only thing `onchange` cannot
report, and the only callback beside it.

`validate` is the ordinary field line and rules over the whole list.

```rust
TagsField {
    label: "Topics",
    tag_rules: move |tag: String| tag.chars().count() >= 3,
    onrefuse: move |tag: String| refused.set(Some(tag)),
    validate: (|tags: &Vec<String>| !tags.is_empty()).error("Add at least one topic."),
    value: topics(),
    onchange: move |next| topics.set(next),
}
```

## Suggestions

Without `suggestions` the field renders no listbox and no portal at all - it is
a frame with chips and an input. With them it is a combobox: the arrows move a
highlight, Enter picks the highlighted row, and anything already held drops out
of the list. A picked suggestion becomes exactly the tag a typed one does.

```rust
TagsField {
    label: "Topics",
    suggestions: vec!["rust".into(), "dioxus".into(), "wasm".into()],
    value: topics(),
    onchange: move |next| topics.set(next),
}
```

## Inside a Form

The visible input holds the *draft*, not the value, so it carries no `name`. The
list posts through **one hidden input of that name per tag** - the same shape a
native `<select multiple>` sends, and the one [MultiSelect](multi_select.md)
already uses.

```rust
#[derive(Clone, PartialEq, Default, Fields)]
struct Article {
    topics: Vec<String>,
}

let article = use_store(Article::default);

rsx! {
    Form {
        value: article,
        onsubmit: move |event: FormEvent| {
            // One entry per tag, all named "topics" - the draft input has no
            // name, so it contributes none.
            posted.set(event.values().iter().filter(|(name, _)| name == "topics").count());
        },
        // No `onchange`: the path binds the list to the form's own value.
        TagsField { label: "Topics", name: Article::FIELDS.topics() }
        Button { r#type: "submit", "Save" }
    }
}
```

A `FieldName` built from a path also binds the list to the surrounding form's
value, so a bound field needs no `onchange` of its own - the form's value is
then the only place the list lives.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Vec<String>` | `[]` | The tags, in order. Strictly controlled |
| `onchange` | `EventHandler<Vec<String>>` | - | The whole list the caller should hold next |
| `suggestions` | `Vec<String>` | - | Adds a dropdown. Absent means no dropdown at all |
| `split_chars` | `Vec<String>` | `[","]` | Each one commits the text before it |
| `allow_duplicates` | `bool` | `false` | Off, the compare is trimmed and case-insensitive |
| `max_tags` | `usize` | - | Refuses everything past it, one tag at a time |
| `tag_rules` | `Callback<String, bool>` | - | Accepts or refuses one tag. Silent |
| `onrefuse` | `EventHandler<String>` | - | A tag was refused - duplicate, past `max_tags`, or `tag_rules` |
| `validate` | `Validators<Vec<String>>` | - | Rules over the whole list |
| `name` | `FieldName<Vec<String>>` | - | One hidden input per tag; a path also binds |
| `placeholder` | `String` | - | Shown while there are no tags |
| `clearable` | `bool` | `false` | An x that empties the field |
| `tag` | `Callback<SelectionArgs<String>, Element>` | a `Chip` | Draws one tag, remove control included |
| `label` | `Caption` | - | The field's caption, above the control |
| `description` | `Caption` | - | Between the label and the control |
| `helper` | `Caption` | - | Under the control |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error |
| `required` | `bool` | `false` | Adds `aria-required` and marks the label |
| `disabled` | `bool` | `false` | Takes the input out of the tab order and dims the field |

`SelectionArgs<String>` carries `value` and `remove`; it is the same struct
`MultiSelect` and `FileField` hand their selection renderers.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes; the attributes land on the input.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field, and the list is
`ComboboxDefaults`. `TagsFieldDefaults` keeps only which `size` and `radius` the
component starts at. A chip sits one step down the field's scale, which is the
component's own arithmetic rather than a theme key.

| Field | Type | Description |
|---|---|---|
| `field.sizes` | `Sizes<FieldSizeLevel>` | The frame's font size, height and padding per size |
| `combobox.max_dropdown_height` | `String` | How tall the list grows before it scrolls |
| `tags_field.size` | `Size` | Default `size` when the prop is omitted; `md` |
| `tags_field.radius` | `Size` | Default `radius` when the prop is omitted; `sm` |

## Accessibility

The `<input>` is the labelled control, named by a plain `<label for>`, and the
captions reach it through `aria-describedby`. With `suggestions` it is also the
combobox: focus never leaves it, and `aria-activedescendant` moves over the
rows.

Each chip's x is a **real tab stop**, unlike `MultiSelect`'s. There the trigger
owns a chip cursor the arrow keys move; here the arrows belong to the text, and
the platform cannot report a caret - so "ArrowLeft only at caret 0" is not
implementable, and Backspace alone would leave every chip but the last
unreachable from the keyboard. Removing a chip hands focus to the one that took
its place, to the new last chip when the removed one was last, and to the input
when nothing is left.

## Data attributes

The wrapper and the frame carry the field's `data-state` tokens: `size-<size>`,
`radius-<size>`, `disabled`, `required`, `warning`, `error`. Each tag sits in a
`[data-slot='tag']` wrapper, and its x in `[data-slot='remove']`. The rows are
`ComboboxOption`s, with `active` - never `selected`.
