# Select

Crate: `libero`
Import: `use libero::components::{Option as SelectOption, Select};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/select>
Index: [index.md](index.md) - every other component's markdown page
Description: A styled native `<select>` with its own label, strictly controlled by `value` plus `onchange`.

A styled native select, always wrapped in its own `label` element - `label`
fills in its text.
Strictly controlled: `value` drives it, `onchange` reports what the user picked.

`Option` is exported under the name `Option`, which collides with Rust's own, so
import it aliased - `Option as SelectOption` - the way the examples below do.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Option as SelectOption, Select};

#[component]
fn Demo() -> Element {
    let mut value = use_signal(|| "sm".to_string());

    rsx! {
        Select {
            label: "Size",
            value: value(),
            onchange: move |v| value.set(v),
            SelectOption { value: "xs", "Extra small" }
            SelectOption { value: "sm", "Small" }
            SelectOption { value: "md", "Medium" }
            SelectOption { value: "lg", "Large" }
            SelectOption { value: "xl", "Extra large" }
        }
    }
}
```

`size` and `radius` step independently, and `disabled` dims the whole control:

```rust
use dioxus::prelude::*;
use libero::components::{Option as SelectOption, Select};

#[component]
fn Demo() -> Element {
    let mut value = use_signal(|| "sm".to_string());

    rsx! {
        Select {
            size: "lg",
            radius: "xl",
            disabled: true,
            value: value(),
            onchange: move |v| value.set(v),
            SelectOption { value: "sm", "Small" }
            SelectOption { value: "lg", "Large" }
        }
    }
}
```

## Accessibility

The root is always a `<label>` wrapping the `<select>`, so a click anywhere on the
control focuses it and the label text - the `<span>` `label` renders - names the
select without an `id`/`for` pair. Leave `label` unset only when something else
already names the select; a bare `<select>` with no accessible name is a defect.
`disabled` sets the native `disabled` attribute, so the browser handles the
focus and interaction semantics.

`value` is deliberately not emitted on the creating render: the `<option>`s mount
in the same pass, so there would be nothing to select yet and the browser would
silently take the first option. It appears on the first update instead.

## Props

### `Select`

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Controls height, padding, and font size. |
| `radius` | `Size` | `sm` | Corner radius, independent of `size`. |
| `value` | `String` | - | The selected option's value; strictly controlled. |
| `onchange` | `EventHandler<String>` | - | Called with the newly picked option's value. |
| `disabled` | `bool` | `false` | Disables interaction and dims the select. |
| `label` | `String` | - | Label text. The wrapper is a `<label>` either way; unset just leaves it wordless. |
| `label_sx` | `Sx` | - | Styles the label alone - the rest of `sx` lands on the wrapper. |
| `children` | `Element` | required | The `Option` elements to list. |

### `Option`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `String` | required | The option's value, reported to `onchange`. |
| `children` | `Element` | required | The option's visible label. |

`Option` also takes `option`'s own HTML attributes.

Like every component, both also take the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`SelectDefaults` on the theme; per-size values live in its `sizes` scale.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |
| `sizes` | `Sizes<SelectSizeLevel>` | `font_size`, `height`, `padding_x` per size. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-select-font-size-<size>` | `font-size` for that size step. |
| `--lsx-select-height-<size>` | `height` for that size step. |
| `--lsx-select-padding-x-<size>` | Horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale rather than one of its own.

## Data attributes

State tokens on the `<select>`'s `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `radius-<size>` | The `radius` in effect. |
| `disabled` | `disabled` is set. |
