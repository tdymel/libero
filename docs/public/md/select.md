# Select

Crate: `libero`
Import: `use libero::components::{Options, Select};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/select/select.rs>
Index: [index.md](index.md) lists every other page
Description: A listbox over an enum with rows you can draw yourself, in the same field frame as every other input.

A listbox over an enum, in the same frame as every other field. Unlike
[NativeSelect](native_select.md), the rows are libero's own, so `option` can
draw them with anything. In exchange there is no OS picker on phones. Pass
`value` with `onchange`. `clearable` lets the user go back to `None`.

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

`onchange` hands over `Option<T>`, `Some` for a pick and `None` for the clear
button. `value: None` leaves nothing to infer `T` from, so write `None::<Fruit>`
and annotate the handler.

`option` draws a row's content. The highlight, selection and click stay the
component's. `selection` draws the value in the trigger and takes the bare `T`.
The emoji is decoration, hidden from screen readers so the row reads as its
label alone:

```rust,ignore
Select {
    value: value(),
    onchange: move |next| value.set(next),
    option: move |o: SelectOptionArgs<Fruit>| rsx! {
        Text { span { "aria-hidden": "true", "{o.value.emoji()} " } "{o.value.label()}" }
    },
    selection: move |fruit: Fruit| rsx! { span { "aria-hidden": "true", "{fruit.emoji()} " } "{fruit.label()}" },
}
```

`searchable` puts a search box at the top of the list. It narrows
case-insensitively on `Options::label`, and `filter` replaces that test, so a
search can match a synonym or a code:

```rust,ignore
Select {
    searchable: true,
    search_placeholder: "Search fruit",
    value: value(),
    onchange: move |next| value.set(next),
    filter: move |f: SelectFilterArgs<Fruit>| {
        let query = f.query.to_lowercase();
        f.value.label().to_lowercase().contains(&query)
            || f.value.note().to_lowercase().contains(&query)
    },
}
```

The query is cleared when the list closes. To complete free text rather than
choose from a set, use [Autocomplete](autocomplete.md).

`name` posts the selected option's `Options::value()` with a plain `<form>`.
For a derived enum that is the variant's name, never the label. A disabled
select posts nothing.

```rust,ignore
Select { label: "Fruit", name: "fruit", value: fruit(), onchange: move |next| fruit.set(next) }
```

Groups and disabled options come through `options`, as an `OptionList<T>`:

```rust,ignore
OptionList::grouped()
    .group("Orchard", [Fruit::Apple.into(), OptionItem::new(Fruit::Cherry).disabled(true)])
    .group("Tropical", [Fruit::Banana, Fruit::Mango].map(OptionItem::new))
```

Each group is a `role="group"` named by its heading. Your order is kept, so a
group label used again after another group draws its heading again. A disabled
option is read out, and the arrows, typeahead and clicks skip it.

## Props

### `Select`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Height, padding and font size of the field and its rows. |
| `radius` | `Size` | `sm` | Corner radius of the frame and the list. |
| `value` | `Option<T>` | - | The selected option. Pair it with `onchange`. `None` shows `placeholder`. |
| `onchange` | `EventHandler<Option<T>>` | - | Called with the option to select next, or `None` from the clear button. |
| `name` | `FieldName<Option<T>>` | - | Posts the selected option's `Options::value()` under this name. A path such as `Order::FIELDS.plan()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Option<T>>` | - | Rules over the selection, shown once the select loses focus or its form is submitted. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. A failed fetch is an empty list, so show your own error beside the field. |
| `option` | `Callback<SelectOptionArgs<T>, Element>` | `T::label()` | Draws one row's content. The highlight, selection and click stay the component's. Hide a decorative glyph such as an emoji with `aria-hidden`, or a screen reader reads it before the label. |
| `selection` | `Callback<T, Element>` | `T::label()` | Draws the selected value inside the trigger. |
| `placeholder` | `String` | - | Shown while `value` is `None`. |
| `clearable` | `bool` | `false` | Shows an x in place of the chevron while something is selected. The only way `onchange` gets `None`. |
| `searchable` | `bool` | `false` | Puts a search box at the top of the list. The query is cleared when the list closes. |
| `filter` | `Callback<SelectFilterArgs<T>, bool>` | `contains` | Narrows the options while searching. Defaults to a case-insensitive `contains` over `Options::label`. |
| `search_placeholder` | `String` | - | What the empty search box says. |
| `label` | `Caption` | - | The caption above the control, and the select's name. |
| `description` | `Caption` | - | Between the label and the control. What to pick. |
| `helper` | `Caption` | - | Under the control. Constraints, or what the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. |
| `disabled` | `bool` | `false` | Takes the trigger out of the tab order and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the select from the tab order and the post instead. |
| `dropdown_parts` | `Parts<DropdownPart>` | - | Styles the portaled dropdown and its inner parts. |

`SelectOptionArgs<T>` carries `value`, `index`, `selected` and `disabled`.
`SelectFilterArgs<T>` carries `value` and `query`.

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
| `SelectPart::Value` | `value` | The picked value or the placeholder, in the trigger. |
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
| `Home` or `End` | Closed: opens the list at the first or last row. Open: jumps to the first or last row. |
| `Up` or `Down` | Open: move the highlight. |
| `Enter` or `Space` | Open: picks the highlighted row. |
| `Tab` or `Alt+Up` | Open: pick the highlighted row and close. |
| `Escape` | Open: closes without a pick. |
| `Letter` | Jumps to a matching label. "b", "e", "r" typed quickly finds Berlin, and a lone "b" after a pause cycles the rows starting with it. On a closed trigger this changes the value in place, as on a native `<select>`. |

### Libero handles

- Disabled options are read out but skipped.
- With `searchable` the search box takes over typing and holds the focus while
  the list is open.
- Android's Back button closes the list as Escape does, rather than the app.
- The clear button is named by the field's `label`, "Clear Fruit", so two clear
  buttons on one form tell apart. Without a `label` it is "Clear" alone.

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
