# Select

Crate: `libero`
Import: `use libero::components::{Options, Select};`
Feature: `select` (on by default, turns on `combobox`, `chip`, `action-icon`, `button`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/select/select.rs>
Index: [index.md](index.md) lists every other page
Description: A listbox over an enum with rows you can draw yourself, in the same field frame as every other input.

A listbox over an enum, in the same frame as every other field. Unlike
[NativeSelect](native_select.md), the rows are libero's own, so `option` can
draw them with anything. In exchange there is no OS picker on phones. Pass
`value` with `onchange`. `clearable` lets the user go back to `None`. Below the
`sm` breakpoint the list opens as a sheet at the foot of the screen, and a touch
leaves the keyboard down until the search box is tapped. For free text, use
`Autocomplete`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Options, Select};
use libero::sx::sx;

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    Mango,
    #[option(label = "Passion fruit")]
    Passion,
}

#[component]
fn Demo() -> Element {
    let mut value = use_signal(|| None::<Fruit>);

    rsx! {
        Select {
            sx: sx().width("280px").max_width("100%"),
            value: value(),
            onchange: move |next| value.set(next),
            placeholder: "Pick a fruit",
            label: "Fruit",
        }
    }
}
```

## Props

### `Select`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size of the field and its rows. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius of the frame and the list. Or any CSS, e.g. `radius: "0"`. |
| `value` | `Option<T>` | - | The selected option. Pair it with `onchange`. `None`, or a value no option matches, shows `placeholder`. A bare `None` names no `T`: write `None::<T>`. |
| `onchange` | `EventHandler<Option<T>>` | - | Called with the option to select next, or `None` from the clear button. |
| `name` | `FieldName<Option<T>>` | - | Posts the selected option's `Options::value()` under this name: for a derived enum the variant's name, never the label. A path such as `Order::FIELDS.plan()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Option<T>>` | - | Rules over the selection, shown once the select loses focus or its form is submitted. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. An enum from another crate takes a `Choices` list built by `choices!`, with `T` a `Choice<Fruit>` that derefs to the `Fruit`. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. A failed fetch is an empty list, so show your own error beside the field. Order is kept, so a group label used again after another group draws its heading again. |
| `option` | `Callback<SelectOptionArgs<T>, Element>` | `T::label()` | Draws one row's content. The highlight, selection and click stay the component's. Hide a decorative glyph such as an emoji with `aria-hidden`, or a screen reader reads it before the label. |
| `selection` | `Callback<T, Element>` | `T::label()` | Draws the selected value inside the trigger. |
| `placeholder` | `String` | - | Shown while no option matches `value`. |
| `clearable` | `bool` | `false` | Shows an x in place of the chevron while something is selected. The only way `onchange` gets `None`. |
| `searchable` | `bool` | `false` | Puts a search box at the top of the list. The query is cleared when the list closes. |
| `open_on_focus` | `bool` | `false` | Opens the list when the field takes focus, by Tab or a press. Focus coming back after a pick or Escape leaves it closed. |
| `filter` | `Callback<SelectFilterArgs<T>, bool>` | `contains` | Narrows the options while searching. Defaults to a case-insensitive `contains` over `Options::label`. It replaces that test, so a search can match a synonym or a code. |
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

### Domain enums

| Method | Type | Description |
|---|---|---|
| `.choices(&X)` | `Mapped<V, Choice<V>, Infallible>` | On a path to `Option<V>`: binds `name` to a domain enum through a `Choices<V>` list, so the form's value holds `V` and the field edits `Choice<V>`. Pass `X.list()` as `options` too. A stored value the list no longer holds shows the placeholder. |

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
| `SelectPart::Value` | `value` | The picked value or the placeholder, in the trigger. |
| `SelectPart::Helper` | `helper` | The caption under the control. |
| `SelectPart::Status` | `status` | The validation message. |

### Dropdown

The dropdown is portaled out of the field, so its parts take the
`dropdown_parts` prop. They match from the dropdown box at any depth.

| Part | `data-slot` | Description |
|---|---|---|
| `DropdownPart::Panel` | `dropdown` | The dropdown box itself. |
| `DropdownPart::SheetHandle` | `sheet-handle` | Below `sm`: the swipe strip atop the sheet. |
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
| `Home` or `End` | Closed: opens the list at the first or last row. Open: jumps to the first or last row. |
| `Up` or `Down` | Open: move the highlight. |
| `PageUp` or `PageDown` | Open: moves the highlight 10 rows, stopping at the first or last. |
| `Enter` or `Space` | Open: picks the highlighted row. |
| `Tab` or `Alt+Up` | Open: pick the highlighted row and close. |
| `Escape` | Open: closes without a pick. |
| `Letter` | Jumps to a matching label. "b", "e", "r" typed quickly finds Berlin, and a lone "b" after a pause cycles the rows starting with it. On a closed trigger this changes the value in place, as on a native `<select>`. |

### Libero handles

- Disabled options are read out but skipped.
- With `searchable` the search box takes over typing and holds the focus while
  the list is open.
- Android's Back button closes the list as Escape does, rather than the app.
- From 200 rows the list draws only the rows in view. The highlighted row stays
  drawn for `aria-activedescendant`, each row carries `aria-posinset` and
  `aria-setsize` over the whole list, and a grouped row is described by its
  heading.
- Each group is a `role="group"` named by its heading.
- The clear button is named by the field's `label`, "Clear Fruit" in the
  example, so two clear buttons on one form tell apart. Without a `label` it is
  "Clear" alone.

### Example

A country field, `Select { label: "Country", .. }`: on the closed field,
typing "ger" picks Germany in place as a native select does, and Enter opens
the list for browsing.

## Theme defaults

Almost everything is `FieldDefaults`, shared by every field, and the list is
`ComboboxDefaults`. `SelectDefaults` holds only the starting `size` and
`radius`.

| Field | Type | Description |
|---|---|---|
| `field.sizes` | `Sizes<FieldSizeLevel>` | The frame's font size, height and padding per size. |
| `combobox.max_dropdown_height` | `String` | How tall the list grows before it scrolls. |
| `popover.gap` | `f64` | Pixels between the trigger and the list. |
| `select.size` | `Size` | Default `size` when the prop is omitted, `md`. |
| `select.radius` | `Size` | Default `radius` when the prop is omitted, `sm`. |

## Data attributes

The wrapper, the frame and the trigger carry the field's `data-state` tokens
`size-<size>`, `radius-<size>`, `disabled`, `required`, `warning`, `error`.
The trigger adds `multiple` on a `MultiSelect`. The rows are `ComboboxOption`s,
with `active` and `selected`.
