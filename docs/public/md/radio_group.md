# RadioGroup

Crate: `libero`
Import: `use libero::components::{Radio, RadioGroup};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/radio_group.rs>
Index: [index.md](index.md) lists every other page
Description: A group of radios over an enum, exactly one selected, with one tab stop, arrow-key selection and the question as the group's name.

A group of radios over an enum, exactly one of them selected. The group is the
field. It holds the question's label and captions, makes the options exclusive
and gives the whole set one tab stop.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Options, RadioGroup};

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
    #[option(label = "Team of 5")]
    Team,
}

#[component]
fn Demo() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));

    rsx! {
        RadioGroup {
            label: "Plan",
            helper: "You can change it later.",
            value: plan(),
            onchange: move |next| plan.set(Some(next)),
        }
    }
}
```

The options are `T::options()` unless `options` narrows them, and `onchange`
hands back the value itself. `value: None` selects nothing. The first option
then holds the tab stop.

`variant: "card"` draws every option as a bordered surface you can click
anywhere. Give each one a line with `option_description`:

```rust,ignore
RadioGroup {
    variant: "card",
    orientation: "horizontal",
    label: "Plan",
    value: plan(),
    onchange: move |next| plan.set(Some(next)),
    option_description: move |plan: Plan| match plan {
        Plan::Free => "Three projects, community support.",
        Plan::Pro => "Unlimited projects, email support.",
        Plan::Team => "Five seats and shared billing.",
    }
    .to_string(),
}
```

## A Radio on its own

Use a `Radio` on its own only to lay a group out by hand. Then the shared
`name`, the tab order and the group's name are yours to set.

## Props

### `RadioGroup`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<T>` | - | The selected option. Pair it with `onchange`. `None` selects nothing, as for an unanswered question. |
| `onchange` | `EventHandler<T>` | - | Called with the option to select next. |
| `name` | `FieldName<Option<T>>` | - | What the group posts as. A path such as `Survey::FIELDS.plan()` also binds it to the surrounding `Form`'s value when it has no `onchange`. |
| `validate` | `Validators<Option<T>>` | - | Rules over the selection, shown once the group loses focus or its form is submitted. |
| `options` | `OptionSource<T>` | `T::options()` | Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single options. Named groups are drawn flat, without headings. |
| `option_label` | `Callback<T, String>` | `T::label()` | Renames an option. Runs during render, so it can read a locale from context. |
| `option_description` | `Callback<T, String>` | - | A line under each option's label. An empty string renders none. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws every option as a bordered surface you can click anywhere. A row of cards stretches them to one height. |
| `orientation` | `Orientation` | `vertical` | `horizontal` lays the options out in a row, for two or three short ones. |
| `color` | `ThemeAwareValue` | `primary` | Ring and dot color of the selected option. |
| `size` | `Size` | `md` | Size of the circles and their labels. |
| `label` | `Caption` | - | The question, and the group's name. |
| `description` | `Caption` | - | Between the question and the options. How to choose. |
| `helper` | `Caption` | - | Under the options. What the choice changes. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` on the group and marks the label. Inside a `Form`, an empty one fails the submit. |
| `disabled` | `bool` | `false` | Disables every option and dims the group. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` drops the group from the tab order and the post instead. Chromium does not announce read-only on a group, so say it in the label or description where it matters. |

### `Radio`

