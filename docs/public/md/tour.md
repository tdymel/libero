# Tour

Crate: `libero`
Import: `use libero::components::{MaskClick, TourOptions, TourStep, TourView, use_tour};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/tour>
Index: [index.md](index.md) lists every other page
Description: A guided tour. A hook that dims the page around one element per step and explains it in a card beside it.

A guided tour. `use_tour` dims the page around one element per step and
explains it in a card beside it, with Back, Next and Skip. The steps are data;
a step's `target` is an element handle from [`use_element`](use_element.md), or
a selector for one in another component. The tour is fixed to the viewport, over
everything but notifications. With a `storage_key` it remembers being finished
or skipped across reloads: offer it while `tour.seen()` is false. The default
card is a [Dialog](dialog.md), placed like a [Popover](popover.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text, TourOptions, TourStep, use_tour},
    hooks::use_element,
};

#[component]
fn Demo() -> Element {
    let search = use_element();
    let mut searches = use_signal(|| 0);
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("welcome")
                .title("Welcome")
                .description("Two stops, under a minute."),
            TourStep::new("search")
                .target(search)
                .interactive(true)
                .title("Search")
                .description("Finds any page by its name. Try it: press it, or Tab to it."),
            TourStep::new("create")
                .target_selector("#tour-demo-create")
                .title("New project")
                .description("Starts an empty project."),
        ],
        storage_key: Some("docs-tour-seen".into()),
        ..Default::default()
    });

    rsx! {
        Flex {
            direction: "row",
            gap: "sm",
            Button {
                variant: "outlined",
                onmounted: search.mount(),
                attributes: search.attributes(),
                onclick: move |_| searches += 1,
                "Search"
            }
            // Found by its id: no handle to pass down.
            Button { id: "tour-demo-create", variant: "outlined", "New project" }
            Button { onclick: move |_| tour.start(), "Take the tour" }
            Button { variant: "text", onclick: move |_| tour.forget(), "Forget seen" }
            Text {
                size: "sm",
                role: "status",
                if tour.seen() { "Seen" } else { "Not seen yet" }
                ", searches: {searches}"
            }
        }
    }
}
```

Spread `handle.attributes()` on each target: a WebView finds the element by it.
A `target_selector` is looked up in the document when its step shows; a WebView
cannot look one up and shows that card in the middle.
Call `use_tour` in a component that outlives every target, and `tour.start()`
from a handler, so focus returns to the trigger. A tour never opens on mount.

`TourHandle` is `Copy`: `start()`, `next()`, `prev()`, `go_to(index)`,
`close()` (ends early, `onclose`), `finish()` (ends as done, `onfinish`),
`is_open()`, `index()` and `total()`, and with a `storage_key`, `seen()` and
`forget()`.

### Your own card

`card` draws the card's inside from a `TourView`: the step, its `progress`
text, and `next()`, `prev()`, `go_to(index)`, `close()` and `finish()`. The
tour keeps the hole, the placement, the name, the focus and the keys. The box
around it has no surface, so draw one.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Paper, Text, Title, TourOptions, TourStep, TourView, use_tour},
    hooks::use_element,
    sx::sx,
};

