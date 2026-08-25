# QrCode

Crate: `libero`
Import: `use libero::components::QrCode;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/qr_code.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Encodes a string as a scalable QR code, rendered as an inline SVG.

Encodes `data` as a scalable QR code, rendered as an inline SVG.
Background/foreground colors come from the theme (`Theme::qr_code`), not
per-instance props - `robustness` is the only thing you tune per code: higher
levels tolerate more damage or occlusion at the cost of a denser code, and unset
takes `Theme::qr_code.robustness` (Medium).

The SVG has no fixed width or height, just a square `viewBox`, so it fills its
container - which is why every example below carries an `sx` width.
`aria_label` is required, not optional: a QR code carries real information to a
sighted or scanning user and none at all to a screen reader without one.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::QrCode, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        QrCode {
            data: "https://github.com/tdymel/libero",
            aria_label: "QR code linking to the libero GitHub repository",
            sx: sx().width("160px"),
        }
    }
}
```

`robustness` takes `low`, `medium`, `quartile` or `high`. Raising it costs
density: the same payload becomes a finer grid, so a code that has to survive a
printed sticker wants a higher level and a code on a screen does not.

```rust
use dioxus::prelude::*;
use libero::{components::QrCode, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        QrCode {
            data: "https://github.com/tdymel/libero",
            aria_label: "QR code linking to the libero GitHub repository",
            robustness: "high",
            sx: sx().width("160px"),
        }
    }
}
```

If `data` is too long for the chosen `robustness` to encode, the component
renders nothing rather than a broken code.

## Accessibility

The root is `role="img"` with the `aria_label` you pass as its accessible name,
so the code is announced as a single image instead of an unlabelled SVG blob.
Say where the code leads or what it contains, not that it is a QR code - a
screen reader user cannot scan it, so the label is the only route to the
payload. Where the code is decorative next to a real link, the link is the
better answer.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `data` | `String` | required | The payload encoded into the code. |
| `robustness` | `QrRobustness` | `medium` | How much damage or occlusion the code tolerates, at the cost of density. |
| `aria_label` | `String` | required | Required: a QR code says nothing to a screen reader without one. |

Like every component, `QrCode` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`QrCodeDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `background` | `&'static str` | The quiet-zone and light-module color. |
| `foreground` | `&'static str` | The dark-module color. |
| `robustness` | `QrRobustness` | Default `robustness` when the prop is omitted. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-qrcode-background` | Light-module and margin color. |
| `--lsx-qrcode-foreground` | Dark-module color. |

Both are baked into the generated SVG's fills, so overriding them on an ancestor
recolors the code without regenerating it.

## Data attributes

None. The root carries no state tokens of its own - only whatever you pass in
`states`.
