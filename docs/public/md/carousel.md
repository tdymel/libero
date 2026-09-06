# Carousel

Crate: `libero`
Import: `use libero::components::Carousel;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/carousel.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A scroll-snap strip of slides that knows which one it is on, with controls, indicators, optional autoplay and no JavaScript carousel library underneath.

A scroll-snap strip that knows which slide it is on. The scrolling is the
browser's - so touch, momentum and rubber-banding are the platform's, not ours
- and the index is derived from the scroll position, which is what makes a
swipe, an arrow key, a control click and an autoplay tick all end up in the
same place. `per_view` sets how many slides are visible at once, fractionally
if you want the next one peeking.

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

Slides are a `Vec<Element>` rather than children. Reasoning about their order
and count is the whole job of this component, and dioxus cannot inspect
children. They are not a `Callback` either: a snap track has every slide in the
DOM, so a closure would buy no laziness, and a closure prop that should change
can compare equal.

`per_view` above `1` shows several slides at once, and the strip then runs out
of scroll before it runs out of slides - six slides three-up stop at index 3,
not 5. The indicator strip and the live region follow that: four dots rather
than six, and a status that runs "Slide 1 of 4" to "Slide 4 of 4". They count
where the strip can rest, as Mantine does, so a strip whose slides all fit is
one position - one dot, "Slide 1 of 1", and both controls inactive.

With no slides at all there is no position, so nothing counts one: no status,
no dots, no controls and no pause button. The root still renders, so the
caller's size and placement hold while slides load, but it is not a named
region and the empty track is not a tab stop.

`align` moves that window rather than just the look. Six slides three-up reach
indices 0-3 aligned to the start, 1-4 centred and 2-5 aligned to the end,
because at either end the browser clamps the scroll and the slide sitting in the
aligned position is not the first or last one. The reported index, the lit dot
and the live region all follow what is actually in that position, and an `index`
outside the window is pulled into it - so a centred three-up carousel asked for
slide 0 reports slide 1, which is the one genuinely centred. At `per_view: 1`
all three alignments coincide.

Where a slide visibly rests needs a fractional `per_view`. With equal slides at
a whole number, a centred or end-aligned snap often lands on the same offset as
a start-aligned one - three-up, every alignment rests on whole-slide steps, and
only the reported index differs. At the very start of the strip, too, every
alignment rests the same way. The demo above is `1.5` up and opens on slide 3,
so switching `align` moves that slide to the left, the middle or the right.

## Vertical

A vertical carousel has nothing to take its height from, so `height` is
required there:

```rust,ignore
Carousel {
    aria_label: "Product photos",
    orientation: "vertical",
    height: "300px",
    slides: slides(),
}
```

A vertical slide is as long as the carousel gives it - one third of the
`height` at `per_view: 3`, all of it at `per_view: 1` - so slide content with a
fixed height paints that much and leaves the rest of the slide blank. Give the
content `height: 100%` and keep any fixed height as a `min-height`, which is
what the usage snippet above does.

## Controlled slide

Pass `index` and the carousel follows it; read `onindexchange` to follow the
carousel. It fires once a scroll settles rather than on every frame, so it is
safe to write straight back into the signal that drives it.

While you are driving `index`, it can also fire with an index you did not ask
for, and that is deliberate: if the one you passed is outside the reachable
window - slide 0 on a centred three-up strip, or an index left over after the
slides got shorter - the carousel clamps it and tells you. Without that the two
of you would disagree forever, with your code pushing the same unreachable value
back on every render and nothing ever converging. An uncontrolled carousel stays
quiet, because there is no second party holding a wrong value.

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

## Autoplay

`autoplay` advances on a timer and brings its whole WCAG 2.2.2 contract with
it: a real pause control, pause on hover, and pause on focus landing anywhere
inside. The pause control is a button rather than a hover affordance, because
hovering helps neither a keyboard nor a touch user. While it rotates unattended
the live region is `aria-live="off"`, and it becomes polite again the moment it
stops.

```rust,ignore
Carousel { aria_label: "Offers", autoplay: true, autoplay_delay: 6000, slides: slides() }
```

## Dragging

`draggable` adds mouse drag-to-scroll. It does nothing for touch, deliberately:
a swipe is already the platform's own scroll, and making the track a drag
handle would need `touch-action: none`, which would take that away. Pointer
capture is unsupported on Blitz and on the WebView floor, so a mouse drag there
loses the pointer once it leaves the track; touch and every other input are
unaffected.

## Accessibility

- Set `aria_label`: it names the region.
- The track is a tab stop. `ArrowLeft` / `ArrowRight`, or `ArrowUp` / `ArrowDown` when vertical: previous and next. `Home` / `End`: first and last.
- The indicators are one tab stop with the same keys, and focus follows the slide.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `slides` | `Vec<Element>` | `[]` | The slides, in order. |
| `slide_label` | `Callback<usize, String>` | `{n} of {m}` | Each slide group's accessible name. |
| `index` | `usize` | uncontrolled | The current slide. Set it and the carousel follows. |
| `onindexchange` | `EventHandler<usize>` | - | Fired once a scroll settles, and on every control, key, indicator and autoplay tick. |
| `per_view` | `f64` | `1` | Slides visible at once. Fractional peeks the next one. |
| `gap` | `Size` | `md` | Between slides. |
| `align` | `CarouselAlign` | `center` | Where a snapped slide comes to rest - `start`, `center` or `end`. Visible with a fractional `per_view`; at a whole one the alignments can share their resting offsets. Above `per_view: 1` it also moves which indices are reachable. |
| `orientation` | `Orientation` | `horizontal` | Scroll axis. Note this differs from `Orientation`'s own default. |
| `height` | `ThemeAwareValue` | `auto` | Required for a vertical carousel. |
| `controls` | `bool` | `true` | Prev/next buttons. |
| `indicators` | `bool` | `false` | The dot strip - one per scroll position, which is fewer than the slides when `per_view` is above 1. |
| `aria_label` | `String` | theme `label` | Names the region. Leaving it unset falls back to the theme's generic name and warns. |
| `draggable` | `bool` | `false` | Mouse drag-to-scroll. Touch already swipes natively. |
| `autoplay` | `bool` | `false` | Advances on a timer, with a pause control and pause on hover and focus. |
| `autoplay_delay` | `u32` | `4000` | Milliseconds between advances. |
| `r#loop` | `bool` | `false` | Wraps at both ends, by cloning slides onto each end. The clones are `aria-hidden`. |

