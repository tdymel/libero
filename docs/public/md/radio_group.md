# RadioGroup

Crate: `libero`
Import: `use libero::components::{Radio, RadioGroup};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/radio_group.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A group of radios over an enum, exactly one selected - one tab stop, arrow-key selection, and the question announced as the group's name.

A group of radios over an enum. The group is the field, and that is the point:
a radio on its own cannot be correct.

## Why the group, and not just a Radio

Three things belong to the set rather than to any one option, and each is a bug
if a lone radio tries to own it.

| Concern | Why it is the group's |
|---|---|
| `name` | Native exclusivity comes from a shared `name`. Left to callers, one typo gives two independently checkable radios |
| One tab stop | WAI-ARIA's radiogroup pattern: Tab enters the group, arrows move *and* select, Tab leaves. A lone radio is its own tab stop, which is the wrong pattern |
| Grouping | A radio's label names one option. Only the group can say the question is "Plan", through `role="radiogroup"` and `aria-labelledby` |

The field chrome follows from the same split: one `description`, one `helper`,
one `status` for the question, and a single `value`/`onchange` pair, because
the answer is one value and not N booleans.

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

The options are `T::options()` unless `options` narrows them, so a misspelled
option is a compile error and `onchange` hands back the value itself - the same
contract [NativeSelect](native_select.md) has.

`value: None` selects nothing, which is what an unanswered question looks like.
The group still needs a way in, so the first option holds the tab stop until
something is selected.

## Keyboard

One tab stop for the whole group:

| Key | Effect |
|---|---|
| Tab | Enters at the selected option, or the first one |
| Arrow down / right | Next option, selected as focus lands |
| Arrow up / left | Previous option, selected as focus lands |
| Tab | Leaves the group entirely |

Selection follows focus, as it does natively.

## Accessibility

Give it a `label`: an option's label names one option, and only the group's
label says what the question is. Without a visible one, spread `"aria-label"`.
With neither, it warns in a debug build.

## Standalone `Radio`

Exported for a caller laying a group out by hand - a radio inside a table row,
say. It is a field like [Checkbox](checkbox.md), with `checked` and `onselect`,
and it is the caller's job to pass a shared `name`, manage the tab order and
group it for assistive tech. Prefer `RadioGroup`.

`onselect` reports a pick and never an unpick: a radio is turned off by another
one being turned on.

## Cards

`variant: "card"` draws every option as a bordered surface, and a click
anywhere on it picks the option. Give each one a line of its own with
`option_description` - a card without one is only a border:

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

In a row the cards stretch to one height. The keyboard, the single tab stop and
the aria wiring are the plain group's; the focus ring moves from the circle to
the card. The selected card is marked by its filled circle, not by its border,
which is decoration. On the web a link or button inside a label or description
keeps its own click, and the option is not picked. Natively (Blitz) the whole
card is still one click target, so a link inside it picks the option.

## Props

### `RadioGroup`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<T>` | - | The selected option; strictly controlled. |
| `onchange` | `EventHandler<T>` | - | Called with the option the caller should select next. |
| `options` | `Vec<T>` | `T::options()` | Narrows or reorders the list. A runtime set passes them here. |
| `option_label` | `Callback<T, String>` | `T::label()` | Overrides what the derive named an option. |
| `option_description` | `Callback<T, String>` | - | A line under each option's label; an empty string renders none. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws every option as a bordered surface that is its own hit area. |
| `orientation` | `Orientation` | `vertical` | A row instead of a column. |
| `color` | `ThemeAwareValue` | `primary` | The ring and dot color of the selected option. |
| `size` | `Size` | `md` | Every circle, and the labels beside them. |
| `label` | `Caption` | - | The question. Names the group through `aria-labelledby`. |
| `description` | `Caption` | - | Between the question and the options. |
| `helper` | `Caption` | - | Under the options. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `aria-required` and marks the label. |
| `disabled` | `bool` | `false` | Disables every option and dims the group. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post. |

### `Radio`

| Prop | Type | Default | Description |
|---|---|---|---|
| `checked` | `bool` | - | Strictly controlled - pair it with `onselect`. |
| `onselect` | `EventHandler<()>` | - | Fires when this radio is picked; never to unpick one. |
| `name` | `String` | - | Shared by every radio in one group. `RadioGroup` sets it. |
| `tabindex` | `String` | - | Which radio is the group's tab stop. `RadioGroup` sets it. |
| `aria_label` | `String` | - | Names the radio when it has no `label`. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws the radio as a bordered surface that is its own hit area. |

Both also take the field props - `label`, `description`, `helper`, `status`,
`size`, `disabled`, `required` - and the shared `sx`, `class`, `style`,
`states` and any extra HTML attributes.

## Theme defaults

`RadioDefaults`: `size`, and one circle per size step (`14px` to `24px`),
published as `--lsx-radio-circle-size-*`. The same scale the checkbox box uses,
so a form mixing the two lines up. There is no `radius`: a radio is a circle at
every size, which is what tells it apart from a checkbox at a glance.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-radio-circle-size-<size>` | Circle diameter for that size step. |
| `--lsx-radio-circle` | The picked step, resolved on the control so the circle and dot inherit it. |
| `--lsx-radio-color` | Ring and dot color: the resolved `color` when checked, `grey.5` when not. |
| `--lsx-radio-on` | `0` or `1`, scaling the dot, so only this var changes between states. |

## Data attributes

The group's wrapper carries `size-*`, plus `disabled`, `required` and the status
token when they apply; the `role="radiogroup"` element adds `horizontal` when
the orientation is a row. Each radio's control carries `checked` when it is the
selected one. A card option's wrapper and control both carry `card`.
