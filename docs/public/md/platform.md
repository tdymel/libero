# Platform

Crate: `libero`
Import: `use libero::platform::{clock, document, keyboard, scroll, timer};`
Index: [index.md](index.md) - every other component's markdown page
Description: The platform APIs for timers, document-level keys, scroll, focus and viewport, the clock and element handles, and how to use them where a renderer lacks one.

Everything that reaches past dioxus to the machine goes through
`libero::platform`. Components use it, and so can you.

## The APIs

| API | What it's for |
|---|---|
| `timer()` | Run a callback after a delay or on an interval. |
| `keyboard()` | Hear key presses anywhere in the document, for shortcuts. |
| `scroll()` | Hear anything scrolling, not only the page. |
| `document()` | Read the focused element and the viewport size. |
| `clock()` | Today's date in the user's time zone. |
| `use_element()` | Focus, scroll and measure one element you mount. |

## Using them

Each accessor returns an `Option`. `None` means the running renderer can't do
it, so branch once and let the feature be absent there instead of unwrapping.

A callback API hands back a subscription, and dropping it stops the callback.
Keep it in a signal for as long as the component lives. The callback runs
outside every scope, so write what it learns into a signal.

Element reads such as `dimensions()` are futures. Start one in an event handler
and await it in a `spawn`. A renderer that can't serve one answers
`PlatformError::Unsupported`.
