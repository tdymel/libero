# Media

Crate: `libero`
Import: `use libero::hooks::{MediaError, MediaHandle, use_media};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/media.rs>
Index: [index.md](index.md) lists every other page
Description: Plays and reads an `<audio>` or `<video>` you render yourself, one signal per read; the engine behind `Audio` and `Video`.

`use_media() -> MediaHandle` plays and reads one `<audio>` or `<video>` you
render yourself: spread its `attributes()` and set `mount()` as the element's
`onmounted`. Each read is its own signal, so a time tick re-renders only what
shows the time. [Audio](audio.md) and [Video](video.md) are built on it and
take a handle through their `media` prop.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::use_media,
};

#[component]
fn OwnPlayer() -> Element {
    let media = use_media();
    let time = media.current_time() as u64;
    let status = match (media.error(), media.buffering()) {
        (Some(_), _) => "The audio could not be played.",
        (None, true) => "Loading",
        (None, false) => "",
    };

    rsx! {
        audio { src: "/podcast.mp3", onmounted: media.mount(), ..media.attributes() }
        Flex { gap: "sm", align: "center",
            Button {
                onclick: move |_| media.toggle(),
                if media.paused() { "Play" } else { "Pause" }
            }
            Button { variant: "outlined", onclick: move |_| media.seek(0.0), "Restart" }
            Text { "{time / 60}:{time % 60:02}" }
            div { role: "status", "{status}" }
        }
    }
}
```

## API

```rust,ignore
pub fn use_media() -> MediaHandle

pub enum MediaError { Aborted, Network, Decode, SourceNotSupported }
```

| Method of `MediaHandle` | Description |
|---|---|
| `attributes()`, `mount()` | Spread on the element and set as its `onmounted`, so the handle finds it on every platform. |
| `element()` | The `ElementHandle` underneath, for focus or measuring. |
| `play()`, `pause()`, `toggle()` | A browser may refuse a `play()` no press started (autoplay policy): it then stays paused. |
| `seek(seconds)` | Jumps from the start, clamped to the duration once known. |
| `set_volume(0.0..=1.0)`, `set_muted(bool)`, `set_rate(f64)` | Volume, mute and speed; `1.0` is normal speed. |
| `paused()`, `ended()`, `current_time()`, `volume()`, `muted()`, `rate()` | The element's state, each its own signal. |
| `duration()` | Seconds; `None` until the metadata loads, and for a live stream. |
| `buffering()` | Playing was asked for, but the data has not arrived yet. |
| `error()` | Why the source failed to load or play, if it did. |
| `is_supported()` | `false` until mounted, then whether anything plays media here. |

`MediaHandle` is `Copy`. On a WebView the time moves at most every 250 ms.

| Platform | Support |
|---|---|
| Web | Full. |
| Linux desktop (WebKitGTK) | Full: play, state and the time, checked by the e2e suite. |
| Android (WebView) | Full: play, state and the time, checked by the e2e suite on the emulator. |
| macOS, Windows desktop | Untested. |
| Blitz, server render | No media: commands do nothing and `is_supported()` stays `false`. |

## Accessibility

### Libero handles

- It renders nothing and announces nothing: your controls carry the names and
  states.
- Commands from your controls only: it never starts playing by itself.

### You must

- Give every control a name that says what it does now, such as Play or Pause.
- Announce buffering and errors yourself, from `buffering()` and `error()`, as
  the demo's status line and `Audio` and `Video` do.
- Never autoplay sound (WCAG 1.4.2); offer captions and a transcript as for any
  media.

### Limits

- Blitz and a server render play no media: every command does nothing and
  `is_supported()` stays false.
- On a WebView each command and state change crosses the IPC, so the time
  trails by a moment.
