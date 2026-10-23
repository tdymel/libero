# MultiSelect

Crate: `libero`
Import: `use libero::components::{MultiSelect, Options};`
Feature: `select` (on by default, turns on `combobox`, `chip`, `action-icon`, `button`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/select/multi_select.rs>
Index: [index.md](index.md) lists every other page
Description: A listbox over an enum that holds any number of its options, drawn as chips in the trigger.

A listbox over an enum that holds any number of its options, drawn as chips in
the trigger. A pick toggles the row and the list stays open. Escape, a click
elsewhere or the trigger close it. Below the `sm` breakpoint the list opens as a
sheet at the foot of the screen, closed by its Done button. It works like
[Select](select.md) otherwise.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{MultiSelect, Options};
use libero::sx::sx;

#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Mushrooms,
    Olives,
    Onions,
    Peppers,
    Pineapple,
}

#[component]
fn Demo() -> Element {
    let mut value = use_signal(|| vec![Topping::Cheese, Topping::Olives]);

    rsx! {
        MultiSelect {
            sx: sx().width("280px").max_width("100%"),
            value: value(),
            onchange: move |next| value.set(next),
            placeholder: "Pick toppings",
            label: "Toppings",
        }
    }
}
```

## Props

### `MultiSelect`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size of the field and its rows. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius of the frame and the list. Or any CSS, e.g. `radius: "0"`. |
| `value` | `Vec<T>` | - | The selection, in the order it was picked. Pair it with `onchange`. Empty shows `placeholder`. |
| `onchange` | `EventHandler<Vec<T>>` | - | Called with the whole next selection. |
| `name` | `FieldName<Vec<T>>` | - | Posts each selected option's `Options::value()` under this name, as a native `<select multiple>` does. A path such as `Order::FIELDS.toppings()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Vec<T>>` | - | Rules over the selection, shown once the select loses focus or its form is submitted. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set goes here. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. |
| `option` | `Callback<SelectOptionArgs<T>, Element>` | `T::label()` | Draws one row's content. Unset, a selected row ends in a check mark; a custom row draws its own from `selected` on the args. Hide a decorative glyph such as an emoji or the checkmark with `aria-hidden`, or a screen reader reads it with the label. |
| `selection` | `Callback<SelectionArgs<T>, Element>` | `Chip` with an x | Draws one selected value in the trigger, remove control included. `remove` on the args drops that value; `disabled` and `readonly` say the field refuses it, so leave the remove control out then. A custom remove control stops `mousedown` from taking focus and `click` from opening the list. |
| `placeholder` | `String` | - | Shown while `value` is empty. |
| `clearable` | `bool` | `false` | Shows an x in place of the chevron that empties the selection. |
| `searchable` | `bool` | `false` | Puts a search box at the top of the list. The query survives a pick and is cleared when the list closes. |
| `open_on_focus` | `bool` | `false` | Opens the list when the field takes focus, by Tab or a press. Focus coming back after a pick or Escape leaves it closed. |
| `filter` | `Callback<SelectFilterArgs<T>, bool>` | `contains` | Narrows the options while searching. Defaults to a case-insensitive `contains` over `Options::label`. |
| `search_placeholder` | `String` | - | What the empty search box says. |
| `label` | `Caption` | - | The caption above the control, and the select's name. |
| `description` | `Caption` | - | Between the label and the control. What to pick. |
| `helper` | `Caption` | - | Under the control. Constraints, or what the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label; a field without a visible `label` shows no mark. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Takes the trigger out of the tab order, dims the field and posts nothing. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the select from the tab order and the post instead. |
| `dropdown_parts` | `Parts<DropdownPart>` | - | Styles the portaled dropdown and its inner parts. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. The attributes land on the trigger.

### `SelectOptionArgs`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | - | The option this row draws. |
| `index` | `usize` | - | The row's position among the rows drawn. |
| `selected` | `bool` | - | Part of the selection, for a checkmark. |
| `disabled` | `bool` | - | The list refuses this row. The greying and `aria-disabled` are drawn anyway. |

### `SelectFilterArgs`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | - | The option under test. |
| `query` | `String` | - | What is typed in the search box. |

### `SelectionArgs`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `T` | - | The value this chip draws. |
| `remove` | `Callback<()>` | - | Drops this value, the same edit as picking its row again. |
| `disabled` | `bool` | - | The field is disabled: `remove` does nothing, so draw no remove control. |
| `readonly` | `bool` | - | The field is read-only: `remove` does nothing, so draw no remove control. |

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
| `Left` or `Right` | Move the focus over the chips, from the trigger to the last one and back past it. |
| `Backspace` or `Delete` | Removes the focused chip and moves the focus on, or the last chip from the trigger. Inside the search box `Backspace` only edits the query. |
| `Letter` | Jumps to a matching label and opens the list there. Unlike on `Select`, it never changes the value in place, since a pick here toggles. |

### Libero handles

- Disabled options are read out but skipped.
- With `searchable` the search box takes over typing and holds the focus while
  the list is open.
- Android's Back button closes the list as Escape does, rather than the app.
- Each chip is a group named by its label, and the focus lands on its remove
  button, "Remove Cherry".
- From 200 rows the list draws only the rows in view. The highlighted row stays
  drawn for `aria-activedescendant`, each row carries `aria-posinset` and
  `aria-setsize` over the whole list, and a grouped row is described by its
  heading.
- The clear button is named by the field's `label`, "Clear Toppings" in the
  example, so two clear buttons on one form tell apart. Without a `label` it is
  "Clear" alone.

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
with `active` and `selected`. Each chip sits in a `data-slot="chip"` wrapper, a
group that carries its id, and the one holding the focus adds `data-cursor="true"`.
