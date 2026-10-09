# HoverCard

Crate: `libero`
Import: `use libero::components::HoverCard;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/hover_card.rs>
Index: [index.md](index.md) lists every other page
Description: An interactive card that opens while its trigger is hovered or focused, a named, dismissible dialog on a paper surface.

A card that opens while its trigger is hovered or focused. It stays open while
the pointer or focus is inside, so its links and buttons work. Unlike
[`Tooltip`](tooltip.md), it is a named dialog. It flips when its side has no
room, and Escape closes it. `sx`, `class` and extra attributes land on the
card, not the trigger.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Flex, HoverCard, Text},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        HoverCard {
            aria_label: "Ada Lovelace",
            content: rsx! {
                Flex {
                    gap: "xs",
                    sx: sx().max_width("16rem"),
                    Text { sx: sx().font_weight("600"), "Ada Lovelace" }
                    Text { size: "sm", "Wrote the first algorithm meant for a machine, in 1843." }
                    Anchor { to: "https://en.wikipedia.org/wiki/Ada_Lovelace", target: "_blank", "Read more" }
                }
            },
            Button { variant: "outlined", "Ada Lovelace" }
        }
    }
}
```

`side` and `align` take the popover's enums, `libero::hooks::{Side, Align}`,
such as `side: Side::Top, align: Align::Center`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `content` | `Element` | required | What the card shows. Links and buttons are fine. |
| `children` | `Element` | required | The trigger. It must hold a link or a button, since focus is the keyboard's only way to open the card. |
| `side` | `Side` | `bottom` | Which side of the trigger the card opens on. It flips when that side has no room. `Start`/`End` are logical: `Start` is the left under `dir="ltr"`, the right under `rtl`. |
| `align` | `Align` | `start` | Where the card lines up along that side. |
| `open_delay` | `u32` | `0` | Milliseconds the pointer must rest on the trigger before the card opens. |
| `close_delay` | `u32` | `150` | Milliseconds the card waits after the pointer leaves. The pointer needs this time to reach the card, so `0` makes it unreachable. While it counts down, the card carries `data-closing`. |
| `open` | `bool` | unset | Forces the card open or closed. Unset, hover and focus decide. A card forced open cannot be dismissed. |
| `radius` | `ThemeAwareValue` | `sm` | The card's corner radius, or any CSS, e.g. `radius: "0"`. |
| `shadow` | `Size` | `md` | The card's elevation. |
| `disabled` | `bool` | `false` | No card. The trigger keeps its wrapper, so enabling or disabling does not remount it or drop its focus. |

Like every component, `HoverCard` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the card.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | On the trigger's last link or button: moves into the card, and past its last link to whatever follows the trigger. |
| `Shift+Tab` | Walks back. |
| `Escape` | Closes the card and returns focus to the trigger if focus was inside. On the web it works wherever focus is. |

### Libero handles

- Focusing the trigger opens the card. A click does not keep it open.
- A card of text taller than the room beside the trigger is a tab stop, so Tab
  enters it and the arrow keys scroll it. A card with a link or button is not:
  the controls take the focus.
- A trigger with nothing focusable and an unnamed card both warn in the
  console.
- On touch, a tap is no hover: a long press opens the card, as a `Tooltip`. It
  closes a moment after the release unless a tap lands in it, and then a tap
  elsewhere closes it. The card has no arrow.

### You must

- Keep it to extras the trigger's own target already offers: a screen reader
  does not announce a hover card, a preview for sighted users. Content a user
  needs goes in a popover ([`use_popover`](popover.md)) that a click opens.
- Put a link or a button in `children`, since focusing the trigger opens the
  card.
- Name the card with `aria_label` or `aria-labelledby` pointing into the
  content.

### Example

A user's name as a link, with a `HoverCard { aria_label: "Ada Lovelace", .. }`
showing an avatar and bio: focusing the link opens the card, and the profile
page the link leads to holds the same facts for a screen reader.

### Limits

- On desktop and mobile, Escape works only while focus is on the trigger or
  in the card, so a card the pointer opened cannot be dismissed from the
  keyboard there (WCAG 1.4.13).
- On the desktop WebView and Android, Tab on the trigger does not move into
  the card, so its links and buttons take a pointer or a tap there.

## Theme defaults

`HoverCardDefaults` on the theme holds `side`, `align`, `open_delay`,
`close_delay`, `radius` and `shadow`. The surface is the theme's `paper`.
