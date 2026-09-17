# Marquee

Crate: `libero`
Import: `use libero::components::Marquee;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/marquee.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Content that scrolls on its own in an endless loop, measured by nothing, with a pause toggle and a reduced-motion fallback.

Content that scrolls on its own in an endless loop - a logo strip, a ticker. The
children are rendered `repeat` times in a row, and every copy after the first is
`aria-hidden` and `inert`, so it is read and tabbed through once. `duration` is
one full cycle, so adding an item makes the whole strip move faster.

The pause toggle is there for WCAG 2.2.2, which asks for a way to stop any
motion that runs longer than five seconds. Its name stays "Pause" and
`aria-pressed` says whether it is paused. Turn it off with `pause_control:
false` only when the page offers its own control through `paused` and
`onpausechange`.

Under `prefers-reduced-motion: reduce` it does not move at all: it shows one
copy in a strip the reader scrolls themselves, without the fade or the toggle.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Marquee};

#[component]
fn Demo() -> Element {
    rsx! {
        Marquee {
            Chip { "Rust" }
            Chip { "Dioxus" }
            Chip { "WebAssembly" }
            Chip { "Blitz" }
            Chip { "Tokio" }
            Chip { "Serde" }
        }
    }
}
```

A vertical marquee needs a height, or it is as tall as all its copies:

```rust
use dioxus::prelude::*;
use libero::{components::{Chip, Marquee}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Marquee { orientation: "vertical", sx: sx().height("160px"),
            Chip { "Rust" }
            Chip { "Dioxus" }
        }
    }
}
```

The page's own control, with the built-in toggle turned off:

```rust
use dioxus::prelude::*;
use libero::components::{Button, Chip, Marquee};

#[component]
fn Demo() -> Element {
    let mut paused = use_signal(|| false);
    rsx! {
        Button { selected: paused(), onclick: move |_| paused.toggle(), "Pause the ticker" }
        Marquee {
            pause_control: false,
            paused: paused(),
            onpausechange: move |next| paused.set(next),
            Chip { "Rust" }
            Chip { "Dioxus" }
        }
    }
}
```

## Accessibility

- **The pause toggle** is the WCAG 2.2.2 mechanism, a tab stop that `Enter` or
  `Space` toggles. `pause_on_hover` is not one, since neither a keyboard nor a
  touch screen can hover. It floats over the strip, so while focus is
  on the content it turns transparent rather than cover a focused link.
- **`paused` is strictly controlled when set.** The toggle then only reports
  through `onpausechange`; without the handler it does nothing.
- **Interactive children work only in the first copy.** The other copies are
  `inert`, so a link or button in them does not react to the pointer either.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `children` | `Element` | required | What scrolls. Rendered once per copy; interactive children work only in the first copy. |
| `orientation` | `Orientation` | `horizontal` | The axis it scrolls along. A vertical marquee needs a height from `sx`, or it is as tall as all its copies. |
| `reverse` | `bool` | `false` | Scrolls towards the end instead of the start. |
| `duration` | `u32` | `40000` | Milliseconds per full cycle. A duration, not a speed: the same number moves a longer strip faster. |
| `gap` | `Size` | `md` | Between copies, and between the last and the first. |
| `repeat` | `u8` | `4` | Copies laid in a row. Raise it when a gap crosses the view. Anything below 2 renders 2. |
| `pause_on_hover` | `bool` | `false` | Pointer hover pauses. Not a way to stop it on its own: a keyboard or a touch screen cannot hover. |
| `paused` | `Option<bool>` | `None` | Strictly controlled when set - pair it with `onpausechange`. `None` leaves the state to the built-in toggle. |
| `onpausechange` | `EventHandler<bool>` | - | The built-in toggle was pressed, with the state it asks for. |
| `pause_control` | `bool` | `true` | Renders the pause toggle. Turn it off only when the page offers its own control, through `paused`. |
| `fade_edges` | `bool` | `false` | Fades both ends into the surface color, `--lsx-paper-background`. Only right on a surface of that color. |

Like every component, `Marquee` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`MarqueeDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `duration` | `u32` | Milliseconds per cycle - `40_000`. |
| `repeat` | `u8` | Copies - `4`. |
| `gap` | `Size` | Gap step - `Md`. |
| `pause_on_hover` | `bool` | `false`. |
| `pause_control` | `bool` | `true`, for WCAG 2.2.2. |
| `fade_edges` | `bool` | `false`: the fade is only right on a `Paper`-coloured surface. |
| `fade_size` | `&'static str` | How far the fade reaches in - `5%`. |

The toggle's accessible name, `"Pause"`, is `MarqueeLabels::pause` in the
[localization](localization.md).

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-marquee-duration` | One cycle, from the theme, on `:root`. |
| `--lsx-marquee-duration-override` | Set by the `duration` prop; wins over the theme's. |
| `--lsx-marquee-fade-size` | The fade's reach, on `:root`. |
| `--lsx-marquee-repeat` | The copy count in effect, per instance. |
| `--lsx-marquee-gap` | The gap in effect, per instance. |
| `--lsx-marquee-shift` | Where a cycle ends: one copy plus one gap along the axis. |

The scroll is `@keyframes lsx-marquee`, appended to the theme stylesheet.

## Data attributes

State tokens on the root's `data-state`.

| Token | Condition |
|---|---|
| `horizontal` / `vertical` | The `orientation` in effect. |
| `reverse` | `reverse` is on. |
| `paused` | Paused, by the toggle or by `paused`. |
| `pause-on-hover` | `pause_on_hover` is on. |
| `fade-edges` | `fade_edges` is on. |

The parts are marked with `data-slot`: `track` (the moving row), `group` (one
copy) and `pause` (the toggle).
