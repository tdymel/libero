# MultiSelect

Crate: `libero`
Import: `use libero::components::{MultiSelect, Options};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/select/multi_select.rs>
Index: [index.md](index.md) lists every other page
Description: A listbox over an enum that holds any number of its options, drawn as chips in the trigger.

A listbox over an enum that holds any number of its options, drawn as chips in
the trigger. A pick toggles the row and the list stays open. Escape, a click
elsewhere or the trigger close it. It works like [Select](select.md) otherwise.

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

`onchange` hands over the whole next selection.

`selection` draws one selected value and replaces the default chip, remove
control included, so a custom chip wires `s.remove` itself. `option` works as on
`Select`, and its `selected` flag is there for a checkmark. The emoji and the
checkmark are decoration, hidden from screen readers so the row reads as its
label alone:

```rust,ignore
MultiSelect {
    value: value(),
    onchange: move |next| value.set(next),
    option: move |o: SelectOptionArgs<Topping>| rsx! {
        Text { component: "span", size: "lg", "aria-hidden": "true", "{o.value.emoji()}" }
        Text { component: "span", sx: sx().flex("1 1 auto"), "{o.value.label()}" }
        if o.selected {
            Text { component: "span", "aria-hidden": "true", "✓" }
        }
    },
    selection: move |s: SelectionArgs<Topping>| rsx! {
        // Unclipped, so the x's 24px hit area reaches past the pill.
        Chip { size: "xs", variant: "outlined", sx: sx().overflow("visible"),
            // After the label, so a long label never clips the x.
            trailing: rsx! {
                // A disabled or read-only field's chip has no remove control.
                if !s.disabled && !s.readonly {
                    span { onmousedown: move |event| event.prevent_default(),
                        onclick: move |event| event.stop_propagation(),
                        ActionIcon {
                            // `words` is `use_localization()`, read in the component.
                            aria_label: fill(words.common.remove, &[("label", &s.value.label())]),
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
            span { "aria-hidden": "true", "{s.value.emoji()} " }
            "{s.value.label()}"
        }
    },
}
```

The two handlers around the `ActionIcon` keep the trigger's focus and stop the
click from opening the list. A custom chip needs both. The default chips sit one
size step below the field's own size.

`name` posts one entry per selected option, each its `Options::value()`, such as
`fruits=Apple&fruits=Pear`. That is what a native `<select multiple>` sends. A
disabled field posts nothing.

Groups and disabled options come through `options`, as an `OptionList<T>`:

```rust,ignore
OptionList::grouped()
    .group("Vegetables", [Topping::Mushrooms, Topping::Olives].map(OptionItem::new))
    .group("Fruit", [OptionItem::new(Topping::Pineapple).disabled(true)])
```

`searchable` puts a search box at the top of the list, and `filter` replaces its
default test. The query survives a pick, so you can tick several matches of one
search. It is cleared when the list closes.

```rust,ignore
MultiSelect {
    searchable: true,
    search_placeholder: "Search toppings",
    value: value(),
    onchange: move |next| value.set(next),
    filter: move |f: SelectFilterArgs<Topping>| {
        let query = f.query.to_lowercase();
        f.value.label().to_lowercase().contains(&query)
            || f.value.note().to_lowercase().contains(&query)
    },
}
```

## Props

### `MultiSelect`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size of the field and its rows. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list. |
| `value` | `Vec<T>` | - | The selection, in the order it was picked. Pair it with `onchange`. Empty shows `placeholder`. |
| `onchange` | `EventHandler<Vec<T>>` | - | Called with the whole next selection. |
| `name` | `FieldName<Vec<T>>` | - | Posts each selected option's `Options::value()` under this name. A path such as `Order::FIELDS.toppings()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Vec<T>>` | - | Rules over the selection, shown once the select loses focus or its form is submitted. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set goes here. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. |
| `option` | `Callback<SelectOptionArgs<T>, Element>` | `T::label()` | Draws one row's content. `selected` on the args is there for a checkmark. Hide a decorative glyph such as an emoji or the checkmark with `aria-hidden`, or a screen reader reads it with the label. |
| `selection` | `Callback<SelectionArgs<T>, Element>` | `Chip` with an x | Draws one selected value in the trigger, remove control included. `remove` on the args drops that value; `disabled` and `readonly` say the field refuses it, so leave the remove control out then. |
| `placeholder` | `String` | - | Shown while `value` is empty. |
| `clearable` | `bool` | `false` | Shows an x in place of the chevron that empties the selection. |
| `searchable` | `bool` | `false` | Puts a search box at the top of the list. The query survives a pick and is cleared when the list closes. |
| `filter` | `Callback<SelectFilterArgs<T>, bool>` | `contains` | Narrows the options while searching. Defaults to a case-insensitive `contains` over `Options::label`. |
| `search_placeholder` | `String` | - | What the empty search box says. |
| `label` | `Caption` | - | The caption above the control, and the select's name. |
| `description` | `Caption` | - | Between the label and the control. What to pick. |
| `helper` | `Caption` | - | Under the control. Constraints, or what the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Takes the trigger out of the tab order and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the select from the tab order and the post instead. |
| `dropdown_parts` | `Parts<DropdownPart>` | - | Styles the portaled dropdown and its inner parts. |

`SelectOptionArgs<T>` carries `value`, `index`, `selected` and `disabled`.
`SelectionArgs<T>` carries `value`, `remove`, `disabled` and `readonly`. `SelectFilterArgs<T>` carries
`value` and `query`.

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. The attributes land on the trigger.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `SelectPart::Label` | `label` | The label above the control. |
| `SelectPart::Required` | `required` | The required asterisk, in the label. |
| `SelectPart::Description` | `description` | The caption between the label and the control. |
| `SelectPart::Frame` | `frame` | The bordered box around the control. |
| `SelectPart::Control` | `control` | The element the label names. |
| `SelectPart::Trailing` | `trailing` | The slot after the control: a chevron, a toggle. |
| `SelectPart::Value` | `value` | The placeholder, in the trigger, while nothing is picked. |
| `SelectPart::Chip` | `chip` | A picked value's chip. |
| `SelectPart::Helper` | `helper` | The caption under the control. |
| `SelectPart::Status` | `status` | The validation message. |

### Dropdown

The dropdown is portaled out of the field, so its parts take the
`dropdown_parts` prop. They match from the dropdown box at any depth.

| Part | `data-slot` | Description |
|---|---|---|
| `DropdownPart::Panel` | `dropdown` | The dropdown box itself. |
| `DropdownPart::Search` | `search` | The search box above the rows. |
| `DropdownPart::Listbox` | `listbox` | The scrolling list of rows. |
| `DropdownPart::Group` | `group` | A group of rows that share a label, from an `OptionList`. |
| `DropdownPart::GroupLabel` | `group-label` | A group's heading. |
| `DropdownPart::Option` | `option` | A row. |
| `DropdownPart::OptionLabel` | `label` | A row's `span { "data-slot": "label" }`, which ends in an ellipsis. |
| `DropdownPart::Empty` | `nothing-found` | The text shown when the query matches nothing. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Down`, `Up`, `Enter` or `Space` | Closed: opens the list. |
| `Home` or `End` | Closed: opens the list at the first or last row. |
| `Up` or `Down` | Open: move the highlight. |
| `PageUp` or `PageDown` | Open: moves the highlight 10 rows, stopping at the first or last. |
| `Enter` or `Space` | Open: toggles the row and keeps the list open. |
| `Escape`, `Tab` or `Alt+Up` | Open: close the list. |
| `Left` or `Right` | Move over the chips. |
| `Backspace` or `Delete` | Removes the chip you are on, or the last one. Inside the search box `Backspace` only edits the query. |
| `Letter` | Jumps to a matching label and opens the list there. Unlike on `Select`, it never changes the value in place, since a pick here toggles. |

### Libero handles

- Disabled options are read out but skipped.
- With `searchable` the search box takes over typing and holds the focus while
  the list is open.
- Android's Back button closes the list as Escape does, rather than the app.
- The clear button is named by the field's `label`, "Clear Fruit", so two clear
  buttons on one form tell apart. Without a `label` it is "Clear" alone.

### Example

A tags field, `MultiSelect { label: "Tags", .. }`: Enter opens the list, Space
ticks rows and keeps it open, and after closing it Left and Right move over
the chips and Delete removes one.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field, and the list is
`ComboboxDefaults`. `theme.multi_select`, a `SelectDefaults`, holds only the
starting `size` and `radius`.

| Field | Type | Description |
|---|---|---|
| `field.sizes` | `Sizes<FieldSizeLevel>` | The frame's font size, height and padding per size. |
| `combobox.max_dropdown_height` | `String` | How tall the list grows before it scrolls. |
| `popover.gap` | `f64` | Pixels between the trigger and the list. |
| `multi_select.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `multi_select.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

## Data attributes

The wrapper, the frame and the trigger carry the field's `data-state` tokens
`size-<size>`, `radius-<size>`, `disabled`, `required`, `warning`, `error`.
The trigger adds `multiple`. The rows are `ComboboxOption`s,
with `active` and `selected`. Each chip sits in a `data-slot="chip"` wrapper that
carries its id, and the one under the keyboard cursor adds `data-cursor="true"`.
