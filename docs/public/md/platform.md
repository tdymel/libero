# Platform

Crate: `libero`
Import: `use libero::{hooks::use_element, platform::{clock, color_scheme, document, keyboard, scroll, timer}};`
Index: [index.md](index.md) lists every other page
Description: Every platform API (elements, timers, keys, scroll, the document, the colour scheme and the clock), what each makes possible, and how to use them where a renderer lacks one.

Everything that reaches past dioxus to the machine goes through
`libero::platform`. Components use it, and so can you. One call works on the web
and natively, and each renderer answers with what it has.

## The APIs

| API | What it gives | What you can build |
|---|---|---|
| `use_element()` | Focus, scroll, measure and query one element you mount. | Focus a field when a dialog opens, scroll a row into view, size a popup to its trigger. |
| `timer()` | A callback after a delay or on an interval. | Autoplay, a toast that closes itself, a debounced search. |
| `keyboard()` | Key presses anywhere in the document. | App-wide shortcuts, such as Ctrl+K for a command palette. |
| `scroll()` | Scrolling anywhere, not only the page. | Close a popup when its trigger scrolls away, a header that hides on scroll. |
| `document()` | The focused element, the viewport size, and attributes on the root element. | Put focus back where it was, lay out by window size, set a theme attribute on the root. |
| `color_scheme()` | The system's light or dark, its changes, and a stored choice. | Follow the system theme and remember what the reader picked. |
| `clock()` | Today's date in the user's time zone. | Mark today in a calendar, start a date field on today. |

The web and Blitz answer all of them. Android's WebView has no `keyboard()`,
`scroll()`, `document()` or `color_scheme()` yet, and some element calls there
are unsupported.

## Using them

Each accessor returns an `Option`. `None` means the running renderer can't do
it, so branch once and let the feature be absent there instead of unwrapping.

A callback API hands back a subscription, and dropping it stops the callback.
Keep it in a signal for as long as the component lives, and write what the
callback learns into a signal.

Element reads such as `dimensions()` are futures. Start one in an event handler
and await it in a `spawn`. A renderer that can't serve one answers
`PlatformError::Unsupported`.
