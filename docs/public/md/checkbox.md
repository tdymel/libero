# Checkbox

Crate: `libero`
Import: `use libero::components::Checkbox;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/checkbox.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A checkbox with its label beside the box, the field slots under both, and an indeterminate state that lives in Rust rather than in the DOM.

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

## Controlled, always

`checked` in, `onchange` out, with no uncontrolled mode. The click is
cancelled and the DOM is re-rendered from Rust, so the box, the `checked` DOM
property, `:checked`, assistive tech and form submission cannot drift apart.
A caller that ignores `onchange` gets a checkbox that does not move - which is
the honest outcome, not a bug.

Passing one half of the pair warns in debug builds: `checked` without
`onchange` can never change, `onchange` without `checked` can never appear
checked.

## Indeterminate

`indeterminate` is the parent of a partly checked group:

```rust
Checkbox {
    label: "All channels",
    checked: mail() && sms(),
    indeterminate: (mail() != sms()).then_some(true),
    onchange: move |next| { mail.set(next); sms.set(next); },
}
```

It outranks `checked` visually - the box draws a dash - and reads as
`aria-checked="mixed"`. Toggling from it gives `true`, the platform
convention.

It is deliberately **not** the native `indeterminate` DOM property. That
property has no HTML attribute, so nothing can write it through a render, and
chasing it from Rust is the same desync the click-cancel above exists to
prevent. The state is ours; ARIA carries it to assistive tech.

## Layout

The control sits in column one, the label in column two, and the description,
helper and status stack under the label rather than under the box. That is
`use_field`'s inline layout, shared by every field with no frame.

## Accessibility

A visually hidden `<input type="checkbox">` owns the name, the state, the focus
ring and the keyboard - Space toggles it, as it would natively. The visible box
is `aria-hidden` decoration that forwards its clicks.

Without a `label`, give it an `aria_label`. The caption slots that are filled
join `aria-describedby`; an error `status` sets `aria-invalid`; `required` sets
`required` and `aria-required` and marks the label with an asterisk.

## Props

### `Checkbox`

| Prop | Type | Default | Description |
|---|---|---|---|
| `color` | `ThemeAwareValue` | `primary` | The box's color when checked. |
| `size` | `Size` | `md` | The size of the box, and of the label and captions beside it. |
| `radius` | `Size` | `sm` | Corner radius of the box, independent of `size`. |
| `checked` | `bool` | - | Strictly controlled - pair it with `onchange`. |
| `indeterminate` | `bool` | `false` | Draws the mixed state and reads as `aria-checked="mixed"`. |
| `onchange` | `EventHandler<bool>` | - | Called with the value `checked` should take next. |
| `label` | `Caption` | - | The caption beside the box. Names the checkbox through a `for`/`id` pair. |
| `description` | `Caption` | - | Under the label: what checking it means. |
| `helper` | `Caption` | - | Under the description, in the label's column. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Sets `required` and `aria-required`, and marks the label. |
| `disabled` | `bool` | `false` | Disables interaction and dims the checkbox. |
| `aria_label` | `String` | - | Names the checkbox when it has no `label`. |

Like every component, it also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - `name` and `value` among them, since
the props extend `input`'s own.

## Theme defaults

`CheckboxDefaults`: `size`, `radius`, and one square per size step (`14px` to
`24px`), published as `--lsx-checkbox-box-size-*`. The label and caption
typography comes from `FieldDefaults`, so a checkbox and a
[TextField](text_field.md) in one form read at the same scale.

## Data attributes

`data-state` on the wrapper carries `size-*`, `radius-*`, `inline`, and
`disabled`, `required` and the status token when they apply. The control
carries those plus `checked` or `mixed`. The caption slots are addressed as
`data-slot="description" | "helper" | "status"`.
