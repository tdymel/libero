# Carousel

Crate: `libero`
Import: `use libero::components::Carousel;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/carousel>
Index: [index.md](index.md) lists every other page
Description: A strip of slides that snaps as it scrolls and knows which one it is on, with controls, indicators and optional autoplay.

A strip of slides that snaps as it scrolls and knows which slide it is on. The
browser does the scrolling, so touch and momentum feel native. A swipe, an
arrow key, a button and autoplay all land on the same slide.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Carousel, Text},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        Carousel {
            aria_label: "Product photos",
            slides: (1..=6)
                .map(|n| rsx! {
                    Box {
                        sx: sx()
                            .display("flex")
                            .align_items("center")
                            .justify_content("center")
                            .min_height("160px")
                            .height("100%")
                            .background(format!("primary.{n}"))
                            .color(format!("primary-contrast.{n}")),
                        Text { "Slide {n}" }
                    }
                })
                .collect(),
            per_view: 3.0,
            indicators: true,
        }
    }
}
```

With `per_view` above 1 the strip runs out of scroll before it runs out of
slides. Six slides three-up stop at index 3, not 5. The indicators follow, with
four dots rather than six, each named by the first slide it shows. The live
region names the slides in view, "Slides 1–3 of 6", counting a slide at least
half in view. When every slide fits, there is one position, one dot, and both
buttons are inactive. With no slides there is no status, no dots and no
buttons, and the empty track is not a tab stop.

`align` moves that window too. Six slides three-up reach indices 0-3 aligned to
the start, 1-4 centred and 2-5 aligned to the end. An `index` outside the
window is pulled into it, so a centred three-up carousel asked for slide 0
reports slide 1. Where a slide visibly rests shows best with a fractional
`per_view`. At a whole number the alignments often rest on the same offset.

A vertical carousel needs a `height`:

```rust,ignore
Carousel {
    aria_label: "Product photos",
    orientation: "vertical",
    height: "300px",
    slides: slides(),
}
```

A controlled slide:

```rust
use dioxus::prelude::*;
use libero::components::{Carousel, Image, Text};

#[component]
fn Demo(photos: Vec<Photo>) -> Element {
    let mut slide = use_signal(|| 0usize);

    rsx! {
        Carousel {
            aria_label: "Product photos",
            index: slide(),
            onindexchange: move |index| slide.set(index),
            slides: photos.iter().map(|photo| rsx! {
                Image { src: "{photo.url}", alt: "{photo.alt}", fit: "cover" }
            }).collect(),
        }
        Text { "Showing {slide() + 1} of {photos.len()}" }
    }
}
#
# #[derive(Clone, PartialEq)]
# struct Photo { url: String, alt: String }
```

Autoplay:

