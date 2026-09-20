# QrCode

Crate: `libero`
Import: `use libero::components::QrCode;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/qr_code.rs>
Index: [index.md](index.md) lists every other page
Description: Encodes a string as a scalable QR code, rendered as an inline SVG.

Encodes `data` as a QR code in an inline SVG. The colors come from the theme
(`Theme::qr_code`). The SVG has no size of its own and fills its container, so
give it a width. If `data` is too long for the `robustness`, it renders nothing.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::QrCode, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        QrCode {
            data: "https://github.com/tdymel/libero",
            aria_label: "The libero repository on GitHub",
            sx: sx().width("160px"),
        }
    }
}
```

`robustness` takes `low`, `medium`, `quartile` or `high`. A higher level
survives more damage but makes a denser code, so a printed sticker wants a high
level and a screen does not.

## Accessibility

### Libero handles

- The code is one `role="img"` named by `aria_label`. The svg inside is hidden.

### You must

- Say in `aria_label` where the code leads or what it holds, not that it is a
  QR code. A screen reader user cannot scan it, so the label is the only way to
  the payload.
- Next to a real link, the link serves better.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `data` | `String` | required | The payload encoded into the code. |
| `robustness` | `QrRobustness` | `medium` | How much damage the code survives. Higher levels make a denser code. |
| `aria_label` | `String` | required | The code's accessible name. Say where it leads. |

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

The SVG's fills read both, so setting them on an ancestor recolors the code.

## Data attributes

None. The root carries only what you pass in `states`.
