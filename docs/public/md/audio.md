# Audio

Crate: `libero`
Import: `use libero::components::Audio;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/audio.rs>
Index: [index.md](index.md) lists every other page
Description: An `<audio>` with libero's own controls, and the `use_media` hook behind them.

An `<audio>` in one compact row, as a chat app's voice message: play, a track
of bars to seek, the time, mute with a volume menu and a speed button, in the
theme's look on every platform that plays media.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Audio;

#[component]
fn Demo() -> Element {
    rsx! { Audio { src: "/episode-12.mp3", label: "Episode 12: Borrowing" } }
}
```

`use_media()` drives the same engine for a layout of your own. Spread its
`attributes()` and `mount()` on a bare `<audio>` or `<video>`, then call
`play()`, `pause()`, `toggle()`, `seek(seconds)`, `set_volume(0.0..=1.0)`,
`set_muted(bool)` and `set_rate(f64)`. Each read is its own signal, so a time
tick re-renders only what shows the time: `paused()`, `ended()`,
`current_time()`, `duration()`, `volume()`, `muted()`, `rate()`, `buffering()`,
`error()` and `is_supported()`.

```rust
use dioxus::prelude::*;
use libero::hooks::use_media;

#[component]
fn Demo() -> Element {
    let media = use_media();
    rsx! {
        audio { src: "/podcast.mp3", onmounted: media.mount(), ..media.attributes() }
        button {
            onclick: move |_| media.toggle(),
            if media.paused() { "Play" } else { "Pause" }
        }
    }
}
```

Pass the handle to `Audio { media }` to drive the built-in player from outside.

## Your own layout

[`use_media()`](use_media.md) drives the same engine for a layout of your own:
spread its `attributes()` and `mount()` on a bare element and read `paused()`,
`current_time()` and friends.

## Formats

A browser plays the first `sources` entry whose type it supports, then `src`.
An Ogg first and an MP3 as `src` reach every browser, Safari included.

```rust
use dioxus::prelude::*;
use libero::components::{Audio, MediaSource};

#[component]
fn Demo() -> Element {
    rsx! {
        Audio {
            src: "/message.mp3",
            sources: vec![MediaSource::new("/message.ogg", "audio/ogg")],
            label: "Voice message",
        }
    }
}
```

## The demo audio

The demo plays "Wikipedia guitar solo" (CC0), streamed from Wikimedia Commons,
so it plays only online.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The file's URL. |
| `sources` | `Vec<MediaSource>` | `[]` | The file in other formats, `MediaSource::new(src, mime)`, tried in order before `src`. `src` stays the fallback and the download link. |
| `label` | `String` | required | Names the player, e.g. the track's title. |
| `media` | `Option<MediaHandle>` | `None` | A handle from `use_media()`, to drive or read the player from outside. |
| `autoplay` | `bool` | `false` | Starts on load. Browsers refuse it with sound: pair it with `muted`; a debug build warns otherwise. |
| `muted` | `bool` | `false` | Starts muted. |
| `looping` | `bool` | `false` | Starts again at the end. |
| `preload` | `MediaPreload` | `metadata` | How much loads before a press: `none`, `metadata` or `auto`. Unless `none`, a file up to 10 minutes is fetched whole once more to draw the bars, unless its `Content-Length` is over 20 MB. |
| `size` | `Size` | theme | Of the buttons and the seek slider. |
| `onplay` | `EventHandler<()>` | `None` | Playing started. |
| `onpause` | `EventHandler<()>` | `None` | Playing paused. |
| `onended` | `EventHandler<()>` | `None` | Playing reached the end. |
| `onerror` | `EventHandler<MediaError>` | `None` | The source failed: `Aborted`, `Network`, `Decode` or `SourceNotSupported`. |
| `volume_parts` | `Parts<VolumePart>` | - | Styles the portaled volume menu and its slider. |
| `children` | `Element` | a sentence and a link | Shown instead of the controls where nothing plays media (Blitz). |

Like every component, `Audio` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `AudioPart::Controls` | `controls` | The row of controls, a named `group`. |
| `AudioPart::Time` | `time` | The time: the total until playing starts, then the elapsed. |
| `AudioPart::Seek` | `seek` | The seek slider's wrapper; its `SliderTrack::Bars` track draws the bars. |
| `AudioPart::Volume` | `volume` | The mute button and the volume menu's trigger. |
| `AudioPart::Message` | `message` | The error text, or the fallback where nothing plays media. |

### Volume menu

The volume menu is portaled out of the player, so its parts take the
`volume_parts` prop.

| Part | `data-slot` | Description |
|---|---|---|
| `VolumePart::Card` | `volume-card` | The card the volume chevron opens. |
| `VolumePart::Slider` | `volume-slider` | The volume slider's wrapper on the card, `8rem` wide. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `K` | Plays or pauses, with focus anywhere in the player. |
| `Space` | On a slider: plays or pauses. On a button: presses it. |
| `J` or `L` | Jumps 10 seconds back or ahead. |
| `M` | Mutes or unmutes. |
| `Shift+?` | Lists these keys in a `ShortcutHelp` dialog. |
| `Left` or `Right` | On the seek slider: 1 second; on the volume slider: 5%. |
| `Escape` | Closes the volume menu and returns to its button. |
| `Tab` | Moves through every control in visual order: play, seek, mute, volume, speed. In the volume menu, returns to its button. |

### Libero handles

- The player is a group named by `label`. Each button and slider is its own
  Tab stop.
- Buttons say what a press does: Play or Pause, Mute or Unmute. The speed
  button reads "Playback speed 1×" and steps to 1.5×, 2× and back to 1×.
- The seek slider reads the time as "1:05 of 4:56". The visible time is hidden
  from screen readers, so it is not read twice.
- The chevron beside the speaker, named "Volume", opens the volume slider and
  focuses it. Raising the volume unmutes.
- The loudness bars on the seek track are hidden from screen readers; the
  slider's keys, press and drag work on them.
- Every control fits at 320px wide and 200% zoom: the track shrinks first, and
  no button gets smaller than 24px (WCAG 1.4.10, 2.5.8).
- The volume bubble's border has 3:1 contrast in light and dark (WCAG 1.4.11).
- A polite "Loading" is announced while the sound waits for data, and an alert
  when the file fails.
- It never plays by itself unless you set `autoplay`; autoplay with sound
  warns in a debug build (WCAG 1.4.2).

### You must

- Give each player a `label` that says what plays.
- Offer a transcript for speech (WCAG 1.2.1): libero cannot write it.

### Example

A podcast episode: `Audio { src: "/episode-12.mp3", label: "Episode 12:
Accessible forms" }` with a transcript link under it. A screen reader
announces the group "Episode 12: Accessible forms", and a deaf user reads the
transcript instead.

### Limits

- Blitz plays no media: the controls give way to `children`, by default a link
  to the file.
- On a WebView each command and state change crosses the IPC, so the time
  trails by a moment.
- The bars need the file readable by the page: a file on another origin
  without CORS, or `preload: none`, keeps placeholder bars drawn from the URL.
  So does a file over 10 minutes or 20 MB, which is not decoded.
