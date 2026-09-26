# Audio

Crate: `libero`
Import: `use libero::components::Audio;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/audio.rs>
Index: [index.md](index.md) lists every other page
Description: An `<audio>` with libero's own controls, and the `use_media` hook behind them.

An `<audio>` with libero's own controls: play, seek, time, mute and volume, in
the theme's look on every platform that plays media.

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
| `preload` | `MediaPreload` | `metadata` | How much loads before a press: `none`, `metadata` or `auto`. |
| `size` | `Size` | theme | Of the buttons and sliders. |
| `onplay` | `EventHandler<()>` | `None` | Playing started. |
| `onpause` | `EventHandler<()>` | `None` | Playing paused. |
| `onended` | `EventHandler<()>` | `None` | Playing reached the end. |
| `onerror` | `EventHandler<MediaError>` | `None` | The source failed: `Aborted`, `Network`, `Decode` or `SourceNotSupported`. |
| `children` | `Element` | a sentence and a link | Shown instead of the controls where nothing plays media (Blitz). |

Like every component, `Audio` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `AudioPart::Controls` | `controls` | The row of controls, a `Toolbar`. |
| `AudioPart::Time` | `time` | The elapsed and total time. |
| `AudioPart::Seek` | `seek` | The seek slider's wrapper. |
| `AudioPart::Volume` | `volume` | The volume slider's wrapper. |
| `AudioPart::Message` | `message` | The error text, or the fallback where nothing plays media. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `K` | Plays or pauses, with focus anywhere in the player. |
| `Space` | On a slider: plays or pauses. On a button: presses it. |
| `J` or `L` | Jumps 10 seconds back or ahead. |
| `M` | Mutes or unmutes. |
| `Left` or `Right` | On the seek slider: 1 second; on the volume slider: 5%. |

### Libero handles

- The player is a `group` named by `label`; its buttons are one `toolbar` stop,
  each slider its own.
- The play and mute buttons change their names (Play/Pause, Mute/Unmute) rather
  than using `aria-pressed`.
- Below 22rem the volume slider hides and the mute button stays, so the row fits
  at 320px (WCAG 1.4.10).
- The seek slider's `aria-valuetext` reads "1:05 of 4:56" (the localization's
  `media.position`).
- A polite status says "Loading" while playing waits for data; a failed source
  shows an alert.
- No autoplay unless asked, and a debug warning for autoplay with sound (WCAG
  1.4.2).

### You must

- Give each player a `label` that says what plays.
- Offer a transcript for speech (WCAG 1.2.1): libero cannot write it.

### Limits

- Blitz plays no media: the controls give way to `children`, by default a link
  to the file.
- On a WebView each command and state change crosses the IPC, so the time
  trails by a moment.