```rust,ignore
Carousel { aria_label: "Offers", autoplay: true, autoplay_delay: 6000, slides: slides() }
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `slides` | `Vec<Element>` | `vec![]` | The slides, in order. |
| `slide_label` | `Callback<usize, String>` | `{n} of {m}` | Each slide's accessible name. |
| `index` | `Option<usize>` | `None`, uncontrolled | The current slide. Set it and the carousel follows. |
| `onindexchange` | `EventHandler<usize>` | `None` | Fires once a scroll settles, and on every control, key, indicator and autoplay step. Safe to write straight back into `index`. An `index` out of reach is clamped and reported here. |
| `per_view` | `f64` | `1` | Slides visible at once. A fraction lets the next one peek in. |
| `gap` | `Size` | `md` | Between slides. |
| `align` | `CarouselAlign` | `center` | Where a snapped slide comes to rest, `start`, `center` or `end`. Shows best with a fractional `per_view`. Above `per_view` 1 it also moves which slides the strip can reach. |
| `orientation` | `Orientation` | `horizontal` | Scroll axis. |
| `height` | `ThemeAwareValue` | `auto` | Required for a vertical carousel, which has nothing else to take its height from. A slide is as long as the carousel makes it, so give its content `height: 100%`. |
| `controls` | `bool` | `true` | Previous and next buttons. |
| `indicators` | `bool` | `false` | The dot strip, one dot per place the strip can rest. That is fewer than the slides when `per_view` is above 1. |
| `aria_label` | `String` | localization `label` | Names the region. Unset, it falls back to the localization's label and warns. |
| `draggable` | `bool` | `false` | Drag to scroll with a mouse. Touch swipes without it. On Blitz and the WebView a drag stops once the pointer leaves the track. |
| `autoplay` | `bool` | `false` | Advances on a timer, with a pause button first in Tab order. Hover pauses it, and focus stops it until the button is pressed. Under `prefers-reduced-motion: reduce` it opens paused. Without `loop` it stops on the last slide and presses Pause; Play there starts over from the first. |
| `autoplay_delay` | `u32` | `4000` | Milliseconds between advances. |
| `r#loop` | `bool` | `false` | Wraps around at both ends. The cloned slides at each end are `aria-hidden` and `inert`, but repeat a slide's DOM: give interactive slide content no `id` or form `name`. |
| `parts` | `Parts<CarouselPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `Carousel` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `CarouselPart::Viewport` | `viewport` | Holds the track and the controls, and clips the strip. |
| `CarouselPart::Track` | `track` | The scrolling strip, the tab stop. |
| `CarouselPart::Slide` | `slide` | One slide. The current one also has `data-current="true"`. |
| `CarouselPart::Controls` | `controls` | The strip holding previous and next. |
| `CarouselPart::Control` | `control` | The previous or the next button. |
| `CarouselPart::Indicators` | `indicators` | The group of dots. |
| `CarouselPart::Indicator` | `indicator` | One dot. |
| `CarouselPart::Pause` | `pause` | The autoplay toggle. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Enters the track, then Previous and Next, then the indicators as one tab stop. With `autoplay`, the pause button comes first. |
| `Left` or `Right` | Moves to the previous or next slide, on the track or the indicators. |
| `Up` or `Down` | The same, when vertical. |
| `Home` or `End` | Goes to the first or last slide. |

### Libero handles

- The carousel is a region named by `aria_label`, and each slide is a group a
  screen reader calls a slide. Slides out of view leave the Tab and reading
  order.
- After each move a hidden status says which slide shows, out of how many. It
  stays quiet while autoplay rotates (WCAG 2.2.2).
- Previous and Next are buttons. The indicators are one Tab stop with the
  current one marked, and focus follows the slide on them.
- With `autoplay`, focus inside the carousel stops it until the pause button
  is pressed. Hover only pauses it.

### You must

- Set `aria_label` to name the region. Without it the theme's generic name is
  used, and a debug build warns.

### Example

A product gallery: `Carousel { aria_label: "Product photos", .. }`. A screen
reader lists a "Product photos" region, and after Next it hears which photo
shows, out of how many.

## Theme defaults

`CarouselDefaults` on the theme, as `theme.carousel`.

| Field | Type | Default | Description |
|---|---|---|---|
| `per_view` | `f64` | `1.0` | Slides visible at once. |
| `gap` | `Size` | `Md` | Between slides. |
| `align` | `CarouselAlign` | `Center` | Where a snapped slide rests. |
| `radius` | `Size` | `Sm` | Corner radius of a slide. |
| `controls` | `bool` | `true` | Prev/next buttons by default. |
| `indicators` | `bool` | `false` | Dot strip by default. |
| `control_size` | `&'static str` | `28px` | Diameter of a control. |
| `controls_offset` | `Size` | `Sm` | Inset from the viewport edge. |
| `indicator_length` | `&'static str` | `24px` | Along the scroll axis. |
| `indicator_current_length` | `&'static str` | `40px` | The current dot's length, so position is not shown by colour alone. |
| `indicator_thickness` | `&'static str` | `5px` | Across it. |
| `indicators_gap` | `&'static str` | `8px` | Between dots. |
| `indicator_color` | `ColorValue` | `muted.6` | An idle dot. It needs 3:1 against the surface (SC 1.4.11). `muted.6` is 3.32:1 on white, `muted.4` only 1.49:1. |
| `indicator_current_color` | `ColorValue` | `primary.6` | The current dot. |
| `control_background` | `ColorValue` | `white` | The previous/next and pause buttons' fill. |
| `control_hover_background` | `ColorValue` | `muted.1` | A previous/next button's fill under the pointer. |
| `control_color` | `ColorValue` | `muted.7` | The controls' glyph, and their focus ring. Change it with `control_background`. |
| `autoplay_delay` | `u32` | `4000` | Milliseconds between advances. |

The words are `CarouselLabels` in the [localization](localization.md).

| Field | Default | Description |
|---|---|---|
| `label` | `Carousel` | Stands in when `aria_label` is unset, which also warns. |
| `previous` / `next` | `Previous slide` / `Next slide` | The controls' names. |
| `indicator` | `Go to slide {n}` | An indicator's name. `{n}` is the first slide it shows. |
| `indicators` | `Choose slide` | Names the dot strip's group. |
| `slide` | `{n} of {m}` | A slide group's name. |
| `status` | `Slide {n} of {m}` | What the live region reads one-up. `{m}` counts resting positions, not slides. |
| `status_range` | `Slides {from}–{to} of {n}` | What it reads above one slide per view, the slides showing of all `{n}`. |
| `pause` | `Pause slideshow` | The autoplay button's name. It stays the same when paused, and `aria-pressed` carries the state. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-carousel-gap` | Between slides. `--lsx-carousel-gap-override` is the `gap` prop. |
| `--lsx-carousel-per-view` | Unitless slides per view. `--lsx-carousel-per-view-override` is the `per_view` prop. |
| `--lsx-carousel-radius` | A slide's corner radius. |
| `--lsx-carousel-control-size` | Diameter of a control and of the pause button. |
| `--lsx-carousel-controls-offset` | Inset from the viewport edge. |
| `--lsx-carousel-indicator-length` / `-thickness` | Indicator geometry along and across the axis. |
| `--lsx-carousel-indicator-current-length` | The current dot's length. Position is not carried by colour alone, since the two dot colours are close in luminance. |
| `--lsx-carousel-indicators-gap` | Between indicators. |
| `--lsx-carousel-indicator-color` / `-current-color` | An idle dot and the current one. |
| `--lsx-carousel-control-background` / `-hover-background` | A control's fill, and a previous/next control's under the pointer. |
| `--lsx-carousel-control-color` | A control's glyph and focus ring. |
| `--lsx-carousel-height` | Set from the `height` prop. The track is `auto` without it. |

## Data attributes

| `data-state` token | Where | When |
|---|---|---|
| `horizontal` / `vertical` | root, track, controls, indicators | The orientation. |
| `align-start` / `align-center` / `align-end` | each slide | From `align`. |
| `current` | the current slide, the current indicator | Its index is the current one. |
| `disabled` | a control | The carousel is at that end. |
| `dragging` | the track | A mouse drag is in progress. Smooth scrolling is off for it. |
| `seam` | the track | A looping strip is jumping across the seam. Smooth scrolling is off for the jump. |

Each slide also carries `data-current="true"` when it is the current one.
