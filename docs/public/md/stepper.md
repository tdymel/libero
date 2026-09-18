# Stepper

Crate: `libero`
Import: `use libero::components::{Options, StepState, Stepper};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/stepper>
Index: [index.md](index.md) lists every other page
Description: The stages of a process over an enum, horizontal or vertical, with the current step's content.

The stages of a process, one per variant of an enum, with the current step's
content. `#[derive(Options)]` lists and names the steps. `panel` matches on the
same enum, so a step without a body does not compile. You own `value` and move
it on, usually from a button inside the step.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Options, StepState, Stepper, Text};

#[derive(Clone, PartialEq, Options)]
enum Stage {
    Account,
    #[option(label = "Shipping address")]
    Shipping,
    Review,
}

#[component]
fn Demo() -> Element {
    let mut stage = use_signal(|| Some(Stage::Account));

    rsx! {
        Stepper {
            value: stage(),
            onstepclick: move |s| stage.set(Some(s)),
            state: |s: Stage| (s == Stage::Shipping && !address_valid())
                .then_some(StepState::Error),
            panel: move |s: Stage| match s {
                Stage::Account => rsx! {
                    Button { onclick: move |_| stage.set(Some(Stage::Shipping)), "Continue" }
                },
                Stage::Shipping => rsx! {
                    Button { onclick: move |_| stage.set(Some(Stage::Review)), "Continue" }
                },
                Stage::Review => rsx! {
                    Button { onclick: move |_| stage.set(None), "Place order" }
                },
            },
        }
    }
}
#
# fn address_valid() -> bool { true }
```

## Step states

`StepState` is `Pending`, `Active`, `Completed` or `Error`. Steps before `value`
are completed, `value` is active, the rest are pending. `value: None` shows
every step as completed, with none current. Render your own done screen for
that case.

`state` overrides single steps. Returning `None` keeps the derived state, so
you name only the step that differs. It is the only way to mark a step `Error`.
An error changes the marker and the status text, not which step is current.

## Orientation

`horizontal` draws the markers in a row and shows the current step's content
below. `label_position` puts the labels at the `side` of the markers or
`below` them. `vertical` shows each step's content under the step and collapses
the others. A closed step's content is never mounted, so it keeps no state.

## Accessibility

With `onstepclick`, completed steps and the current one are buttons, and
pending ones too with `allow_next_steps`. Each is a tab stop. Enter and Space
activate. There are no arrow keys.

Name the steps with `aria_label` or `aria_labelledby` on `Stepper`. Both land on
the `<ol>`, not the root, so two steppers on a page are two named lists.

With `OptionLabel::rich`, the name replaces the drawn label for a screen
reader, so it must contain the visible text (WCAG 2.5.3).

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<T>` | required | The current step. `None` means every step is finished. |
| `panel` | `Callback<T, Element>` | - | A step's body. Horizontal shows it below the strip, vertical under its own step. A closed step's content is not mounted. |
| `options` | `Vec<T>` | `T::options()` | The steps to show, in order. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Overrides a step's label. `OptionLabel::rich` draws it as rsx and keeps a text name. |
| `option_description` | `Callback<T, String>` | - | A second line under a step's label. An empty string prints none. |
| `state` | `Callback<T, Option<StepState>>` | derived | Overrides a step's state. `None` keeps the derived one. The only way to mark a step `Error`. |
| `onstepclick` | `EventHandler<T>` | - | Called with the picked step. Without it the steps are plain text with no tab stops. |
| `allow_next_steps` | `bool` | `false` | With `onstepclick`, lets steps not reached yet be picked too. |
| `orientation` | `Orientation` | `horizontal` | `vertical` puts each step's content under the step itself. |
| `label_position` | `StepLabelPosition` | `side` | `side` or `below` the marker. Ignored when vertical. Below 360px wide, `side` draws as `below`. |
| `size` | `Size` | `md` | Marker, type and spacing. |
| `color` | `ThemeAwareValue` | `primary` | The current and completed markers, and the connectors behind them. |

It also takes `sx`, `class`, `states`, and any extra HTML attributes. An `id`
seeds the ids of the steps.

## Theme defaults

`StepperDefaults` on the theme: `size`, `sizes` (marker, font sizes, gaps),
`label_position`, `color`, `pending_color`, `error_color`, `connector_color`,
`description_color`, `line_width` and `content_padding`. The vertical collapse
takes `theme.collapse.duration`. The status words are `StepperLabels`
(`completed`, `error`) in the [localization](localization.md).