Like every component, `Carousel` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

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
| `indicator_current_length` | `&'static str` | `40px` | The current dot's length - the second channel beside its colour. |
| `indicator_thickness` | `&'static str` | `5px` | Across it. |
| `indicators_gap` | `&'static str` | `8px` | Between dots. |
| `indicator_color` | `ColorValue` | `grey.6` | An idle dot. It is a button carrying the only visible position affordance, so it owes 3:1 against the surface (SC 1.4.11): `grey.6` is 3.32:1 on white, `grey.4` is 1.49:1. |
| `indicator_current_color` | `ColorValue` | `primary.6` | The current dot. |
| `control_background` | `ColorValue` | `white` | The previous/next and pause buttons' fill. |
| `control_hover_background` | `ColorValue` | `grey.1` | A previous/next button's fill under the pointer. |
| `control_color` | `ColorValue` | `grey.7` | The controls' glyph, and their focus ring. Change it with `control_background`. |
| `autoplay_delay` | `u32` | `4000` | Milliseconds between advances. |
| `label` | `&'static str` | `Carousel` | Stands in when a caller omits `aria_label` - which also warns. |
| `previous_label` / `next_label` | `&'static str` | `Previous slide` / `Next slide` | The controls' names. |
| `indicator_label` | `&'static str` | `Go to slide {n}` | An indicator's name. |
| `slide_label` | `&'static str` | `{n} of {m}` | A slide group's name. |
| `status_label` | `&'static str` | `Slide {n} of {m}` | What the live region reads. `{m}` counts resting positions, not slides. |
| `pause_label` | `&'static str` | `Pause slideshow` | The autoplay control's name. It does not change when paused: `aria-pressed` carries the state. |

The label fields are English literals on the theme, the same as `DateDefaults`.
The library has no i18n mechanism yet, so overriding them on the theme - or
passing `aria_label` and `slide_label` per instance - is how a carousel speaks
another language today.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-carousel-gap` | Between slides. `--lsx-carousel-gap-override` is the `gap` prop. |
| `--lsx-carousel-per-view` | Unitless; the slide-size formula divides by it. `--lsx-carousel-per-view-override` is the `per_view` prop. |
| `--lsx-carousel-radius` | A slide's corner radius. |
| `--lsx-carousel-control-size` | Diameter of a control and of the pause button. |
| `--lsx-carousel-controls-offset` | Inset from the viewport edge. |
| `--lsx-carousel-indicator-length` / `-thickness` | Indicator geometry along and across the axis. |
| `--lsx-carousel-indicator-current-length` | The current dot's length. Position is not carried by colour alone, since the two dot colours are close in luminance. |
| `--lsx-carousel-indicators-gap` | Between indicators. |
| `--lsx-carousel-indicator-color` / `-current-color` | An idle dot and the current one. |
| `--lsx-carousel-control-background` / `-hover-background` | A control's fill, and a previous/next control's under the pointer. |
| `--lsx-carousel-control-color` | A control's glyph and focus ring. |
| `--lsx-carousel-height` | Set from the `height` prop; the track is `auto` without it. |

## Data attributes

| `data-state` token | Where | When |
|---|---|---|
| `horizontal` / `vertical` | root, track, controls, indicators | The orientation. |
| `align-start` / `align-center` / `align-end` | each slide | From `align`. |
| `current` | the current slide, the current indicator | Its index is the current one. |
| `disabled` | a control | The carousel is at that end. |
| `dragging` | the track | A mouse drag is in progress; smooth scrolling is off for it. |
| `seam` | the track | A looping strip is jumping across the seam; smooth scrolling is off for the jump. |

Each slide also carries `data-current="true"` when it is the current one.
