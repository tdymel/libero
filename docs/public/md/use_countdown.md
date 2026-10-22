# Countdown

Crate: `libero`
Import: `use libero::{hooks::{format_duration, use_countdown}, localization::DurationStyle};`
Feature: `use-countdown` (off by default; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/countdown.rs>
Index: [index.md](index.md) lists every other page
Description: The time left until an end, live, on one ticker every countdown shares, and the duration as a clock, short units or spoken words in the active language.

`use_countdown(end: SystemTime) -> ReadSignal<Duration>` is the time left until
`end`, live, and `Duration::ZERO` once it passed. All mounted countdowns share
one 1 s ticker, aligned to the second, which stops when none is mounted or all
reached zero. `format_duration(span, DurationStyle::Clock | Short | Spoken)`
writes it as `1:30`, `1 min 30 s` or `1 minute 30 seconds` in the active
language. Build `end` from `platform::clock().now()`: `SystemTime::now()`
panics on wasm.

## Usage

```rust
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text, VisuallyHidden},
    hooks::{format_duration, use_countdown},
    localization::DurationStyle,
    platform::clock,
};

fn in_90_seconds() -> SystemTime {
    clock().map_or(UNIX_EPOCH, |clock| clock.now()) + Duration::from_secs(90)
}

#[component]
fn Offer() -> Element {
    let mut end = use_signal(in_90_seconds);
    let left = use_countdown(end());

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Text {
                "Offer ends in "
                span { aria_hidden: "true", {format_duration(left(), DurationStyle::Clock)} }
                VisuallyHidden { {format_duration(left(), DurationStyle::Spoken)} }
            }
            Text { "In short units: {format_duration(left(), DurationStyle::Short)}" }
            Button { variant: "outlined", onclick: move |_| end.set(in_90_seconds()), "Restart" }
        }
    }
}
```

## API

```rust,ignore
pub fn use_countdown(end: SystemTime) -> ReadSignal<Duration>
pub fn format_duration(span: Duration, style: DurationStyle) -> String
impl DateLocale { pub fn format_duration(&self, span: Duration, style: DurationStyle) -> String }
```

| Style | English | German |
|---|---|---|
| `DurationStyle::Clock` | `1:04:05` | `1:04:05` |
| `DurationStyle::Short` | `1 h 4 min 5 s` | `1 Std. 4 Min. 5 Sek.` |
| `DurationStyle::Spoken` | `1 hour 4 minutes 5 seconds` | `1 Stunde 4 Minuten 5 Sekunden` |

The time left is in whole seconds, rounded up. A new `end` restarts the
countdown. `format_duration` reads the active language and panics outside a
`LiberoProvider`; `DateLocale::format_duration` takes the language itself. The
words are `DateLocale`'s `hours_short` and `hours_value` families, so an
override there changes them too.

## Accessibility

### Libero handles

- `DurationStyle::Spoken` says the time in the active language's words, `1
  minute 30 seconds`, where a clock face would be read as a ratio or a time of
  day.
- Every countdown on the page turns on the same second, so a reader comparing
  two never sees them disagree.

### You must

- Hide the clock face from screen readers (`aria-hidden`) and put the spoken
  text beside it in a `VisuallyHidden`, or in the element's `aria-label`.
- Keep the label out of a live region: it is not `aria-live`, since a polite
  region would speak every second. Announce a milestone once instead, such as
  "One minute left", in a `role="status"`.
- Give the reader a way to ask for more time when the countdown ends something
  they are doing (WCAG 2.2.1).

### Example

A checkout hold: "Seats held for 9:41" with "9 minutes 41 seconds" in a
`VisuallyHidden`, a status line saying "One minute left" once, and an Extend
button.

### Limits

- Whole seconds, rounded up: the label reads `0:00` only once the end has
  passed, up to a second after it.
