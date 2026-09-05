# HoverCard

Crate: `libero`
Import: `use libero::components::HoverCard;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/hover_card.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An interactive card that opens while its trigger is hovered or focused - a named, dismissible dialog on a paper surface.

A card that opens while its trigger is hovered or focused, and stays open while
the pointer or focus is inside it, so its links and buttons can be used. Unlike
`Tooltip` it is a `role="dialog"` with a name of its own, is portaled so no
`overflow: hidden` ancestor clips it, flips when its side has no room, and
closes on Escape. The surface is the theme's `paper`; `sx`, `class` and spread
attributes land on the card, not the trigger.

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

`side` and `align` take the popover's enums, `libero::hooks::{Side, Align}`:
`side: Side::Top, align: Align::Center`.

## Props

| prop | type | default | |
|---|---|---|---|
| `content` | `Element` | - | What the card shows. Links and buttons are fine. |
| `children` | `Element` | - | The trigger. |
| `side` | `Side` | `Bottom` | Which side of the trigger the card opens on. Flips when there is no room. |
| `align` | `Align` | `Start` | Where the card lines up along that side. |
| `open_delay` | `u32` | `0` | Milliseconds the pointer must rest on the trigger before the card opens. |
| `close_delay` | `u32` | `150` | Milliseconds the card waits after the pointer leaves - also the time the pointer has to cross into the card, so `0` makes it unreachable by pointer. |
| `open` | `Option<bool>` | `None` | Forces the card open or closed. A card forced open cannot be dismissed. |
| `radius` | `Size` | `sm` | Corner radius. |
| `shadow` | `Size` | `md` | Elevation. |
| `disabled` | `bool` | `false` | Renders `children` bare. |

Theme: `theme.hover_card` holds `open_delay`, `close_delay`, `radius` and
`shadow`.

## Accessibility

- Focusing the trigger from the keyboard opens the card; a click does not keep
  it open.
- Tab on the trigger moves into the card, Tab past its last focusable element
  moves on to whatever follows the trigger, Shift+Tab walks back.
- Escape closes it wherever focus is - on the web even when the pointer opened
  it and focus never left a text field - and hands focus back to the trigger
  when focus was inside.

The card is a dialog, so name it: `aria_label`, or `aria-labelledby` pointing
into the content. An unnamed card warns in the console.

## Limits

- Escape from anywhere relies on a document-level key listener, which only the
  web has. On desktop and mobile the card hears Escape only while focus is on
  its trigger or inside it, so a pointer-opened card cannot be dismissed from
  the keyboard there - WCAG 1.4.13 is not met on those targets.
- Touch is sticky: a tap opens it, a tap elsewhere closes it.
- No arrow.
