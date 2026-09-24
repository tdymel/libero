# Autocomplete

Crate: `libero`
Import: `use libero::components::Autocomplete;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/autocomplete.rs>
Index: [index.md](index.md) lists every other page
Description: A text field that offers completions from any `Options` type, while the value stays a `String`.

A [TextField](text_field.md) that offers completions. The value stays a
`String`, and picking a suggestion inserts its label. To choose from a fixed
set, use [Select](select.md).

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

`String` implements `Options`, so a list of strings works as is. `T` is
inferred from `options`.

The component narrows `options` case-insensitively on `Options::label`.
`filter` replaces the test, and `prefiltered` turns filtering off for a list
that is already narrowed:

```rust,ignore
// Prefix instead of contains.
Autocomplete {
    options: cities,
    value: city(),
    oninput: move |next| city.set(next),
    filter: move |f: AutocompleteFilterArgs<String>| {
        f.value.to_lowercase().starts_with(&f.query.to_lowercase())
    },
}

// A list fetched per keystroke.
Autocomplete {
    options: results(),
    prefiltered: true,
    value: query(),
    oninput: move |next| query.set(next),
}
```

`option` draws a row's content. `onpick` fires after `oninput` with the whole
value, which is how you reach the id behind the label:

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

`empty` is shown in place of the list when nothing matches:

```rust,ignore
Autocomplete {
    options: cities,
    value: city(),
    oninput: move |next| city.set(next),
    empty: rsx! { Text { "No city by that name." } },
}
```

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `FieldPart::Label` | `label` | The label above the control. |
| `FieldPart::Required` | `required` | The required asterisk, in the label. |
| `FieldPart::Description` | `description` | The caption between the label and the control. |
| `FieldPart::Frame` | `frame` | The bordered box around the control. |
| `FieldPart::Leading` | `leading` | The slot before the control: an icon, a prefix. |
| `FieldPart::Control` | `control` | The element the label names. |
| `FieldPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `FieldPart::Helper` | `helper` | The caption under the control. |
| `FieldPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Down` | Opens the list. Typing opens it too. |
| `Up`, `Down`, `Home` or `End` | Move the highlight. |
| `Enter` | Picks the highlighted row. |
| `Escape` or `Tab` | Close the list. |

### Libero handles

- Nothing is highlighted until you arrow onto a row, so Enter on text that
  matches nothing still submits the form.

## Props

### `Autocomplete`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size of the field and its rows. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list. |
| `value` | `String` | `""` | The text. Pair it with `oninput`. Picking a suggestion inserts its label. |
| `oninput` | `EventHandler<String>` | - | Fires per keystroke, and again with the label when a suggestion is picked or the field is cleared. |
| `name` | `FieldName<String>` | - | What the field posts as. A path such as `Signup::FIELDS.city()` also binds it to the surrounding `Form`'s value when it has no `oninput`. |
| `validate` | `Validators<String>` | - | Rules over the text, shown once the field loses focus or its form is submitted. |
| `options` | `Vec<T>` | `[]` | The suggestions to offer. `T` is inferred from it. |
| `option` | `Callback<AutocompleteOptionArgs<T>, Element>` | `T::label()` | Draws one row's content. The highlight and click stay the component's. |
| `onpick` | `EventHandler<T>` | - | A suggestion was accepted, with the whole value behind the text. Fires after `oninput`. |
| `filter` | `Callback<AutocompleteFilterArgs<T>, bool>` | `contains` | Narrows `options`. Defaults to a case-insensitive `contains` over the label. |
| `prefiltered` | `bool` | `false` | `options` arrives already narrowed, such as a list fetched per keystroke. Skips filtering, so `filter` never runs. |
| `placeholder` | `String` | - | Shown while the field is empty. |
| `clearable` | `bool` | `false` | Shows an x at the end of the frame that empties the field. |
| `empty` | `Element` | - | Shown in place of the list when nothing matches. Screen readers hear the localization's `combobox.nothing_found` either way, so change that string to match. |
| `leading` | `Element` | - | Inside the frame, before the control, such as a search icon. |
| `trailing` | `Element` | - | Inside the frame, after the control, before the clear x. |
| `describe_leading` | `bool` | `false` | `leading` is text that describes the value, such as a unit, so screen readers read it with the input. |
| `describe_trailing` | `bool` | `false` | The same for `trailing`. |
| `label` | `Caption` | - | The caption above the control, and the field's name. |
| `description` | `Caption` | - | Between the label and the control. What to enter. |
| `helper` | `Caption` | - | Under the control. Formatting rules, or what the entry changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. |
| `disabled` | `bool` | `false` | Takes the input out of the tab order and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead. |

`AutocompleteOptionArgs<T>` carries `value` and `index`.
`AutocompleteFilterArgs<T>` carries `value` and `query`.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. The attributes land on the input.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field, and the list is
`ComboboxDefaults`. `AutocompleteDefaults` holds only the starting `size` and
`radius`.

| Field | Type | Description |
|---|---|---|
| `field.sizes` | `Sizes<FieldSizeLevel>` | The frame's font size, height and padding per size. |
| `combobox.max_dropdown_height` | `String` | How tall the list grows before it scrolls. |
| `popover.gap` | `f64` | Pixels between the field and the list. |
| `autocomplete.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `autocomplete.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

## Data attributes

The wrapper and the frame carry the field's `data-state` tokens `size-<size>`,
`radius-<size>`, `disabled`, `required`, `warning`, `error`. The rows are
`ComboboxOption`s, with `active` but never `selected`.
