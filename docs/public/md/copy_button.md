# CopyButton

Crate: `libero`
Import: `use libero::components::CopyButton;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/copy_button.rs>
Index: [index.md](index.md) lists every other page
Description: An icon button that copies a value to the clipboard and confirms it with a check and a spoken "Copied".

An icon button that copies a value to the clipboard. Once the write landed, the
icon turns into a check and a screen reader hears "Copied". Both reset when the
pointer or the focus leaves. `CodeBlock`'s copy control is one.

Name it after what it copies with `aria_label`, such as "Copy link". For a copy
control of your own, build on `use_clipboard()`: `copy(text)` starts the write,
`copied()` or `failed()` rises once the platform answers, and `reset()` clears
both. Say the result in a status region that is already mounted. On the web the
browser allows the write only over HTTPS or on localhost, inside a user action;
natively it needs the `native` feature.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::CopyButton;

#[component]
fn Demo() -> Element {
    rsx! {
        CopyButton {
            value: "cargo add libero",
            aria_label: "Copy the install command",
            variant: "outlined",
        }
    }
}
```

## Accessibility

### Libero handles

- An always-mounted `role="status"` region says "Copied", or that the copy
  failed, once the platform answers. A second copy announces again.
- The icon and the status reset when the pointer or the focus leaves.
- Unset, `aria_label` is the localization's `copy`. The words are
  `CopyButtonLabels`.

### You must

- Name it after what it copies with `aria_label`, such as "Copy link".
- For a copy control of your own on `use_clipboard()`, say the result in a
  status region that is already mounted.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `String` | - | Required. The text a press writes to the clipboard. |
| `variant` | `Variant` | - | Visual style, as on `ActionIcon`. With `color` also unset, the button draws no chrome of its own. |
| `color` | `ThemeAwareValue` | - | Accent color. A theme color name or any CSS color. |
| `size` | `ThemeAwareValue` | `md` | Button size. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. |
| `aria_label` | `String` | `"Copy"` | The button's name, such as "Copy link". Unset, the localization's. |
| `disabled` | `bool` | `false` | Disables and dims the button. |

Like every component, `CopyButton` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the button.

The words are `CopyButtonLabels` in the [localization](localization.md): `copy`
(the name), `copied` and `copy_failed` (what the status says).

## CSS variables

None of its own; it renders an `ActionIcon`, whose variables apply.

## Data attributes

`ActionIcon`'s, on the button's `data-state`.
