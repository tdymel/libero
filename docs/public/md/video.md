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

## Formats

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

## The demo video

"Big Buck Bunny", (c) 2008 Blender Foundation, <https://peach.blender.org>,
under CC BY 3.0, streamed from Wikimedia Commons, so it plays only online. Its
English and German tracks are one placeholder caption each, not the film's
sound.

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
| `volume_parts` | `Parts<VolumePart>` | - | Styles the portaled volume menu and its slider. |
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

### Volume menu

The volume menu is portaled out of the player, so its parts take the
`volume_parts` prop.

| Part | `data-slot` | Description |
|---|---|---|
| `VolumePart::Card` | `volume-card` | The card the volume chevron opens. |
| `VolumePart::Slider` | `volume-slider` | The volume slider's wrapper on the card, `8rem` wide. |

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

- The player is a `group` named by `label`, its bar a `group` named "Player
  controls"; every button and slider is its own Tab stop.
- The play, mute and fullscreen buttons change their names (Play/Pause,
  Mute/Unmute, Fullscreen/Exit fullscreen); the captions toggle uses
  `aria-pressed`.
- With two or more captions or subtitles tracks, the captions button opens a
  menu of Off and each track by its `label` (else its `srclang`), as radio
  items. Without one it stays, disabled but focusable, and says "No captions
  for this video".
- The speed button shows the rate and is named "Playback speed 1×" ("1,5×"
  under `Formats::GERMAN`); its menu offers 0.5× to 2× as radio items.
- The speaker button mutes; the chevron beside it, named "Volume", opens a
  `dialog` with the volume slider focused. At volume 0, Unmute brings back the
  last audible volume.
- A press anywhere on the seek track jumps there; a drag scrubs.
- The seek slider's `aria-valuetext` reads "1:05 of 4:56" (the localization's
  `media.position`), with chapters "1:05 of 4:56, The plan"
  (`slider.segment`); the bubble shows the same.
- With chapters a menu button, named "Chapters", lists each start and title as
  radio items, the current one checked; picking one seeks there. It is the way
  to a chapter on touch and without the chord keys.
- In fullscreen the menus and tooltips open inside the player. Where the page
  may not go fullscreen, the player covers the window instead, the page behind
  stays still, and Escape, F or a Tab out leaves it; focus stays on the
  pressed control.
- The bar overlays the bottom of the picture and fades after 3 seconds of
  untouched play or as the mouse leaves; paused, it stays. A pointer move, a
  tap, a key or focus moved into it brings it back, and after a key it stays
  until the next click or tap. It never leaves the Tab order.
- A click on the picture plays or pauses. A tap while the bar is faded only
  shows it.
- No control leaves the player at any width: below 22rem the total time goes,
  below 15rem the bar moves under the picture and wraps, so it fits at 320px
  and 200% zoom (WCAG 1.4.10).
- A black scrim under the bar keeps its text at 4.5:1 and its icons and tracks
  at 3:1 over any picture, in light and dark (WCAG 1.4.3, 1.4.11).
- A polite status says "Loading" while playing waits for data; a failed source
  shows an alert.
- No autoplay unless asked, and a debug warning for autoplay with sound (WCAG
  1.4.2).

### You must

- Give each player a `label` that says what plays.
- Add a captions track for speech (WCAG 1.2.2), and a described version or a
  transcript for what only the picture shows (WCAG 1.2.3, 1.2.5): libero
  cannot write them.
- Do not let a clip flash more than three times a second (WCAG 2.3.1).
- In a parent that shrink-wraps its content, such as a flex column, give the
  parent `max-width: 100%` or `min-width: 0`: an unsized player asks for 40rem
  and would push it past a narrow screen (WCAG 1.4.10).

### Example

The demo's player has `label: "Big Buck Bunny"` and four chapters. Tab reaches
the seek slider; Ctrl+Right jumps to 1:55, and the slider's value ends in "The
bullies". K pauses, C shows the English captions, and Shift+? lists every key.

### Limits

- Blitz plays no media: the controls give way to `children`, by default a link
  to the file.
- On a WebView each command and state change crosses the IPC, so the time
  trails by a moment.
- The seek slider moves in whole seconds, so a chapter starting at 12.4 s is
  named from 13 s in the slider's value; the label beside the time follows the
  exact time. A chapters track on another origin needs CORS.
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