| Prop | Type | Default | Description |
|---|---|---|---|
| `checked` | `bool` | - | Whether it is selected. Pair it with `onselect`. |
| `onselect` | `EventHandler<()>` | - | Fires when this radio is picked. Never when another one is. |
| `name` | `String` | - | Shared by every radio in one group, which makes them exclusive. `RadioGroup` sets it. |
| `tabindex` | `String` | - | Which radio is the group's tab stop. `RadioGroup` sets it. |
| `color` | `ThemeAwareValue` | `primary` | Ring and dot color when selected. |
| `size` | `Size` | `md` | Size of the circle and its label. |
| `label` | `Caption` | - | The text beside the circle, and the radio's name. |
| `description` | `Caption` | - | A second line under the label. |
| `helper` | `Caption` | - | A caption under the radio. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error; an empty one or `None` is `Valid`. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. In a group, set it on `RadioGroup`: a lone `Radio` cannot see its siblings, so a `Form` does not block a submit on it. |
| `disabled` | `bool` | `false` | Cannot be picked, dimmed and out of the tab order. |
| `aria_label` | `String` | - | Names the radio when it has no `label`. |
| `readonly` | `bool` | `false` | Refuses the pick. ARIA has no read-only radio, so only `RadioGroup` can announce it. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws the radio as a bordered surface you can click anywhere. |

`Radio` also takes the field props `label`, `description`, `helper`, `status`,
`size`, `disabled` and `required`. Both take the shared `sx`, `class`, `style`,
`states` and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

`RadioGroup`:

| Part | `data-slot` | Description |
|---|---|---|
| `RadioGroupPart::Label` | `label` | The label above the control. |
| `RadioGroupPart::Required` | `required` | The required asterisk, in the label. |
| `RadioGroupPart::Description` | `description` | The caption between the label and the control. |
| `RadioGroupPart::Control` | `control` | The `radiogroup` holding the options. |
| `RadioGroupPart::Circle` | `circle` | Each option's ring. |
| `RadioGroupPart::Dot` | `dot` | Each option's checked mark. |
| `RadioGroupPart::Helper` | `helper` | The caption under the control. |
| `RadioGroupPart::Status` | `status` | The validation message. |

`Radio`:

| Part | `data-slot` | Description |
|---|---|---|
| `RadioPart::Label` | `label` | The label beside the control. |
| `RadioPart::Required` | `required` | The required asterisk, in the label. |
| `RadioPart::Description` | `description` | The caption between the label and the control. |
| `RadioPart::Control` | `control` | Holds the hidden input and the circle, beside the label. |
| `RadioPart::Circle` | `circle` | The drawn ring. |
| `RadioPart::Dot` | `dot` | The checked mark. |
| `RadioPart::Helper` | `helper` | The caption under the control. |
| `RadioPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Enters the group at the selected option, or the first one that is not disabled. Pressed again, leaves the group. |
| `Down` or `Right` | Moves to the next option and selects it, wrapping at the end. |
| `Up` or `Left` | Moves to the previous option and selects it, wrapping at the start. |

### Libero handles

- The group is a `radiogroup` named by its `label`, so the question is read on
  entering it.
- `required`, an error and read-only are set on the group, and its description,
  helper and status are its description.
- Each option's description is read with its radio.
- The whole group is one tab stop, and the arrows select as they move. Disabled
  options are skipped.
- A group with no name logs a warning.

### You must

- Without a visible `label`, spread `"aria-label"`, since the option labels do
  not say what the question is.

### Example

A plan question, `RadioGroup { label: "Plan" }` with three options: Tab enters
the group on the picked plan, a screen reader says "Plan" on the way in, and
the arrows move and pick at once.

## Theme defaults

`RadioDefaults` holds `variant` (`plain`, for a `Radio` and a `RadioGroup`),
`size`, and one circle per size step (`14px` to `24px`), published as
`--lsx-radio-circle-size-*`. It is the scale the checkbox uses, so a form mixing
the two lines up. A radio is a circle at every size, so it has no `radius`.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-radio-circle-size-<size>` | Circle diameter for that size step. |
| `--lsx-radio-circle` | The picked step, resolved on the control so the circle and dot inherit it. |
| `--lsx-radio-color` | Ring and dot color. The resolved `color` when checked, `muted.6` when not. |
| `--lsx-radio-on` | `0` or `1`, scaling the dot. |

## Data attributes

The group's wrapper carries `size-*`, plus `disabled`, `required` and the status
token when they apply. The `role="radiogroup"` element adds `horizontal` when
the orientation is a row. Each radio's control carries `checked` when it is the
selected one. A card option's wrapper and control both carry `card`.
