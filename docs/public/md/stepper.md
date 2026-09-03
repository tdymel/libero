# Stepper

Crate: `libero`
Import: `use libero::components::{Options, StepState, Stepper};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/stepper>
Index: [index.md](index.md) - every other component's markdown page
Description: The stages of a process over an enum, horizontal or vertical, with the current step's content.

The stages of a process over an enum, with the current one's content. The steps
are the enum's variants - `#[derive(Options)]` lists them in declaration order
and names each one - and `content` is a match over the same type, so a step
without a body is a compile error. `active` is strictly controlled: moving on is
the caller's, usually from a button inside the step.

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
            active: stage(),
            onstepclick: move |s| stage.set(Some(s)),
            state: |s: Stage| (s == Stage::Shipping && !address_valid())
                .then_some(StepState::Error),
            content: move |s: Stage| match s {
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
```

## Where a step's state comes from

`StepState` is `Pending`, `Active`, `Completed` or `Error`.

- Three are derived from position: steps before `active` are completed,
  `active` is current, the rest are pending.
- `active: None` means every step is finished: all show as completed and none is
  current. There is no `Stepper.Completed`; render your own done screen.
- `state` only overrides. Returning `None` keeps the derived state, so you name
  just the step that differs. It is the only way to say `Error`.
- `Error` changes the marker and the status text, never which step is current:
  an errored active step keeps `aria-current` and its content.

## Orientation

- `horizontal` (default): markers in a row joined by connectors, and one content
  region below the strip for the current step. `label_position` puts labels
  `side` (default) or `below` the markers.
- `vertical`: each step's content sits under the step in a `Collapse`, open only
  for the current step. `label_position` is ignored. `content` is called for
  every step here, because a closing step animates out around its content; a
  closed step's content is still never mounted and keeps no state.

## Props

| Prop | Type | Default | What |
|---|---|---|---|
| `active` | `Option<T>` | required | The current step; `None` = all finished |
| `content` | `Callback<T, Element>` | - | A step's body |
| `steps` | `Vec<T>` | `T::options()` | The steps to show, in order |
| `label` | `Callback<T, OptionLabel>` | `T::label()` | Rename a step, or `OptionLabel::rich` to draw it as rsx |
| `description` | `Callback<T, String>` | - | A second line under the label; `""` prints none |
| `state` | `Callback<T, Option<StepState>>` | derived | Override a step's state; the only source of `Error` |
| `onstepclick` | `EventHandler<T>` | - | Absent: the steps are not interactive |
| `allow_next_steps` | `bool` | `false` | With `onstepclick`, pending steps are clickable too |
| `orientation` | `Orientation` | `horizontal` | `vertical` collapses each step's content under it |
| `label_position` | `StepLabelPosition` | `theme.stepper.label_position` (`side`) | `side` or `below`; ignored when vertical |
| `size` | `Size` | `theme.stepper.size` (`md`) | Marker, type and spacing |
| `color` | `ThemeAwareValue` | `theme.stepper.color` (`primary.6`) | Current and completed markers, and connectors behind them |

Plus `class`, `sx`, `states` and any global attribute (`id` seeds the ids below).
The root is a `div` holding the `<ol>` and, horizontally, the content region.

## Accessibility

- An `<ol role="list">`; the explicit role keeps list semantics in Safari with
  VoiceOver.
- The current step's own element - its button, or its label when not clickable -
  has `aria-current="step"`, so tabbing to it announces it.
- A completed or errored step appends its status in words (`theme.stepper.
  completed_label`, `error_label`); the marker's number, check and cross are
  `aria-hidden`.
- With `onstepclick`, completed steps and the current one are native
  `<button>`s, and pending ones too with `allow_next_steps`. Each is a tab stop in
  document order; Enter and Space activate. There are **no arrow keys**: arrows
  belong to composite widgets, and a list of individually tabbable buttons must
  not swallow them.
- Content is `role="region"` with `aria-labelledby` pointing at its step, ids
  `{id}-step-{i}` and `{id}-content-{i}`.
- When `active` moves while focus is inside the step's content (a "Continue"
  button), focus goes to the step that is now current, or back to the closed
  step's header when every step is finished. A non-clickable header takes it
  through `tabindex="-1"`, which is not a tab stop.

## Theme

`theme.stepper`: `size`, `sizes` (marker, font size, description font size, gap
between marker and label, spacing between steps), `label_position`, `color`,
`pending_color`, `error_color`, `connector_color`, `description_color`,
`line_width`, `content_padding`, `completed_label`, `error_label`. The vertical
arm's height animation is `theme.collapse.duration`.
