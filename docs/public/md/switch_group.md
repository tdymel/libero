# SwitchGroup

Crate: `libero`
Import: `use libero::components::SwitchGroup;`
Feature: `switch-group` (off by default, turns on `switch`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/check_group.rs>
Index: [index.md](index.md) lists every other page
Description: A group of switches over an enum, any number of them on, each switch its own tab stop, the question as the group's name and an optional cap.

A group of switches over an enum, any number of them on. The group is the
field: it holds the question's label and captions, keeps the value as one
`Vec<T>` in option order and can cap how many are on.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Options, SwitchGroup};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alert {
    Email,
    Push,
    #[option(label = "Text message")]
    Sms,
    Digest,
}

#[component]
fn Demo() -> Element {
    let mut alerts = use_signal(|| vec![Alert::Email, Alert::Digest]);

    rsx! {
        SwitchGroup {
            value: alerts(),
            onchange: move |next| alerts.set(next),
            max_checked: 2,
            label: "Alerts",
        }
    }
}
```

## Props

### `SwitchGroup`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Vec<T>` | - | The switches that are on, in option order. Pair it with `onchange`. Empty turns none on. |
| `onchange` | `EventHandler<Vec<T>>` | - | Called with the whole selection to hold next, in option order whatever the click order. |
| `name` | `FieldName<Vec<T>>` | - | Posts one hidden input per switch that is on, with `Options::value`. A path such as `Settings::FIELDS.alerts()` also binds the group to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Vec<T>>` | - | Rules over the selection, shown once the group loses focus or its form is submitted. |
| `max_checked` | `usize` | - | At most this many on. At the cap the off switches disable, and switching one off frees them again. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single options. Named groups are drawn flat, without headings. |
| `option_label` | `Callback<T, String>` | `T::label()` | Renames an option. Runs during render, so it can read a locale from context. |
| `option_description` | `Callback<T, String>` | - | A line under each option's label. An empty string renders none. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws every option as a bordered surface you can click anywhere. |
| `orientation` | `Orientation` | `vertical` | `horizontal` lays the options out in a row, for two or three short ones. |
| `color` | `ThemeAwareValue` | `primary` | Track color of the switches that are on; `theme.switch.color` when unset. |
| `size` | `Size` | `md` | Size of the tracks, the thumbs and their labels. |
| `label` | `Caption` | - | The question, and the group's name. |
| `description` | `Caption` | - | Between the question and the options. How to choose. |
| `helper` | `Caption` | - | Under the options. What the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Marks the label. Inside a `Form`, a group with nothing on fails the submit. No option is required on its own. |
| `disabled` | `bool` | `false` | Disables every option and dims the group. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the group from the tab order and the post instead. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

`SwitchGroup`:

| Part | `data-slot` | Description |
|---|---|---|
| `SwitchGroupPart::Label` | `label` | The label above the control. |
| `SwitchGroupPart::Required` | `required` | The required asterisk, in the label. |
| `SwitchGroupPart::Description` | `description` | The caption between the label and the control. |
| `SwitchGroupPart::Control` | `control` | The `group` holding the options. |
| `SwitchGroupPart::Track` | `track` | Each option's track. |
| `SwitchGroupPart::Thumb` | `thumb` | Each option's thumb. |
| `SwitchGroupPart::Helper` | `helper` | The caption under the control. |
| `SwitchGroupPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Moves to the next option that is not disabled: every switch is its own tab stop. |
| `Space` | Toggles the focused switch. |
| `Enter` | Outside a `Form`: toggles the focused switch. Inside one: submits the form, as on a native checkbox. |

### Libero handles

- The group is a `group` named by its `label`, so the question is read on
  entering it.
- Its description, helper and status make up the group's `aria-describedby`.
- A required group says "Required" after its label, from `common.required` in
  the localization: `aria-required` is not allowed on a `group`.
- Each switch is a native checkbox announced as a switch, with its own label,
  and its description is read with it.
- At `max_checked`, the off switches are disabled and leave the tab order, and
  the group's `aria-describedby` says the cap, from `common.at_most` in the
  localization. A `readonly` group disables nothing.
- A group with no name logs a warning.

### You must

- Without a visible `label`, spread `"aria-label"`, since the option labels do
  not say what the question is.

### Example

An alerts question, `SwitchGroup { label: "Alerts", max_checked: 2 }`: a screen
reader says "Alerts" on the first switch, and once two are on the rest are read
as disabled.

## Theme defaults

A `SwitchGroup` reads `SwitchDefaults`: `variant`, `size` and `color` are the
ones a single `Switch` uses, so a group and a lone switch look alike.
[Switch](switch.md) lists them.

## Data attributes

The group's wrapper carries `size-*`, plus `disabled`, `required` and the status
token when they apply. The `role="group"` element adds `horizontal` when the
orientation is a row. Each option is a `Switch` and carries its own states.
