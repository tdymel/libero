# Marquee

Crate: `libero`
Import: `use libero::components::Marquee;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/marquee.rs>
Index: [index.md](index.md) lists every other page
Description: Content that scrolls on its own in an endless loop, with a pause toggle and a still fallback under reduced motion.

Content that scrolls on its own in an endless loop, such as a logo strip or a
ticker. The children render `repeat` times in a row. `duration` is one full
cycle, so adding an item makes the strip move faster.

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

A vertical marquee needs a height, or it is as tall as all its copies.

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

The page's own control, with the built-in toggle turned off.

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

## Reduced motion

Under `prefers-reduced-motion: reduce` it does not move. It shows one copy in a
strip the reader scrolls, without the fade or the toggle.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `parts` | `Parts<MarqueePart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |
| `children` | `Element` | required | What scrolls, rendered once per copy. Interactive children work only in the first copy, and an `id` repeats in every copy. |
| `orientation` | `Orientation` | `horizontal` | The axis it scrolls along. A vertical marquee needs a height from `sx`, or it is as tall as all its copies. |
| `reverse` | `bool` | `false` | Scrolls towards the end instead of the start. |
| `duration` | `u32` | `40000` | Milliseconds per full cycle. The same number moves a longer strip faster. |
| `gap` | `ThemeAwareValue` | `md` | Between copies, and between the last and the first, or any CSS, e.g. `gap: "0"`. |
| `repeat` | `u8` | `4` | Copies in a row. Raise it when a gap crosses the view; a debug build warns when the copies do not fill the box. Anything below 2 renders 2. |
| `pause_on_hover` | `bool` | `false` | Pauses under the pointer. Not enough on its own, since a keyboard or a touch screen cannot hover. |
| `paused` | `Option<bool>` | `None` | Controlled when set, so pair it with `onpausechange`. `None` leaves the state to the built-in toggle. |
| `onpausechange` | `EventHandler<bool>` | `None` | The built-in toggle was pressed, with the state it asks for. |
| `pause_control` | `bool` | `true` | Renders the pause toggle. Turn it off only when the page offers its own control through `paused`. |
| `fade_edges` | `bool` | `false` | Fades both ends into the surface color, `--lsx-paper-background`. Only right on a surface of that color. |

Like every component, `Marquee` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `MarqueePart::Track` | `track` | The moving row of copies. |
| `MarqueePart::Group` | `group` | One copy of the children. |
| `MarqueePart::Pause` | `pause` | The pause toggle. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Enter` or `Space` | On the pause toggle, a tab stop: pauses or resumes the motion. |

### Libero handles

- Every copy after the first is `aria-hidden` and `inert`, so the content is
  read and tabbed through once.
- The pause toggle meets WCAG 2.2.2, which asks for a way to stop motion that
  runs longer than five seconds. It is named "Pause", and `aria-pressed` says
  whether it is paused.
- While focus is on the content the toggle turns transparent, so it never
  covers a focused link.

### You must

- Put interactive children in the content knowing they work only in the first
  copy.
- Give the children no `id`: they render once per copy, so an `id` would repeat
  and `label for`, `aria-labelledby` or `#id` links would find only the first.
- Don't rely on `pause_on_hover` alone: a keyboard or a touch screen cannot
  hover.
- Turn the toggle off only when the page offers its own control through
  `paused` and `onpausechange`. `paused` is controlled when set: the toggle then
  only reports through `onpausechange`, and without the handler it does
  nothing.

### Example

A logo strip in a `Marquee`: a screen reader reads the logos once, and a
keyboard user Tabs to the toggle named "Pause" and stops the motion with
Enter.

## Theme defaults

`MarqueeDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `duration` | `u32` | Milliseconds per cycle, `40_000`. |
| `repeat` | `u8` | Copies, `4`. |
| `gap` | `Size` | Gap step, `Md`. |
| `pause_on_hover` | `bool` | `false`. |
| `pause_control` | `bool` | `true`, for WCAG 2.2.2. |
| `fade_edges` | `bool` | `false`. The fade is only right on a `Paper`-coloured surface. |
| `fade_size` | `&'static str` | How far the fade reaches in, `5%`. |

The toggle's accessible name, `"Pause"`, is `MarqueeLabels::pause` in the
[localization](localization.md).

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-marquee-duration` | One cycle, from the theme, on `:root`. |
| `--lsx-marquee-duration-override` | Set by the `duration` prop. Wins over the theme's. |
| `--lsx-marquee-fade-size` | The fade's reach, on `:root`. |
| `--lsx-marquee-repeat` | The copy count in effect, per instance. |
| `--lsx-marquee-gap` | The gap in effect, per instance. |
| `--lsx-marquee-shift` | Where a cycle ends, one copy plus one gap along the axis. |

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

The parts carry `data-slot`, as `track` (the moving row), `group` (one copy)
and `pause` (the toggle).
