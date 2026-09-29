# Transition

Crate: `libero`
Import: `use libero::components::{Transition, TransitionKind};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/transition.rs>
Index: [index.md](index.md) lists every other page
Description: Fades, slides or scales its children in on mount, and out when `open` turns false.

Animates its children in on mount, and out when `open` turns false. Leave
`open` out for a one-time entrance. It renders a wrapper `div`; the motion is a
CSS transition, so a native renderer without transitions swaps instantly.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Flex, Paper, Text, Transition, TransitionKind};

#[component]
fn Demo() -> Element {
    let mut show = use_signal(|| false);

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Button {
                onclick: move |_| show.toggle(),
                "Toggle"
            }
            Transition {
                kind: TransitionKind::FadeUp,
                open: show(),
                Paper { Text { "Hello" } }
            }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `kind` | `TransitionKind` | `Fade` | How the children move in and out. Travel: `Fade`, `FadeUp`, `FadeDown`, `FadeLeft`, `FadeRight`, `SlideUp`, `SlideDown`, `SlideLeft`, `SlideRight`. Grow: `Scale`, `ScaleX`, `ScaleY`, `Pop`, `PopTopLeft`, `PopTopRight`, `PopBottomLeft`, `PopBottomRight`. Lean or turn: `SkewUp`, `SkewDown`, `RotateLeft`, `RotateRight`. All of them also fade. The slides travel the children's own size; left and right are physical directions. |
| `open` | `bool` | `true` | Omitted, the children animate in once on mount and never exit. Passed, they enter and exit as it flips; the first value does not animate. You own this state. |
| `duration` | `u32` | `200` | Animation length in milliseconds. `0` turns the animation off. |
| `children` | `Element` | required | The content that animates. It is unmounted once the exit ends. |

Like every component, `Transition` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- Closed children are hidden from the focus order and screen readers once the
  exit ends, and removed from the DOM.
- Under reduced motion the children switch instantly.

### You must

- Move focus yourself when focused content exits: use
  [`use_focus_return`](use_focus_return.md).
- Do not hide content that must be announced behind an omitted `open` on the
  server: it renders in its from-state until the page hydrates.

## Theme defaults

`TransitionDefaults` on the theme, as `theme.transition`.

| Field | Type | Default | Description |
|---|---|---|---|
| `duration` | `u32` | `200` | Milliseconds the enter and exit take. |
| `easing` | `&'static str` | `"ease"` | The CSS timing function. |
| `distance` | `&'static str` | `"1rem"` | How far the `Fade*`, `Skew*` and `Rotate*` kinds travel. |
| `scale` | `&'static str` | `"0.9"` | The factor `Scale` grows from. |
| `pop_scale` | `&'static str` | `"0.8"` | The factor `Pop` grows from. |
| `rotate` | `&'static str` | `"5deg"` | The angle the `Rotate*` kinds turn from. |
| `skew` | `&'static str` | `"10deg"` | The angle the `Skew*` kinds lean from. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-transition-duration` | Duration of the transition, as a CSS time. |
| `--lsx-transition-easing` | Timing function of the transition. |
| `--lsx-transition-distance` | Travel of the `Fade*`, `Skew*` and `Rotate*` kinds, a CSS length. |
| `--lsx-transition-scale` | Starting factor of `Scale`. |
| `--lsx-transition-pop-scale` | Starting factor of `Pop`. |
| `--lsx-transition-rotate` | Starting angle of the `Rotate*` kinds. |
| `--lsx-transition-skew` | Starting angle of the `Skew*` kinds. |

## Data attributes

On the root's `data-state`, where a caller's `sx`, `class` and `states` land.

| Token | When |
|---|---|
| `open` | The children are shown, or are animating in. |
| `closed` | The children are hidden, or are animating out. |