#[component]
fn Demo() -> Element {
    let filters = use_element();
    let card = use_callback(|view: TourView| {
        let title = view.step.title.clone().unwrap_or_default();
        let (skip, next) = (view.clone(), view.clone());
        rsx! {
            Paper {
                shadow: "md",
                sx: sx().padding("md"),
                Text { size: "xs", "{view.progress}" }
                Title { size: "sm", component: "h2", "{title}" }
                Flex {
                    direction: "row",
                    gap: "xs",
                    justify: "flex-end",
                    Button { variant: "text", size: "sm", onclick: move |_| skip.close(), "Not now" }
                    Button { size: "sm", onclick: move |_| next.next(), "Next" }
                }
            }
        }
    });
    let tour = use_tour(TourOptions {
        steps: vec![TourStep::new("filters").target(filters).title("Filters")],
        card: Some(card),
        ..Default::default()
    });

    rsx! {
        Button { variant: "outlined", onmounted: filters.mount(), "Filters" }
        Button { onclick: move |_| tour.start(), "Show me around" }
    }
}
```

### Controlled

`current: Some(index)` makes the step shown yours: the buttons, keys and
handle calls only call `onchange`, and the tour shows the step you pass back.

```rust,ignore
let mut step = use_signal(|| 0usize);
let tour = use_tour(TourOptions {
    steps,
    current: Some(step()),
    onchange: Some(use_callback(move |index| step.set(index))),
    ..Default::default()
});
```

### Show it once

`storage_key` keeps in local storage that the tour was finished or closed
early. `tour.seen()` reads it, reactively, and `tour.forget()` drops it, for a
"show the tour again" setting. `start()` still starts: offer the tour while
`!tour.seen()`. Without a key, `onfinish` and `onclose` say it was seen; keep
that where your app keeps its settings.

## API

### `use_tour`

```rust,ignore
pub fn use_tour(options: TourOptions) -> TourHandle
```

### `TourOptions`

`TourOptions` implements `Default`; set `steps`.

| Field | Type | Default | Description |
|---|---|---|---|
| `steps` | `Vec<TourStep>` | required | The stops, in order. A step without a `target` shows its card in the middle of the screen. |
| `current` | `Option<usize>` | - | Makes the step shown controlled: the buttons, keys and handle calls only call `onchange`. |
| `onchange` | `Option<Callback<usize>>` | - | Called with the step a button, key or handle call asks for. |
| `onfinish` | `Option<Callback<()>>` | - | Called once when Next is pressed on the last step. |
| `onclose` | `Option<Callback<usize>>` | - | Called with the step shown when the tour ends early: Escape, Back, Skip or the close button. Steps going empty while open end it too, with the last step shown. |
| `mask_click` | `MaskClick` | `None` | What a press on the dimmed page does: `None`, `Close` or `Next`. |
| `aria_label` | `Option<String>` | - | Names every step's card, over the step titles. |
| `card` | `Option<Callback<TourView, Element>>` | - | Draws the card's inside in place of the default. |
| `sx` | `Input<Sx>` | - | Styles the card. |
| `parts` | `Input<Parts<TourPart>>` | - | Styles the mask, the highlight and the card's parts. |
| `storage_key` | `Option<String>` | - | Remembers in local storage that the tour was finished or closed early, for `tour.seen()`; `tour.forget()` drops it. Read at mount. |

### `TourStep`

| Method | Type | Default | Description |
|---|---|---|---|
| `new(key)` | `impl Into<String>` | required | Tells the steps apart. The card is drawn afresh when it changes. |
| `target` | `ElementHandle` | - | The element the hole goes around. |
| `target_selector` | `impl Into<String>` | - | A CSS selector for the target, as `"#search"`, looked up when the step shows. `target` wins. Not on a WebView. |
| `interactive` | `bool` | `false` | Lets presses through the hole to the target, and puts the target in the Tab order beside the card. |
| `title` | `impl Into<String>` | - | The card's heading and, unless the tour has an `aria_label`, its name. |
| `description` | `impl Into<String>` | - | The card's text. |
| `content` | `Element` | - | Shown in place of `description`, for rich content. |
| `side` | `Side` | `Bottom` | The card's side of the target. It flips when that side has no room. |
| `align` | `Align` | `Center` | Where the card lines up along that side. |
| `padding` | `f64` | `6` | Room between the target and the edge of the hole, in pixels. |
| `radius` | `f64` | `4` | The hole's corner radius, in pixels. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `TourPart::Mask` | `mask` | The transparent layer over the page that takes every press. Four strips around the hole on an `interactive` step. |
| `TourPart::Highlight` | `highlight` | The hole around the target. Its outer shadow is the dimming. |
| `TourPart::Positioner` | `positioner` | Places the card beside the target, or in the middle. |
| `TourPart::Card` | `card` | The `dialog`: the default card, or the box around `card`'s. |
| `TourPart::Header` | `header` | The row holding the title and the close button. |
| `TourPart::Title` | `title` | The step's title. |
| `TourPart::Close` | `close` | The close button. |
| `TourPart::Body` | `body` | The step's description or content. |
| `TourPart::Footer` | `footer` | The progress text and the buttons. |
| `TourPart::Progress` | `progress` | "2 of 3". |
| `TourPart::Skip` | `skip` | Ends the tour early. Not on the last step. |
| `TourPart::Previous` | `previous` | Not on the first step. |
| `TourPart::Next` | `next` | Reads Done on the last step. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `→` | Goes to the next step. On the last step it does nothing: only Done finishes. Under `dir="rtl"`, `←` does. A held key steps once. |
| `←` | Goes to the previous step. |
| `Escape` | Ends the tour early and returns focus to what started it. |
| `Tab` or `Shift+Tab` | Moves the focus within the card. It does not leave while the tour shows, but for an `interactive` step's target: Tab past the card's last control reaches it, and Tab from it returns to the card. On the target the arrows are its own; Escape still ends the tour. |

### Libero handles

- Each step's card is a `dialog` with `aria-modal`, named by the step title and
  described by its text and its progress ("2 of 3"), also with a custom `card`,
  which the tour describes by a hidden text of its own. Focus moves to it on every step.
- The card names its arrow keys in `aria-keyshortcuts`; few screen readers
  announce it, so say the keys in the first step's text too.
- The hole has a 2px ring of its own, and an outline in forced colours, so the
  highlighted element stands out on a dark page too.
- A card taller than the room it has scrolls, so its buttons stay reachable at
  400% zoom or on a phone held sideways.
- The highlighted element cannot be pressed unless its step is `interactive`,
  and a press on the dimmed page does nothing unless `mask_click` says so.
- An `interactive` step's card has `aria-modal="false"`: the target outside it
  is reachable.
- Each step scrolls its target into view; smoothly, unless the user reduces
  motion. The hole glides to a new step's target, without animation then too,
  and follows a scroll at once.
- Android's Back button ends the tour, as Escape does, rather than the app.
- A `target_selector` that matches nothing when its step shows centres the
  card, with a warning in a debug build.
- A target that never mounts, or renders nothing (`display: none`), shows its
  step's card in the middle, with a warning in a debug build.
- Steps that go empty while the tour shows end it, as closing does.

### You must

- Call `tour.start()` from a button's handler, so focus returns there. Never
  start a tour on mount.
- Give each step a `title`, or the tour an `aria_label`, so each card has a
  name of its own.

### Example

A first-run tour over Search and New project: "Take the tour" moves focus into
the Welcome card, → or Next moves the hole to Search, and Escape ends the tour
and returns focus to "Take the tour".

### Limits

- The hole follows the target on scroll and window resize, not when the target
  alone changes size.
- In a native window, the hole drifts under a page scroll the app causes
  itself; libero hears its own scrolls and the wheel.

## Theme defaults

`TourDefaults` on the theme, `theme.tour`. The card's gap to the hole and its
distance to the window edge come from `PopoverDefaults`.

| Field | Type | Description |
|---|---|---|
| `padding` | `f64` | Room between a step's target and the edge of its hole, `6`. |
| `radius` | `f64` | The hole's corner radius, `4`. |
| `max_width` | `f64` | The card's widest, `360`. |

## Localization

`TourLabels` on the localization, `localization.tour`: `label`, `previous`,
`next`, `done`, `skip`, `progress` (`"{n} of {m}"`) and `close`, in English and
German.

## Data attributes

| Attribute | Element |
|---|---|
| `data-lsx-tour` | The tour layer, in the portal outlet. |
| `data-slot` | Each part, by the names in `TourPart`. |
