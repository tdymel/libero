# CheckboxGroup

Crate: `libero`
Import: `use libero::components::CheckboxGroup;`
Feature: `checkbox-group` (off by default, turns on `checkbox`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/check_group.rs>
Index: [index.md](index.md) lists every other page
Description: A group of checkboxes over an enum, any number of them checked, each box its own tab stop, the question as the group's name and an optional cap.

A group of checkboxes over an enum, any number of them checked. The group is the
field: it holds the question's label and captions, keeps the value as one
`Vec<T>` in option order and can cap how many are checked.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{CheckboxGroup, Options};

#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Olives,
    Basil,
    #[option(label = "Chili flakes")]
    Chili,
}

#[component]
fn Demo() -> Element {
    let mut toppings = use_signal(|| vec![Topping::Cheese, Topping::Basil]);

    rsx! {
        CheckboxGroup {
            label: "Toppings",
            helper: "More cost extra at the counter.",
            value: toppings(),
            onchange: move |next| toppings.set(next),
        }
    }
}
```

The options are `T::options()` unless `options` narrows them, and `onchange`
hands back the whole selection in option order. `max_checked: 2` caps how many
are checked: at the cap the unchecked options disable.

`variant: "card"` draws every option as a bordered surface you can click
anywhere. Give each one a line with `option_description`:

```rust,ignore
CheckboxGroup {
    variant: "card",
    orientation: "horizontal",
    label: "Toppings",
    max_checked: 2,
    value: toppings(),
    onchange: move |next| toppings.set(next),
    option_description: move |topping: Topping| match topping {
        Topping::Cheese => "Aged, grated over the top.",
        Topping::Olives => "Black, pitted.",
        Topping::Basil => "Fresh leaves, added last.",
        Topping::Chili => "Hot, on the side.",
    }
    .to_string(),
}
```

## Props

### `CheckboxGroup`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Vec<T>` | - | The checked options, in option order. Pair it with `onchange`. Empty checks nothing. |
| `onchange` | `EventHandler<Vec<T>>` | - | Called with the whole selection to hold next, in option order whatever the click order. |
| `name` | `FieldName<Vec<T>>` | - | Posts one hidden input per checked option, with `Options::value`. A path such as `Order::FIELDS.toppings()` also binds the group to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Vec<T>>` | - | Rules over the selection, shown once the group loses focus or its form is submitted. |
| `max_checked` | `usize` | - | At most this many checked. At the cap the unchecked options disable, and unchecking one frees them again. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single options. Named groups are drawn flat, without headings. |
| `option_label` | `Callback<T, String>` | `T::label()` | Renames an option. Runs during render, so it can read a locale from context. |
| `option_description` | `Callback<T, String>` | - | A line under each option's label. An empty string renders none. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws every option as a bordered surface you can click anywhere. |
| `orientation` | `Orientation` | `vertical` | `horizontal` lays the options out in a row, for two or three short ones. |
| `color` | `ThemeAwareValue` | `primary` | Box color of the checked options; `theme.checkbox.color` when unset. |
| `size` | `Size` | `md` | Size of the boxes and their labels. |
| `label` | `Caption` | - | The question, and the group's name. |
| `description` | `Caption` | - | Between the question and the options. How to choose. |
| `helper` | `Caption` | - | Under the options. What the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Marks the label. Inside a `Form`, a group with nothing checked fails the submit. No option is required on its own. |
| `disabled` | `bool` | `false` | Disables every option and dims the group. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the group from the tab order and the post instead. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

`CheckboxGroup`:

| Part | `data-slot` | Description |
|---|---|---|
| `CheckboxGroupPart::Label` | `label` | The label above the control. |
| `CheckboxGroupPart::Required` | `required` | The required asterisk, in the label. |
| `CheckboxGroupPart::Description` | `description` | The caption between the label and the control. |
| `CheckboxGroupPart::Control` | `control` | The `group` holding the options. |
| `CheckboxGroupPart::Box` | `box` | Each option's box. |
| `CheckboxGroupPart::Helper` | `helper` | The caption under the control. |
| `CheckboxGroupPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Moves to the next option that is not disabled: every box is its own tab stop. |
| `Space` | Checks or unchecks the focused option. |

### Libero handles

- The group is a `group` named by its `label`, so the question is read on
  entering it.
- Its description, helper and status make up the group's `aria-describedby`.
- Each option is a native checkbox with its own label, and its description is
  read with it.
- At `max_checked`, the unchecked options are disabled and leave the tab order.
- A group with no name logs a warning.

### You must

- Without a visible `label`, spread `"aria-label"`, since the option labels do
  not say what the question is.
- With `max_checked`, say the cap in the `label` or `description`: a disabled
  option does not say why.

### Example

A toppings question, `CheckboxGroup { label: "Toppings", max_checked: 2 }`: a
screen reader says "Toppings" on the first box, and once two are checked the
rest read as disabled.

## Theme defaults

A `CheckboxGroup` reads `CheckboxDefaults`: `variant`, `size` and `color` are the
ones a single `Checkbox` uses, so a group and a lone box look alike.
[Checkbox](checkbox.md) lists them.

## Data attributes

The group's wrapper carries `size-*`, plus `disabled`, `required` and the status
token when they apply. The `role="group"` element adds `horizontal` when the
orientation is a row. Each option is a `Checkbox` and carries its own states.
