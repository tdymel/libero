# Timeline

Crate: `libero`
Import: `use libero::components::{Timeline, TimelineEvent, TimelineLine};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/timeline>
Index: [index.md](index.md) lists every other page
Description: An ordered list of events drawn against a rail, with an `active` index colouring the bullets and connectors up to the current one.

An ordered list of events drawn against a rail, rendered as an
`<ol role="list">`. `active` marks the current event. Bullets up to it fill
with the accent and the connectors between them draw in it, so the rail reads
as progress. Done and pending bullets differ by shape too, not by color alone.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Text, Timeline, TimelineEvent};

#[component]
fn Demo() -> Element {
    rsx! {
        Timeline {
            active: 2,
            items: vec![
                TimelineEvent::new("Pushed to main")
                    .content(rsx! { Text { "3 commits" } }),
                TimelineEvent::new("Review requested")
                    .content(rsx! { Text { "@tom" } }),
                TimelineEvent::new("Deployed")
                    .content(rsx! { Text { "v2.4.0 to production" } }),
                TimelineEvent::new("Rolled back")
                    .content(rsx! { Text { "reverted in 4 minutes" } }),
            ],
        }
    }
}
```

`active` is controlled. Bind it to whatever knows how far along the process
is. An index past the end clamps to the last event, so "step 7 of 4" means
finished.

```rust
use dioxus::prelude::*;
use libero::components::{Text, Timeline, TimelineEvent};

#[component]
fn Demo() -> Element {
    let mut step = use_signal(|| 1usize);

    rsx! {
        Timeline {
            active: step(),
            items: vec![
                TimelineEvent::new("Ordered").content(rsx! { Text { "Payment captured" } }),
                TimelineEvent::new("Packed").content(rsx! { Text { "Two parcels" } }),
                TimelineEvent::new("Shipped").content(rsx! { Text { "DHL, tracked" } }),
                TimelineEvent::new("Delivered").content(rsx! { Text { "Signed for" } }),
            ],
        }
    }
}
```

A bullet draws active for `0..=active`, and the connector below an event for
`0..active`. So the rail between completed events is filled, and the span below
the current one is not.

`.color(..)` gives one event its own accent, and `.line(..)` styles the
connector below it:

```rust
use dioxus::prelude::*;
use libero::components::{Text, Timeline, TimelineEvent, TimelineLine};

#[component]
fn Demo() -> Element {
    rsx! {
        Timeline {
            active: 3,
            items: vec![
                TimelineEvent::new("Deployed").content(rsx! { Text { "v2.4.0" } }),
                TimelineEvent::new("Health check failed")
                    .color("error")
                    .line(TimelineLine::Dashed),
                TimelineEvent::new("Rolled back").color("error"),
            ],
        }
    }
}
```

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `TimelinePart::Item` | `item` | One event's `<li>`. |
| `TimelinePart::Bullet` | `bullet` | The dot, or the ring around an event's `bullet`. |
| `TimelinePart::Body` | `body` | Title and content, beside the rail. |
| `TimelinePart::Title` | `title` | The event's title. |

## Accessibility

### Libero handles

- The list gives a screen reader the position and count the rail shows.
- Bullets are `aria-hidden`, and the title is the text. A custom `.bullet(..)`
  is hidden too.

### You must

- Never put anything focusable in a `.bullet(..)`. It would stay a tab stop with
  no name. Interactive content belongs in `.content(..)`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `items` | `Vec<TimelineEvent>` | `vec![]` | The events, in render order. |
| `active` | `usize` | `None` | The current event. Bullets up to and including it, and the connectors between them, draw in the accent. An index past the end clamps to the last event, so "step 7 of 4" means finished. |
| `align` | `TimelineAlign` | `start` | `start`, `end`, or `alternate` for content on both sides of a centred rail. Mirrored in a right-to-left layout. `alternate` fills its parent, so a narrower parent makes it narrower. |
| `color` | `ThemeAwareValue` | `primary` | The active accent. An event's own `.color(..)` overrides it. |
| `bullet_size` | `Size` | `md` | Bullet diameter. |
| `radius` | `Size` | `xl` | Bullet corner radius. `xl` is a dot. |
| `gap` | `Size` | `xl` | Space between events, which is also each connector's length. |
| `parts` | `Parts<TimelinePart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

`TimelineEvent`

| Method | Type | Default | Description |
|---|---|---|---|
| `new(title)` | `impl Into<OptionLabel>` | required | The event's name, optionally with its own rendering. |
| `.content(..)` | `Element` | none | The body below the title. |
| `.bullet(..)` | `Element` | the dot | An icon or avatar inside the bullet. Inverts when active. |
| `.color(..)` | `impl Into<ThemeAwareValue>` | inherits | This event's own accent. |
| `.line(..)` | `TimelineLine` | `Solid` | The connector below this event, `Solid`, `Dashed` or `Dotted`. The last event has none. |

Like every component, `Timeline` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`TimelineDefaults` on the theme, as `theme.timeline`.

| Field | Type | Default | Description |
|---|---|---|---|
| `align` | `TimelineAlign` | `Start` | Which side content sits on. |
| `color` | `&'static str` | `"primary.6"` | The active accent. |
| `line_color` | `&'static str` | `"muted.6"` | Inactive bullets and connectors, 3:1 on the page (WCAG 1.4.11). |
| `bullet_background` | `&'static str` | `PAPER_BACKGROUND` | The bullet's fill, and the glyph colour once it inverts. Follows the surface color, so dark mode comes from `PaperDefaults`. |
| `radius` | `Size` | `Xl` | Bullet corner radius. |
| `bullet_size` | `Size` | `Md` | Which step of `bullet_sizes` is the default. |
| `bullet_sizes` | `Sizes<u16>` | 12/16/20/24/28/32 px | The bullet scale. |
| `line_width` | `u8` | `2` | Connector thickness, and the bullet's ring. |
| `gap` | `Size` | `Xl` | Which step of `gaps` is the default. |
| `gaps` | `Sizes<u16>` | 12/16/24/32/40/48 px | The event-spacing scale. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-timeline-color` | The active accent. Set per event by `.color(..)`. |
| `--lsx-timeline-line-color` | Inactive bullets and connectors. |
| `--lsx-timeline-line-width` | Connector thickness and bullet ring width. |
| `--lsx-timeline-bullet-background` | The bullet's fill, `var(--lsx-paper-background)` by default. |
| `--lsx-timeline-bullet-size-*` | The bullet scale, one per `Size`. |
| `--lsx-timeline-gap-*` | The event-spacing scale, one per `Size`. |

Some are set per element. `--lsx-timeline-bullet`, `--lsx-timeline-space` and
`--lsx-timeline-radius` hold the picked step of each size on the root.
`--lsx-timeline-line-style`, `--lsx-timeline-connector` and
`--lsx-timeline-marker` hold each event's line style and its two active
colours.

## Data attributes

On the root.

| Token | When |
|---|---|
| `align-start` / `align-end` / `align-alternate` | The resolved `align`. |
| `xs`..`xxl` | The resolved `bullet_size`. |
| `radius-xs`..`radius-xxl` | The resolved `radius`. |
| `gap-xs`..`gap-xxl` | The resolved `gap`. |

On each event, and on its bullet.

| Token | When |
|---|---|
| `active` | This event is at or before `active`. |
| `line-active` | The connector below this event is before `active`. |
| `with-child` | The bullet holds a caller's element (bullet only). |
