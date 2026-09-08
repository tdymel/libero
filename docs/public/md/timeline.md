# Timeline

Crate: `libero`
Import: `use libero::components::{Timeline, TimelineEvent, TimelineLine};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/timeline>
Index: [index.md](index.md) - every other component's markdown page
Description: An ordered list of events drawn against a rail, with an `active` index colouring the bullets and connectors up to the current one.

An ordered list of events drawn against a rail. It renders an
`<ol role="list">` - the rail shows position and count visually, and the list is
how a screen-reader user gets the same two facts. `active` names the current
event: bullets up to and including it draw in the accent, as do the connectors
between them, so the rail reads as progress rather than as a highlight. Bullets
are decorative and hidden from the accessibility tree; the title is the text.

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

`items` is a `Vec`, not compound children, and that is a dioxus constraint
rather than a preference: there is no `Children.map`, so `Timeline.Item`
children could not see their own order, and an item's connector colour is a
function of `active` and its own index. A `Vec` makes that plain indexing.

`active` is strictly controlled. Bind it to whatever already knows how far
along the process is, and the rail follows. An index past the end clamps to the
last event, so "step 7 of 4" means finished rather than nothing.

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

Two active states move independently. A bullet draws active for `0..=active`,
and the connector *below* an event draws active for `0..active` - so the rail
between completed events is filled, and the span below the current one is not.

## Events

`TimelineEvent` is a builder, because the parts compose: a title alone is the
common case, and content, a custom bullet, a colour and a line style are each
independently optional.

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

`.color(..)` overrides the timeline's accent for one event - an error step in
an otherwise unremarkable run - and `.line(..)` sets the connector *below* the
event it is called on. The last event has no connector: a rail that runs past
its final marker points at nothing.

A `.bullet(..)` holding an icon inverts when active: the ring fills with the
accent instead of outlining it, because a light glyph on a white ring is
invisible.

**Never put anything focusable in a bullet.** It is `aria-hidden="true"`, and
`aria-hidden` does not remove an element from the tab order - a `Button` or a
link in a bullet stays tabbable and announces as nothing, which is a dead stop
for a keyboard user. Nothing can detect this for you: `.bullet()` takes any
`Element`. Interactive content belongs in `.content(..)`.

## Alignment

`align` puts content on either side of the rail, or alternates it:

```rust,ignore
Timeline { align: "alternate", items: vec![/* .. */] }
```

`"alternate"` alternates at every width. There is no minimum below which it
falls back to one side: a timeline that narrow will be cramped, but asking for
one is a decision you have made and the component does not overrule it.

`"alternate"` also makes the list fill its parent's width, where `"left"` and
`"right"` shrink to their content - a rail centred in the list needs a width
to be centred in. Give it a parent with the width you want the rail centred
in, and that is also how you make it narrower.

`align: "right"` mirrors with physical properties. `Sx` has no logical ones and
nothing in the crate uses any, so right-to-left text is a library-wide gap
rather than something this component can answer.

## Accessibility

Bullets are `aria-hidden`, so a focusable element inside a bullet is a tab stop
with no accessible name - see "Events".

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `items` | `Vec<TimelineEvent>` | `vec![]` | The events, in render order. |
| `active` | `usize` | - | The current event. Bullets `0..=active` and connectors `0..active` draw active; out of range clamps to the last event. |
| `align` | `TimelineAlign` | `theme.timeline.align` | `"left"`, `"right"` or `"alternate"`. |
| `color` | `ThemeAwareValue` | `theme.timeline.color` | The active accent. A per-event `.color(..)` overrides it. |
| `bullet_size` | `Size` | `theme.timeline.bullet_size` | Bullet diameter. |
| `radius` | `Size` | `theme.timeline.radius` | Bullet corner radius; `xl` is the dot. |
| `gap` | `Size` | `theme.timeline.gap` | Space between events, which is also each connector's length. |

`TimelineEvent`:

| Method | Type | Default | Description |
|---|---|---|---|
| `new(title)` | `impl Into<OptionLabel>` | required | The event's name, optionally with its own rendering. |
| `.content(..)` | `Element` | none | The body below the title. |
| `.bullet(..)` | `Element` | the dot | An icon or avatar inside the bullet. Inverts when active. |
| `.color(..)` | `impl Into<ThemeAwareValue>` | inherits | This event's own accent. |
| `.line(..)` | `TimelineLine` | `Solid` | The connector below this event: `Solid`, `Dashed`, `Dotted`. |

Like every component, `Timeline` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`TimelineDefaults` on the theme, as `theme.timeline`.

| Field | Type | Default | Description |
|---|---|---|---|
| `align` | `TimelineAlign` | `Left` | Which side content sits on. |
| `color` | `&'static str` | `"primary.6"` | The active accent. |
| `line_color` | `&'static str` | `"muted.3"` | Inactive bullets and connectors. |
| `bullet_background` | `&'static str` | `PAPER_BACKGROUND` | The bullet's fill, and the glyph colour once it inverts. Reads the surface token, so dark mode is a change to `PaperDefaults`. |
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
| `--lsx-timeline-bullet-background` | The bullet's fill; `var(--lsx-paper-background)` by default. |
| `--lsx-timeline-bullet-size-*` | The bullet scale, one per `Size`. |
| `--lsx-timeline-gap-*` | The event-spacing scale, one per `Size`. |

Three more are resolved per element rather than per theme, so one rule serves
every combination instead of one rule per combination: `--lsx-timeline-bullet`,
`--lsx-timeline-space` and `--lsx-timeline-radius` hold the picked step of each
size axis on the root, and `--lsx-timeline-line-style`,
`--lsx-timeline-connector` and `--lsx-timeline-marker` hold each event's line
style and its two active colours.

## Data attributes

On the root:

| Token | When |
|---|---|
| `align-left` / `align-right` / `align-alternate` | The resolved `align`. |
| `xs`..`xxl` | The resolved `bullet_size`. |
| `radius-xs`..`radius-xxl` | The resolved `radius`. |
| `gap-xs`..`gap-xxl` | The resolved `gap`. |

On each event, and on its bullet:

| Token | When |
|---|---|
| `active` | This event is at or before `active`. |
| `line-active` | The connector below this event is before `active`. |
| `with-child` | The bullet holds a caller's element (bullet only). |
