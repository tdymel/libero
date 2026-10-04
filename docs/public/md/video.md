# Video

Crate: `libero`
Import: `use libero::components::{MediaSource, MediaTrack, TrackKind, Video};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/video.rs>
Index: [index.md](index.md) lists every other page
Description: A `<video>` with libero's own controls, captions and fullscreen, on the `use_media` hook.

A `<video>` with libero's own controls: play, seek, time, mute, volume,
speed, captions and fullscreen, in a bar over the picture as YouTube's, in the
theme's look on every platform that plays media. `use_media()` drives the same engine for a
layout of your own: see [use_media](use_media.md).

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

The demo plays "Big Buck Bunny", (c) 2008 Blender Foundation,
<https://peach.blender.org>, under CC BY 3.0, streamed from Wikimedia Commons,
so it plays only online. Its English and German tracks are one placeholder
caption each, not the film's sound.

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
            tracks: vec![
                MediaTrack {
                    src: "/launch.en.vtt".into(),
                    kind: TrackKind::Captions,
                    srclang: "en".into(),
                    label: "English".into(),
                    default: false,
                },
                MediaTrack {
                    src: "/launch.de.vtt".into(),
                    kind: TrackKind::Subtitles,
                    srclang: "de".into(),
                    label: "Deutsch".into(),
                    default: false,
                },
            ],
        }
    }
}
```

A captions or subtitles track enables the captions button: it shows the
default one, else the first. Two or more make it a menu of Off and each track.
Without one the button stays, disabled. Pass a `use_media()` handle to `Video { media }` to drive
the player from outside.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `src` | `String` | required | The file's URL. |
| `sources` | `Vec<MediaSource>` | `[]` | The file in other formats, `MediaSource::new(src, mime)`, tried in order before `src`. `src` stays the fallback and the download link. |
| `label` | `String` | required | Names the player, e.g. the video's title. |
| `poster` | `Option<String>` | `None` | A picture shown until playing starts. |
| `aspect_ratio` | `Option<String>` | `16 / 9` | The picture's CSS `aspect-ratio`, which the box holds before the file loads, poster or not. A portrait clip wants `"9 / 16"`; `"auto"` follows the file, and the box jumps as it loads. A picture of another shape is letterboxed. |
| `tracks` | `Vec<MediaTrack>` | `[]` | WebVTT files: `src`, `kind` (`Captions`, `Subtitles`, `Descriptions`, `Chapters`), `srclang`, `label`, `default`. A captions or subtitles track enables the captions button; two or more make it a menu of them. |
| `chapters` | `Vec<Chapter>` | `[]` | `Chapter::new(start_seconds, title)`: splits the seek track into segments with gaps, names the chapter in the slider's value, shows the current one beside the time and adds a chapters menu. Wins over a `Chapters` track, which is otherwise fetched and read. |
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
| `VideoPart::Controls` | `controls` | The bar of controls over the bottom of the picture, a named `group`. |
| `VideoPart::Time` | `time` | The elapsed and total time. |
| `VideoPart::Chapter` | `chapter` | The current chapter's title beside the time, hidden below 22rem. |
| `VideoPart::Seek` | `seek` | The seek slider's wrapper. |
| `VideoPart::Volume` | `volume` | The mute button and the volume menu's trigger. |
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
| `C` | Shows or hides the captions, with a captions or subtitles track: the last one chosen in the track menu. |
| `F` | Enters or leaves fullscreen. |
| `Ctrl+Right`, `Ctrl+Left` | With chapters: the next chapter, or back to the start of this one (the previous one in its first 3 seconds). Swapped in a right-to-left page. |
| `Shift+?` | Lists these keys in a `ShortcutHelp` dialog, inside the player in fullscreen. |
| `Escape` | Closes the speed or volume menu and returns to its button; else leaves fullscreen. |
| `Left` or `Right` | On the seek slider: 1 second; on the volume slider: 5%. |
| `Tab` | Moves through every control in visual order: seek, play, mute, volume, captions, speed, fullscreen. In the volume menu, returns to its button. |

### Libero handles

- The player is a `group` named by `label`, its controls a `group` named
  "Player controls"; every button and slider is its own Tab stop, as in the
  browser's own controls.
- The play, mute and fullscreen buttons change their names (Play/Pause,
  Mute/Unmute, Fullscreen/Exit fullscreen); the captions button uses
  `aria-pressed`, and a bar under its icon shows it pressed.
- In fullscreen the speed and volume menus and the tooltips open inside the
  player, so they show over the fullscreen picture.
- A press anywhere on the seek track jumps there; a drag scrubs.
- The speed button shows the rate ("1×", "1,5×" under `Formats::GERMAN`) and is
  named "Playback speed 1×"; its menu offers 0.5× to 2× as radio items.
- Without a captions or subtitles track the captions button stays, disabled
  but focusable, and says "No captions for this video".
- With two or more captions or subtitles tracks the captions button opens a
  menu: Off and each track by its `label` (else its `srclang`), as radio items.
  With one it stays a toggle.
- The speaker button mutes and unmutes. The chevron beside it, named "Volume",
  opens a `dialog` holding the volume slider and focuses it, as `Audio`'s.
- At volume 0 the mute button offers Unmute, which brings back the last audible
  volume; moving the volume up while muted unmutes.
- Where the page may not go fullscreen, the player covers the window as a fixed
  box instead, which Escape, F and a Tab out of the player leave, so focus never
  hides behind it. The page behind it does not scroll, as under a modal. Focus
  stays on the control that was pressed, inside the box, and is there again when
  the box closes.
- The controls overlay the bottom of the picture, as YouTube's, and fade after 3
  seconds of playing untouched or as the mouse leaves the player; while paused
  they stay. A pointer move, a tap or a key brings them back. After a key they
  stay until the next click or tap, so keyboard focus never sits on a faded
  control, and focus moved into them by code or a screen reader shows them too.
  They never leave the Tab order.
- A click on the picture plays or pauses, as YouTube's. A tap on it while the
  controls are faded only shows them; once shown, a tap plays or pauses.
- No control leaves the player at any width: the seek track has a row of its own
  and shrinks with the player, below 22rem the total time goes, and below 15rem
  the bar moves under the picture and wraps, so it fits at 320px and 200% zoom
  (WCAG 1.4.10).
- A black scrim under the bar keeps its white text at 4.5:1 and its icons,
  tracks and thumbs at 3:1 over any picture, a white one included, in light and
  dark (WCAG 1.4.3, 1.4.11).
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
- In a parent that shrink-wraps its content, such as a flex column, give the
  parent `max-width: 100%` or `min-width: 0`: an unsized player asks for 40rem
  and would push it past a narrow screen (WCAG 1.4.10).

### Limits

- Blitz plays no media: the controls give way to `children`, by default a link
  to the file.
- On a WebView each command and state change crosses the IPC, so the time
  trails by a moment.
- The shown controls cover the bottom of the picture. Chromium-based browsers
  draw the captions above them, through the WebKit captions box Safari shares;
  Firefox offers no hook to move them, so there the bar covers them until it
  fades.
- Captions show once playing starts: browsers draw none over the poster.
- Fullscreen is libero's own `use_fullscreen` over the browser's Fullscreen
  API, no library; where the API is refused, a fixed box over the window. A
  floating mini player is the browser's own Picture-in-Picture (Firefox's
  button on the video, Chrome's context menu): its window is outside the page,
  so focus cannot follow it.
- The demo's VP9 WebM may not play in older Safari: give `sources` a WebM and
  keep an MP4 (H.264) as `src`.
