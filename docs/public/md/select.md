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

## Drawing the rows and the selection

`option` draws a row's *content*. The row around it - its highlight, its
`aria-selected`, its click - stays the component's, so a custom row cannot break
the wiring:

```rust
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

## Accessibility

The trigger is a focusable element with `role="combobox"`, `aria-haspopup="listbox"`,
`aria-expanded` and `aria-controls`; while the list is open,
`aria-activedescendant` names the highlighted row, so focus never leaves the
trigger. The label names it through `aria-labelledby` - `<label for>` cannot name
an element that is not a form control. Description, helper and status join
`aria-describedby` exactly as on every other field.

Keys: Enter, Space and ArrowDown open the list on the selected row; the arrows,
Home and End move the highlight; Enter picks; Escape and Tab close. Clicking
elsewhere closes it too - the trigger closes on blur, and the list cancels
`mousedown` so a click on a row never takes focus away first.

Rows carry `aria-selected`. Typeahead - jumping to a row by its first letter - is
not implemented yet.

The list is portaled to the document root through `use_popover`, so no
`overflow: hidden` ancestor clips it and it flips above the trigger near the
bottom of the viewport.

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
| `clearable` | `bool` | `false` | An x in place of the chevron while something is selected. |
| `label` | `Caption` | - | The field's caption. Names the trigger through `aria-labelledby`. |
| `description` | `Caption` | - | Between the label and the control. |
| `helper` | `Caption` | - | Under the control. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Adds `aria-required` and marks the label. |
| `disabled` | `bool` | `false` | Takes the trigger out of the tab order and dims the field. |

`SelectOptionArgs<T>` carries `value`, `index` and `selected`.

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
