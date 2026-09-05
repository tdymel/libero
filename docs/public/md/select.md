# Select

Crate: `libero`
Import: `use libero::components::{Options, Select};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/select/select.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A listbox over an enum with libero's own rows, in the same field frame as every other input.

A listbox over an enum, with the five slots every field shares. Unlike
[NativeSelect](native_select.md) the rows are libero's own, so `option` can draw
them with anything. That costs the OS picker on phones and working without wasm;
reach for `NativeSelect` when those matter.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Options, Select};

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

#[component]
fn Demo() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));

    rsx! {
        Select {
            label: "Fruit",
            placeholder: "Pick a fruit",
            clearable: true,
            value: value(),
            onchange: move |next| value.set(next),
        }
    }
}
```

`onchange` hands over `Option<T>`: `Some` for a pick, `None` for the clear
button. Without `clearable` nothing produces `None`.

## Captions

`label`, `description` and `helper` are `Caption`s, so each takes a string or an
`Element`. They sit where they do on every other field: the label and the
description above the trigger, the helper and the status below it.

```rust,ignore
Select {
    label: "Fruit",
    description: "Delivered with your next box.",
    helper: "You can swap it until Friday.",
    value: value(),
    onchange: move |next| value.set(next),
}
```

## Drawing the rows and the selection

`option` draws a row's *content*. The row around it - its highlight, its
`aria-selected`, its click - stays the component's, so a custom row cannot break
the wiring:

```rust,ignore
Select {
    value: value(),
    onchange: move |next| value.set(next),
    option: move |o: SelectOptionArgs<Fruit>| rsx! {
        Text { "{o.value.emoji()} {o.value.label()}" }
    },
    selection: move |fruit: Fruit| rsx! { "{fruit.emoji()} {fruit.label()}" },
}
```

`selection` takes the bare `T` rather than `SelectOptionArgs`, since an index
means nothing inside the trigger. Drawing the same thing in both places takes two
closures.

## Nothing picked yet

`value: None` leaves `T` with nothing to be inferred from, so write
`None::<Fruit>` and annotate the handler. The same holds for `NativeSelect`.

## Searching

`searchable` puts a search box at the top of the list. The trigger is unchanged -
it still shows the selection - so a closed select looks exactly as it did.

```rust,ignore
Select {
    label: "Fruit",
    searchable: true,
    search_placeholder: "Search fruit",
    value: value(),
    onchange: move |next| value.set(next),
}
```

By default it narrows case-insensitively on `Options::label`. `filter` replaces
that test, so a search can match anything the caller knows - a synonym, a code,
a record's id:

```rust,ignore
Select {
    searchable: true,
    value: value(),
    onchange: move |next| value.set(next),
    filter: move |f: SelectFilterArgs<Fruit>| {
        let query = f.query.to_lowercase();
        f.value.label().to_lowercase().contains(&query)
            || f.value.note().to_lowercase().contains(&query)
    },
}
```

That is the `filter` switch in the demo above: with it on, "thumb" finds Mango
and "counter" finds Banana, through notes that are never drawn on the row.

The box takes focus while the list is open, and the query is cleared when it
closes, so a reopened list always starts unfiltered. A query that matches
nothing leaves the box on screen with an empty list under it - closing it there
would take away the thing you need to edit. On `MultiSelect` the query instead
survives a pick, so several matches of one search can be ticked without
retyping it.

To complete free text rather than choose from a set, reach for
[Autocomplete](autocomplete.md).

## Posting with a form

`name` emits a hidden input carrying the selected option's `Options::value()`,
so the select takes part in a plain `<form>` submit. The trigger is a
`<div role="combobox">` and cannot carry a `name` itself, which is why this is a
prop and not an attribute you can spread.

```rust,ignore
Select { label: "Fruit", name: "fruit", value: fruit(), onchange: move |next| fruit.set(next) }
```

`Options::value()` is the **variant's name** for a derived enum, never the
label - so a `#[option(label = "..")]` or a translated label never changes what
a form sends. For a runtime set like `String` it is the string itself.

A disabled select posts nothing: the hidden input is disabled with the field.

## Accessibility

Enter, Space and ArrowDown open the list on the selected row; the arrows, Home
and End move the highlight; Enter picks; Escape and Tab close. Typeahead -
jumping to a row by its first letter - is not implemented yet.

## Props

### `Select`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, font size and the rows' size. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list. |
| `value` | `Option<T>` | - | The selected option; strictly controlled. `None` shows `placeholder`. |
| `onchange` | `EventHandler<Option<T>>` | - | The option to select next, or `None` from the clear button. |
| `options` | `Vec<T>` | `T::options()` | Narrows or reorders the list. A runtime set passes it here. |
| `option` | `Callback<SelectOptionArgs<T>, Element>` | `T::label()` | Draws one row's content. |
| `selection` | `Callback<T, Element>` | `T::label()` | Draws the selected value inside the trigger. |
| `placeholder` | `String` | - | Shown while `value` is `None`. |
| `name` | `String` | - | Emits a hidden input of that name carrying the selected option's `Options::value()`, so the select posts with a native form. |
| `clearable` | `bool` | `false` | An x in place of the chevron while something is selected. |
| `searchable` | `bool` | `false` | A search box at the top of the list. |
| `filter` | `Callback<SelectFilterArgs<T>, bool>` | case-insensitive `contains` | Narrows the options while searching. |
| `search_placeholder` | `String` | - | What the search box says while empty. |
| `label` | `Caption` | - | The field's caption. Names the trigger through `aria-labelledby`. |
| `description` | `Caption` | - | Between the label and the control. |
| `helper` | `Caption` | - | Under the control. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Adds `aria-required` and marks the label. |
| `disabled` | `bool` | `false` | Takes the trigger out of the tab order and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post. |

`SelectOptionArgs<T>` carries `value`, `index` and `selected`.
`SelectFilterArgs<T>` carries `value` and `query`.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes; the attributes land on the trigger.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field, and the list is
`ComboboxDefaults`. `SelectDefaults` keeps only which `size` and `radius` the
component starts at.

| Field | Type | Description |
|---|---|---|
| `field.sizes` | `Sizes<FieldSizeLevel>` | The frame's font size, height and padding per size. |
| `combobox.max_dropdown_height` | `String` | How tall the list grows before it scrolls. |
| `popover.gap` | `f64` | Pixels between the trigger and the list. |
| `select.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `select.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |

## Data attributes

The wrapper, the frame and the trigger carry the field's `data-state` tokens:
`size-<size>`, `radius-<size>`, `disabled`, `required`, `warning`, `error`.
The trigger adds `multiple` on a `MultiSelect`. The rows are `ComboboxOption`s,
with `active` and `selected`.
