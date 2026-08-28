# MultiSelect

Crate: `libero`
Import: `use libero::components::{MultiSelect, Options};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/select/multi_select.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A listbox over an enum that holds any number of its options, drawn as chips in the trigger.

The multi-value sibling of [Select](select.md), on the same engine. The list
stays open on a pick and a pick toggles the row; Escape, clicking elsewhere and
the trigger close it. The selection is drawn in the trigger as `Chip`s, in the
order it was picked.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{MultiSelect, Options};

#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Mushrooms,
    Olives,
}

#[component]
fn Demo() -> Element {
    let mut value = use_signal(|| vec![Topping::Cheese]);

    rsx! {
        MultiSelect {
            label: "Toppings",
            placeholder: "Pick toppings",
            value: value(),
            onchange: move |next| value.set(next),
        }
    }
}
```

`onchange` hands over the whole next selection, not a delta.

## Drawing the selection

`selection` draws one selected value and replaces the default chip entirely -
comma-joined text, a count, or a chip of your own. `option` works as on
`Select`; its `selected` flag is there for a checkmark.

```rust
MultiSelect {
    value: value(),
    onchange: move |next| value.set(next),
    option: move |o: SelectOptionArgs<Topping>| rsx! {
        Text { component: "span", size: "lg", "{o.value.emoji()}" }
        Text { component: "span", sx: sx().flex("1 1 auto"), "{o.value.label()}" }
        if o.selected {
            Text { component: "span", "✓" }
        }
    },
    selection: move |topping: Topping| rsx! {
        Chip { size: "xs", variant: "outlined", "{topping.emoji()} {topping.label()}" }
    },
}
```

The default chips cannot be removed from the trigger: `Chip` has no close
affordance yet, so deselecting means reopening the list. `clearable` empties the
whole selection at once.

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
The listbox carries `aria-multiselectable="true"`, and Enter toggles the
highlighted row without closing the list.

## Props

### `MultiSelect`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, font size and the rows' size. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list. |
| `value` | `Vec<T>` | - | The selection, in pick order; strictly controlled. Empty shows `placeholder`. |
| `onchange` | `EventHandler<Vec<T>>` | - | The whole selection the caller should hold next. |
| `options` | `Vec<T>` | `T::options()` | Narrows or reorders the list. |
| `option` | `Callback<SelectOptionArgs<T>, Element>` | `T::label()` | Draws one row's content. |
| `selection` | `Callback<T, Element>` | `Chip` | Draws one selected value inside the trigger. |
| `placeholder` | `String` | - | Shown while `value` is empty. |
| `clearable` | `bool` | `false` | An x in place of the chevron that empties the selection. |
| `label` | `Caption` | - | The field's caption. Names the trigger through `aria-labelledby`. |
| `description` | `Caption` | - | Between the label and the control. |
| `helper` | `Caption` | - | Under the control. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. |
| `required` | `bool` | `false` | Adds `aria-required` and marks the label. |
| `disabled` | `bool` | `false` | Takes the trigger out of the tab order and dims the field. |

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
| `multi_select.size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `multi_select.radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |

## Data attributes

The wrapper, the frame and the trigger carry the field's `data-state` tokens:
`size-<size>`, `radius-<size>`, `disabled`, `required`, `warning`, `error`.
The trigger adds `multiple` on a `MultiSelect`. The rows are `ComboboxOption`s,
with `active` and `selected`.
