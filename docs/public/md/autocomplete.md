# Autocomplete

Crate: `libero`
Import: `use libero::components::Autocomplete;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/autocomplete.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A text field that offers completions - the value stays a `String`, and the suggestions are drawn from any `Options` type.

A [TextField](text_field.md) that offers completions, with the five slots every
field shares. The value is a `String` at all times: picking a suggestion inserts
its label, it does not make the field hold the suggestion's type. To *choose* out
of a fixed set instead, reach for [Select](select.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Autocomplete;

#[component]
fn Demo() -> Element {
    let mut city = use_signal(String::new);
    let cities = vec!["Amsterdam".to_string(), "Berlin".to_string()];

    rsx! {
        Autocomplete {
            label: "City",
            placeholder: "Start typing",
            options: cities,
            value: city(),
            oninput: move |next| city.set(next),
        }
    }
}
```

`String` implements `Options`, so a list of strings needs no ceremony. The
generic is inferred from `options`, so no call site ever annotates it.

## Filtering

By default the component narrows `options` itself, case-insensitively, on
`Options::label`. Two props change that:

```rust,ignore
// A different test - prefix instead of contains.
Autocomplete {
    options: cities,
    value: city(),
    oninput: move |next| city.set(next),
    filter: move |f: AutocompleteFilterArgs<String>| {
        f.value.to_lowercase().starts_with(&f.query.to_lowercase())
    },
}

// A list fetched per keystroke is already narrowed - don't narrow it twice.
Autocomplete {
    options: results(),
    prefiltered: true,
    value: query(),
    oninput: move |next| query.set(next),
}
```

`prefiltered` skips filtering entirely, so `filter` never runs alongside it.

## Rich rows, and the record behind the text

`option` draws a row's *content*; the row around it - its highlight, its click -
stays the component's. `onpick` fires after `oninput` with the whole value, which
is how a caller reaches the id behind the label:

```rust,ignore
Autocomplete {
    options: airports(),
    value: text(),
    oninput: move |next| text.set(next),
    option: move |o: AutocompleteOptionArgs<Airport>| rsx! {
        Text { "{o.value.code()} - {o.value.city()}" }
    },
    onpick: move |airport: Airport| selected.set(Some(airport.id())),
}
```

`AutocompleteOptionArgs<T>` carries `value` and `index`. There is no `selected`:
a suggestion is not a selection.

## Nothing matches

A list with no rows and no `empty` renders nothing at all, which is usually what
you want while the field is still being typed into. Pass `empty` to say so out
loud:

```rust,ignore
Autocomplete {
    options: cities,
    value: city(),
    oninput: move |next| city.set(next),
    empty: rsx! { Text { "No city by that name." } },
}
```

## Accessibility

Typing opens the list, and so does ArrowDown; the arrows, Home and End move the
highlight; Enter picks the highlighted row; Escape and Tab close. Nothing is
highlighted until the user arrows onto a row, so Enter on text that matches no
suggestion still submits the form.

## Props

### `Autocomplete`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, font size and the rows' size. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list. |
| `value` | `String` | `""` | The text; strictly controlled. |
| `oninput` | `EventHandler<String>` | - | Per keystroke, and again with the label on a pick or a clear. |
| `options` | `Vec<T>` | `[]` | The suggestions to offer. `T` is inferred from it. |
| `option` | `Callback<AutocompleteOptionArgs<T>, Element>` | `T::label()` | Draws one row's content. |
| `onpick` | `EventHandler<T>` | - | A suggestion was accepted, with the whole value. Fires after `oninput`. |
| `filter` | `Callback<AutocompleteFilterArgs<T>, bool>` | case-insensitive `contains` | Narrows `options`. |
| `prefiltered` | `bool` | `false` | `options` is already narrowed. Skips filtering, so `filter` never runs. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `clearable` | `bool` | `false` | An x that empties the field, at the end of the frame. |
| `empty` | `Element` | - | Shown in place of the list when nothing matches. |
| `leading` | `Element` | - | Inside the frame, before the control - a search icon. |
| `trailing` | `Element` | - | Inside the frame, after the control, before the clear x. |
| `describe_leading` | `bool` | `false` | `leading` is text that describes the input, so it joins the input's `aria-describedby`. |
| `describe_trailing` | `bool` | `false` | The same for `trailing`. The clear x stays out of the description. |
| `label` | `Caption` | - | The field's caption, above the control. |
| `description` | `Caption` | - | Between the label and the control. |
| `helper` | `Caption` | - | Under the control. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Adds `aria-required` and marks the label. |
| `disabled` | `bool` | `false` | Takes the input out of the tab order and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post. |

`AutocompleteFilterArgs<T>` carries `value` and `query`.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes; the attributes land on the input.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field, and the list is
`ComboboxDefaults`. `AutocompleteDefaults` keeps only which `size` and `radius`
the component starts at.

| Field | Type | Description |
|---|---|---|
| `field.sizes` | `Sizes<FieldSizeLevel>` | The frame's font size, height and padding per size. |
| `combobox.max_dropdown_height` | `String` | How tall the list grows before it scrolls. |
| `popover.gap` | `f64` | Pixels between the field and the list. |
| `autocomplete.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `autocomplete.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |

## Data attributes

The wrapper and the frame carry the field's `data-state` tokens: `size-<size>`,
`radius-<size>`, `disabled`, `required`, `warning`, `error`. The rows are
`ComboboxOption`s, with `active` - never `selected`.
