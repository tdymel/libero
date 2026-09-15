# Checkbox

Crate: `libero`
Import: `use libero::components::Checkbox;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/checkbox.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A checkbox with its label beside the box, the field slots under both, and an indeterminate state that lives in Rust.

A checkbox. It takes the same slots every field takes - `label`,
`description`, `helper`, `status` - but no frame: the box is the control, so
there is nothing to put a border around.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Checkbox;

#[component]
fn Demo() -> Element {
    let mut accepted = use_signal(|| false);

    rsx! {
        Checkbox {
            label: "Accept the terms",
            description: "The licence and the privacy policy.",
            checked: accepted(),
            onchange: move |next| accepted.set(next),
        }
    }
}
```

## Who owns the checked state

The browser never toggles the input itself - a click on the label is cancelled
and Space is answered on `keydown` - and the DOM is re-rendered from Rust, so
the box, the `checked` DOM property, `:checked`, assistive tech and form
submission cannot drift apart. That does not change.

What changes is who holds the state:

- `checked` given: it wins, and the caller owns it. Pair it with `onchange` or
  the box cannot move.
- A `name` that is a path into the surrounding `Form`'s value: the form owns
  it.
- None of those: the checkbox remembers the user's own activation, so it ticks
  and its `validate` rules judge what was ticked. A required checkbox outside a
  store therefore works, where before it read unticked for ever.

Passing one half of the pair still warns in debug builds: `checked` without
`onchange` can never change, `onchange` without `checked` can never appear
checked.

## Indeterminate

`indeterminate` is the parent of a partly checked group:

```rust,ignore
Checkbox {
    label: "All channels",
    checked: mail() && sms(),
    indeterminate: (mail() != sms()).then_some(true),
    onchange: move |next| { mail.set(next); sms.set(next); },
}
```

It outranks `checked` visually - the box draws a dash - and reads as mixed.
Toggling from it gives `true`, the platform convention.

The state is ours: Rust writes it to the input's `indeterminate` DOM property
after each render, because a browser ignores `aria-checked` on a native
checkbox. Where that property cannot be written (natively, server-side),
`aria-checked="mixed"` carries it. No style reads `:indeterminate`.

## Layout

The control sits in column one, the label in column two, and the description,
helper and status stack under the label rather than under the box. That is
the inline layout every field with no frame shares.

## Card

`variant: "card"` draws the whole field as a bordered surface - Paper's
background, border colour and radius - and a click anywhere on it toggles the
checkbox. Pair it with a `description`; the card is what makes room for one:

```rust,ignore
Checkbox {
    variant: "card",
    label: "Priority support",
    description: "Answers within four hours, around the clock.",
    checked: support(),
    onchange: move |next| support.set(next),
}
```

Nothing else changes: the same hidden input, one tab stop, Space to toggle.
The focus ring moves from the box to the card. The checked state is shown by
the box, not by the card's border, which is decoration. On the web a link or
button inside the label or a caption keeps its own click, and the card does
not toggle. Natively (Blitz) the whole card is still one click target, so a
link inside it toggles the card.

## Accessibility

Space toggles it. Without a `label`, give it an `aria_label`.

## Props

### `Checkbox`

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | The box's color when checked. |
| `size` | `Size` | `md` | The size of the box, and of the label and captions beside it. |
| `radius` | `Size` | `sm` | Corner radius of the box, independent of `size`. |
| `checked` | `bool` | - | Pair it with `onchange`. Left out, the box keeps its own state unless a `name` binds it to the form around it. |
| `indeterminate` | `bool` | `false` | Draws the mixed state and reads as mixed to assistive tech. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. |
| `label` | `Caption` | - | The caption beside the box. Names the checkbox through a `for`/`id` pair. |
| `description` | `Caption` | - | Under the label: what checking it means. |
| `helper` | `Caption` | - | Under the description, in the label's column. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the checkbox. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post. Sets `aria-readonly`, which Chromium does not announce on a checkbox: say it in the label or description where it matters. |
| `aria_label` | `String` | - | Names the checkbox when it has no `label`. |
| `variant` | `ChoiceVariant` | `plain` | `card` draws the checkbox as a bordered surface that is its own hit area. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - `name` and `value` among them, since
the props extend `input`'s own.

## Theme defaults

`CheckboxDefaults`: `variant` (`plain`), `size`, `radius`, and one square per size step (`14px` to
`24px`), published as `--lsx-checkbox-box-size-*`. A card's padding is
`FieldDefaults::card_paddings`, `8px` to `18px`, published as
`--lsx-field-card-padding-*`. The label and caption
typography comes from `FieldDefaults`, so a checkbox and a
[TextField](text_field.md) in one form read at the same scale.

## Data attributes

`data-state` on the wrapper carries `size-*`, `radius-*`, `inline`, `card` for
the card variant, and
`disabled`, `required` and the status token when they apply. The control
carries those plus `checked` or `mixed`. The caption slots are addressed as
`data-slot="description" | "helper" | "status"`.
