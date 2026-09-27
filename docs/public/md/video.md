# Video

Crate: `libero`
Import: `use libero::components::{MediaSource, MediaTrack, TrackKind, Video};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/video.rs>
Index: [index.md](index.md) lists every other page
Description: A `<video>` with libero's own controls, captions and fullscreen, on the `use_media` hook.

A `<video>` with libero's own controls: play, seek, time, mute, volume,
speed, captions and fullscreen, the same row as `Audio`'s, in the theme's look on
every platform that plays media. `use_media()` drives the same engine for a
layout of your own: see [use_media](use_media.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{MediaTrack, TrackKind, Video};

#[component]
fn Demo() -> Element {
    rsx! {
        Video {
            src: "/launch.webm",
            label: "Launch day",
            poster: "/launch.jpg",
            aspect_ratio: "16 / 9",
            tracks: vec![MediaTrack {
                src: "/launch.en.vtt".into(),
                kind: TrackKind::Captions,
                srclang: "en".into(),
                label: "English".into(),
                default: false,
            }],
        }
    }
}
```

A captions or subtitles track enables the captions button: it shows the
default one, else the first. Without one the button stays, disabled. Pass a `use_media()` handle to `Video { media }` to drive
the player from outside.

### Several formats

A browser plays the first `sources` entry whose type it supports, then `src`.
WebM first and an MP4 as `src` reach every browser, Safari included.

```rust
use dioxus::prelude::*;
use libero::components::{MediaSource, Video};

#[component]
fn Demo() -> Element {
    rsx! {
        Video {
            src: "/launch.mp4",
            sources: vec![MediaSource::new("/launch.webm", "video/webm")],
            label: "Launch day",
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The file's URL. |
| `sources` | `Vec<MediaSource>` | `[]` | The file in other formats, `MediaSource::new(src, mime)`, tried in order before `src`. `src` stays the fallback and the download link. |
| `label` | `String` | required | Names the player, e.g. the video's title. |
| `poster` | `Option<String>` | `None` | A picture shown until playing starts. |
| `aspect_ratio` | `Option<String>` | `None` | The picture's CSS `aspect-ratio`, such as `"16 / 9"`, so the box holds its shape before the file loads. Unset, the file's own. |
| `tracks` | `Vec<MediaTrack>` | `[]` | WebVTT files: `src`, `kind` (`Captions`, `Subtitles`, `Descriptions`, `Chapters`), `srclang`, `label`, `default`. A captions or subtitles track enables the captions button. |
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

Like every component, `Video` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `VideoPart::Media` | `media` | The `<video>` element. |
| `VideoPart::Controls` | `controls` | The row of controls, a named `group`. |
| `VideoPart::Time` | `time` | The elapsed and total time. |
| `VideoPart::Seek` | `seek` | The seek slider's wrapper. |
| `VideoPart::Volume` | `volume` | The volume slider's wrapper. |
| `VideoPart::Message` | `message` | The error text, or the fallback where nothing plays media. |

The player carries `data-fullscreen="native"` or `"drawn"` while it fills the
screen, from [use_fullscreen](use_fullscreen.md).

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `K` | Plays or pauses, with focus anywhere in the player. |
| `Space` | On a slider: plays or pauses. On a button: presses it. |
| `J` or `L` | Jumps 10 seconds back or ahead. |
| `M` | Mutes or unmutes. |
| `C` | Shows or hides the captions, with a captions or subtitles track. |
| `F` | Enters or leaves fullscreen. |
| `Escape` | Leaves fullscreen. |
| `Left` or `Right` | On the seek slider: 1 second; on the volume slider: 5%. |
| `Tab` | Moves through every control in visual order: play, seek, mute, volume, speed, captions, fullscreen. |

### Libero handles

- The player is a `group` named by `label`, its controls a `group` named
  "Player controls"; every button and slider is its own Tab stop, as in the
  browser's own controls.
- The play, mute and fullscreen buttons change their names (Play/Pause,
  Mute/Unmute, Fullscreen/Exit fullscreen); the captions button uses
  `aria-pressed`.
- The speed button shows the rate ("1×") and is named "Playback speed 1×";
  its menu offers 0.5× to 2× as radio items.
- Without a captions or subtitles track the captions button stays, disabled
  but focusable, and says "No captions for this video".
- At volume 0 the mute button offers Unmute, which brings back the last audible
  volume; moving the volume up while muted unmutes.
- Where the page may not go fullscreen, the player covers the window as a fixed
  box instead, which Escape, F and a Tab out of the player leave, so focus never
  hides behind it. Focus stays on the control that was pressed, inside the box,
  and is there again when the box closes.
- In fullscreen the controls overlay the bottom of the picture and fade after 3
  seconds of playing untouched. A pointer move, a tap or a key brings them
  back. After a key they stay until the next click or tap, so keyboard focus
  never sits on a faded control; they never leave the Tab order, and a tap on
  the picture only shows them.
- No control leaves the player at any width: the seek track shrinks first,
  below 28rem the volume slider hides (mute stays), below 22rem the total time,
  and only then does the row wrap, so it fits at 320px and 200% zoom (WCAG
  1.4.10).
- The controls bar has a border at 3:1 against the page in light and dark
  (WCAG 1.4.11).
- The seek slider's `aria-valuetext` reads "1:05 of 4:56" (the localization's
  `media.position`).
- A polite status says "Loading" while playing waits for data; a failed source
  shows an alert.
- No autoplay unless asked, and a debug warning for autoplay with sound (WCAG
  1.4.2).

### You must

- Give each player a `label` that says what plays.
- Add a captions track for speech (WCAG 1.2.2), and a described version or a
  transcript for what only the picture shows (WCAG 1.2.3, 1.2.5): libero cannot
  write them.
- Do not let a clip flash more than three times a second (WCAG 2.3.1).

### Limits

- Blitz plays no media: the controls give way to `children`, by default a link
  to the file.
- On a WebView each command and state change crosses the IPC, so the time
  trails by a moment.
- In fullscreen the shown controls cover the bottom of the picture, captions
  included.
- Captions show once playing starts: browsers draw none over the poster.
- Fullscreen is libero's own `use_fullscreen` over the browser's Fullscreen
  API, no library; where the API is refused, a fixed box over the window. A
  floating mini player is the browser's own Picture-in-Picture (Firefox's
  button on the video, Chrome's context menu): its window is outside the page,
  so focus cannot follow it.
- The demo's VP9 WebM may not play in older Safari: give `sources` a WebM and
  keep an MP4 (H.264) as `src`.
