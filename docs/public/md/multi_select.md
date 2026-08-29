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
    selection: move |s: SelectionArgs<Topping>| rsx! {
        Chip { size: "xs", variant: "outlined",
            "{s.value.emoji()} {s.value.label()}"
            span { onmousedown: move |event| event.prevent_default(),
                onclick: move |event| event.stop_propagation(),
                ActionIcon {
                    aria_label: "Remove {s.value.label()}",
                    size: "xs",
                    // A `<button>` takes the UA's `buttontext`, not the chip's
                    // colour, so the cross needs this or it stays black.
                    sx: sx().color("inherit"),
                    tabindex: "-1",
                    onclick: move |_| s.remove.call(()),
                    "x"
                }
            }
        }
    },
}
```

The default chips sit one step down the field's own size scale - an `lg`
`MultiSelect` draws `md` chips - and their x is 60% of the chip's height, so the
two scale together. The x tints itself with `currentColor` on hover, which reads
on a chip of any colour. A trigger full of chips is taller than the field's
`min-height`: at `md` the frame measures 38px against the empty control's 36.

The split: **the control owns the keyboard, the slot owns the drawing.** A
default chip carries an x of its own; a custom one draws whatever it likes and
wires `s.remove` into it, and nothing of the component's is added around it. The
two handlers in the snippet are what keep the trigger's focus and stop the click
from opening the list - a custom chip needs both.

`Chip` itself gains no close affordance: the x belongs to whoever owns the
collection, which is `MultiSelect`. `clearable` still empties the whole selection
at once.

## Posting with a form

`name` emits one hidden input carrying every selected option's
`Options::value()`, comma-joined - `"Apple,Pear"` - which is the shape Mantine
sends too. A backend that expects `fruits[]` repeated once per value has to
split it; which of the two libero should send is still open, and is settled with
the rest of form support.

An empty selection posts an empty value, and a disabled field posts nothing.

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

The chips answer the keyboard too, and the trigger keeps the focus for that as
well. ArrowLeft and ArrowRight move a cursor over the chips - from no cursor,
ArrowLeft lands on the last one - and Backspace or Delete removes the chip under
it, or the last chip when there is none. While the cursor is on a chip and the
list is closed, `aria-activedescendant` names that chip; an open list takes the
attribute back, so the two never claim it at once. The remove buttons are
`tabindex="-1"`: the control is one tab stop, and the keys are how a keyboard
removes a chip. Inside the search box Backspace only ever edits the query.

The list is portaled to the document root through `use_popover`, so no
`overflow: hidden` ancestor clips it and it flips above the trigger near the
bottom of the viewport.
The listbox carries `aria-multiselectable="true"`, and Enter toggles the
highlighted row without closing the list.

## Searching

`searchable` puts a search box at the top of the list, exactly as on
[Select](select.md), and `filter` replaces its default case-insensitive
`contains` over `Options::label`. `SelectFilterArgs<T>` carries `value` and
`query`.

```rust
MultiSelect {
    label: "Toppings",
    searchable: true,
    search_placeholder: "Search toppings",
    value: value(),
    onchange: move |next| value.set(next),
}
```

`filter` is called once per option with the query and answers one `bool`, so a
match can test anything the caller knows rather than only the label - here a
note that is never drawn, which is what the demo's `filter` switch turns on:

```rust
MultiSelect {
    searchable: true,
    value: value(),
    onchange: move |next| value.set(next),
    filter: move |f: SelectFilterArgs<Topping>| {
        let query = f.query.to_lowercase();
        f.value.label().to_lowercase().contains(&query)
            || f.value.note().to_lowercase().contains(&query)
    },
}
```

With it on, "earthy" finds Mushrooms and "divides" finds Pineapple - neither
word is on the row.

The one difference from `Select`: **the query survives a pick.** A multi-select
stays open when a row is toggled, so one search can have several of its matches
ticked without retyping it. It is cleared when the list closes.

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
| `selection` | `Callback<SelectionArgs<T>, Element>` | `Chip` with an x | Draws one selected value inside the trigger, the remove control included. `remove` on the args drops that value. |
| `placeholder` | `String` | - | Shown while `value` is empty. |
| `name` | `String` | - | Emits a hidden input of that name carrying every selected option's `Options::value()`, comma-joined. |
| `clearable` | `bool` | `false` | An x in place of the chevron that empties the selection. |
| `searchable` | `bool` | `false` | A search box at the top of the list. |
| `filter` | `Callback<SelectFilterArgs<T>, bool>` | case-insensitive `contains` | Narrows the options while searching. |
| `search_placeholder` | `String` | - | What the search box says while empty. |
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
with `active` and `selected`. Each chip sits in a `data-slot="chip"` wrapper that
carries its id, and the one under the keyboard cursor adds `data-cursor="true"`.
