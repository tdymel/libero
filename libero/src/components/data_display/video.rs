use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

use super::media_controls::{
    Captions, MediaControls, MediaFallback, SPEED, VolumePart, chapter_jump, notify,
    use_media_keys, use_media_start, use_sound,
};
use super::{
    MediaPreload, MediaSource,
    audio::{media_sources, use_source_reload},
};
use crate::{
    components::{
        common::{
            HtmlTag, Input, Part, Parts, base_props, media_url, parts_enum, use_name_warning,
        },
        layout::use_box,
        overlay::Shortcut,
    },
    context::{HostOutlet, PortalHost},
    css::declaration_value,
    hooks::{
        FULLSCREEN_ATTR, Hotkey, MediaError, MediaHandle, TimeoutHandle, listener, use_cache,
        use_element, use_fullscreen, use_localization, use_media, use_portal_slot, use_scroll_lock,
        use_timeout,
    },
    platform,
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, sx},
    theme::{ACTION_ICON_SIZE, BUTTON_HEIGHT, ColorCss, ColorShade, Size, SizeCss, Z_INDEX_MODAL},
    utils::warn,
};

parts_enum! {
    /// [`Video`]'s inner parts, for its `parts` prop.
    pub enum VideoPart {
        /// The `<video>` element.
        Media = "media" => "& > [data-slot='media']",
        /// The bar of controls over the bottom of the picture.
        Controls = "controls" => "& > [data-slot='controls']",
        /// The elapsed and total time.
        Time = "time" => "& [data-slot='time']",
        /// The current chapter's title, beside the time.
        Chapter = "chapter" => "& [data-slot='chapter']",
        /// The seek slider's wrapper.
        Seek = "seek" => "& [data-slot='seek']",
        /// The mute button and the volume menu's trigger.
        Volume = "volume" => "& [data-slot='volume']",
        /// The text shown when the source fails, or nothing plays media.
        Message = "message" => "& > [data-slot='message']",
    }
}

const NEXT_CHAPTER: &str = "ctrl+ArrowRight";
const PREVIOUS_CHAPTER: &str = "ctrl+ArrowLeft";
/// Set on the player while its controls are faded out.
const CONTROLS_ATTR: &str = "data-controls";
/// How long the controls stay after the last pointer move or key while playing.
const CONTROLS_IDLE_MS: u64 = 3000;
const PLAYER_CONTAINER: &str = "libero-video";
const CONTROLS_CONTAINER: &str = "libero-video-controls";
const NARROWEST: &str = "(max-width: 22rem)";
/// Too small a picture to lay the wrapped bar over: it moves below.
const TINY: &str = "(max-width: 15rem)";
/// How far the scrim fades out above the controls.
const SCRIM_FADE: &str = "2rem";
/// Letterboxing and the bar's scrim stay black in either scheme, as every player draws them.
const BLACK: &str = "#000";
/// The scrim's floor: 3:1 for the light tracks and 4.5:1 for the white text even over a white frame.
const SCRIM: &str = "rgba(0, 0, 0, 0.8)";
/// A hovered, pressed or open button on the scrim: white text stays above 6:1 over a white frame.
const STATE_TINT: &str = "rgba(255, 255, 255, 0.2)";
const DEFAULT_RATIO: &str = "16 / 9";
/// On the `<video>`: how far its captions move up to clear the shown bar.
const BAR_VAR: &str = "--libero-video-bar";
/// Blink and WebKit's box of the captions; Firefox has none to move.
const CUES: &str = "::-webkit-media-text-track-container";
/// Light tracks, as YouTube's: the theme's muted one sinks into the scrim.
const LIGHT_TRACK: &str = "rgba(255, 255, 255, 0.5)";
const LIGHT_FILL: &str = "#fff";

/// A direct-child `part` of the player while its controls are faded out.
fn when_faded(part: VideoPart) -> String {
    let child = part.selector().trim_start_matches("& ");
    format!("&[{CONTROLS_ATTR}='hidden'] {child}")
}

static VIDEO_SX: StaticSx = StaticSx::new(|| {
    let controls = VideoPart::Controls.selector();
    let media = VideoPart::Media.selector();
    let filled = sx()
        .background_color(BLACK)
        .selector(media, sx().height("100%").aspect_ratio("auto"));
    let light_track = || sx().background(LIGHT_TRACK);
    sx().position("relative")
        // A size container has no content width: in a shrink-wrapping parent it collapsed (todo 1369).
        .width("100%")
        .with("contain-intrinsic-inline-size", "40rem")
        .min_width("0")
        .container(PLAYER_CONTAINER)
        .selector(
            media,
            sx().display("block")
                .width("100%")
                // Before the file loads too, poster or none: no jump from the browser's 2:1 (todo 1369).
                .aspect_ratio(DEFAULT_RATIO)
                .background_color(BLACK)
                .object_fit("contain")
                // Forced colours repaint the black as Canvas, which leaves the picture's box unmarked.
                .media(
                    FORCED_COLORS,
                    sx().outline("1px solid CanvasText").outline_offset("-1px"),
                )
                // The captions sit above the shown bar, as YouTube's.
                .selector(
                    format!("&{CUES}"),
                    sx().transform(format!("translateY(calc(-1 * var({BAR_VAR})))"))
                        .transition("transform 200ms ease")
                        .media(REDUCED_MOTION, sx().transition("none")),
                ),
        )
        .selector(
            format!("{}{CUES}", when_faded(VideoPart::Media)),
            sx().transform("none"),
        )
        // YouTube's bar: over the bottom of the picture on a black scrim, white
        // text and icons, the seek track a row of its own above the buttons.
        .selector(
            controls,
            sx().position("absolute")
                .inset("auto 0 0 0")
                .display("flex")
                .flex_wrap("wrap")
                .align_items("center")
                .gap(SizeCss::SPACING.value(Size::Xs))
                .padding(SizeCss::SPACING.value(Size::Xs))
                .padding_top(SCRIM_FADE)
                .color("#fff")
                .background_image(format!(
                    "linear-gradient(to top, {SCRIM} calc(100% - {SCRIM_FADE}), transparent)"
                ))
                .container(CONTROLS_CONTAINER)
                .transition("opacity 200ms ease")
                .media(REDUCED_MOTION, sx().transition("none"))
                .selector("& > *", sx().flex_shrink("0"))
                // A narrow bar fits the chapters button too: wrapping happens before any shrinking.
                // The speed label is text: it keeps its width, at least a button's.
                .selector(
                    "& button",
                    sx().container_query(CONTROLS_CONTAINER, NARROWEST, sx().max_height("2.25rem")),
                )
                .selector(
                    format!("& button:not([data-slot='{SPEED}'])"),
                    sx().container_query(CONTROLS_CONTAINER, NARROWEST, sx().max_width("2.25rem")),
                )
                .selector(
                    format!("& button[data-slot='{SPEED}']"),
                    sx().container_query(CONTROLS_CONTAINER, NARROWEST, sx().min_width("2.25rem")),
                )
                .selector("& [data-slot='track']", light_track())
                // Chapters: the gaps between the light segments show the scrim.
                .selector(
                    "& [data-state~='segments'] > [data-slot='track']",
                    sx().background("transparent"),
                )
                .selector("& [data-slot='segment']", light_track())
                // The Slider's fill is in the text role, dark on a light page: white on the scrim.
                .selector(
                    "& [data-slot='bar'], & [data-slot='segment-fill']",
                    sx().background(LIGHT_FILL)
                        .media(FORCED_COLORS, sx().background("Highlight")),
                )
                .selector("& [data-slot='thumb']", sx().border_color(LIGHT_FILL))
                // The theme's light state tint left the white text under 3:1 (todo 1388).
                .selector(
                    "& button:is(:hover, :active, [aria-expanded='true']):not(:disabled)",
                    sx().background(STATE_TINT).color("#fff"),
                )
                // Shown captions: a bar under the icon, as YouTube's; a border survives forced colours.
                .selector(
                    "& button[data-captions]",
                    sx().position("relative").selector(
                        "& > [data-mark]",
                        sx().display("none")
                            .position("absolute")
                            .inset("auto 25% 3px")
                            .border_bottom("3px solid currentColor")
                            .border_radius("2px"),
                    ),
                )
                .selector(
                    "& button[data-captions='on'] > [data-mark]",
                    sx().display("block"),
                ),
        )
        // On a tiny player the bar would wrap up past the picture's top, so it sits below.
        .container_query(
            PLAYER_CONTAINER,
            TINY,
            sx().selector(
                controls,
                sx().position("static")
                    .padding_top(SizeCss::SPACING.value(Size::Xs))
                    .background_image("none")
                    .background_color(BLACK),
            )
            .selector(format!("{media}{CUES}"), sx().transform("none")),
        )
        // The seek track takes its row, so it shrinks with the player; below 22rem the total time goes.
        .selector(
            VideoPart::Time.selector(),
            sx().font_variant_numeric("tabular-nums")
                .white_space("nowrap")
                .padding_inline(SizeCss::SPACING.value(Size::Xs))
                // Pushes captions, speed and fullscreen to the end.
                .margin_inline_end("auto")
                .selector(
                    "& > span",
                    sx().container_query(CONTROLS_CONTAINER, NARROWEST, sx().display("none")),
                ),
        )
        // Grows from nothing, so it never wraps and takes the room the time's margin would.
        .selector(
            VideoPart::Chapter.selector(),
            sx().flex("1 1 0")
                .min_width("0")
                .overflow("hidden")
                .text_overflow("ellipsis")
                .white_space("nowrap")
                .container_query(CONTROLS_CONTAINER, NARROWEST, sx().display("none")),
        )
        .selector(
            VideoPart::Seek.selector(),
            sx().flex("1 0 100%").min_width("0"),
        )
        .selector(
            VideoPart::Volume.selector(),
            sx().display("flex").align_items("center"),
        )
        .selector(
            VideoPart::Message.selector(),
            sx().margin_top(SizeCss::SPACING.value(Size::Xs))
                .color(ColorCss::MUTED.value(ColorShade::S7)),
        )
        .selector("&:fullscreen", filled.clone())
        .selector(
            format!("&[{FULLSCREEN_ATTR}='drawn']"),
            filled
                .position("fixed")
                .inset("0")
                .z_index(Z_INDEX_MODAL.value()),
        )
        .selector(format!("&[{CONTROLS_ATTR}='hidden']"), sx().cursor("none"))
        .selector(
            when_faded(VideoPart::Controls),
            // A tap on the bar then reaches the picture, which only shows the controls.
            sx().opacity("0").pointer_events("none"),
        )
});

/// What a [`MediaTrack`] holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TrackKind {
    /// Speech and sounds, for viewers who cannot hear them.
    #[default]
    Captions,
    /// Speech in another language.
    Subtitles,
    /// The picture, in words, for viewers who cannot see it.
    Descriptions,
    /// Chapter titles.
    Chapters,
}

impl TrackKind {
    fn attribute(self) -> &'static str {
        match self {
            Self::Captions => "captions",
            Self::Subtitles => "subtitles",
            Self::Descriptions => "descriptions",
            Self::Chapters => "chapters",
        }
    }

    /// Drawn over the picture, so the captions button toggles it.
    fn shows_text(self) -> bool {
        matches!(self, Self::Captions | Self::Subtitles)
    }
}

/// A stretch of a [`Video`], from `start` (in seconds) to the next chapter's.
///
/// ```rust
/// # use libero::components::Chapter;
/// let chapters = vec![Chapter::new(0.0, "Intro"), Chapter::new(42.0, "Setup")];
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Chapter {
    pub start: f64,
    pub title: String,
}

impl Chapter {
    pub fn new(start: f64, title: impl Into<String>) -> Self {
        Self {
            start,
            title: title.into(),
        }
    }

    /// The cues of a WebVTT chapters file, each its start and text; a cue
    /// without a readable start or text is skipped.
    pub(crate) fn parse_vtt(text: &str) -> Vec<Self> {
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        text.split("\n\n")
            .filter_map(|block| {
                let mut lines = block.lines().skip_while(|line| !line.contains("-->"));
                let start = lines.next()?.split("-->").next().and_then(vtt_time)?;
                let title = lines.map(cue_text).collect::<Vec<_>>().join(" ");
                let title = title.trim();
                (!title.is_empty()).then(|| Self::new(start, title))
            })
            .collect()
    }
}

/// `mm:ss.ttt` or `hh:mm:ss.ttt`, in seconds.
fn vtt_time(text: &str) -> Option<f64> {
    let (clock, fraction) = text.trim().split_once('.').unwrap_or((text.trim(), "0"));
    let mut seconds = 0.0;
    for part in clock.split(':') {
        seconds = seconds * 60.0 + part.parse::<u32>().ok()? as f64;
    }
    let fraction = format!("0.{fraction}").parse::<f64>().ok()?;
    (clock.split(':').count() >= 2).then_some(seconds + fraction)
}

/// A cue line without its tags, entities decoded.
fn cue_text(line: &str) -> String {
    let mut text = String::with_capacity(line.len());
    let mut in_tag = false;
    for character in line.chars() {
        match character {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", "\u{a0}")
        .replace("&lrm;", "\u{200e}")
        .replace("&rlm;", "\u{200f}")
        .replace("&amp;", "&")
}

/// In order, one per start, none before 0 or not a number.
fn sorted_chapters(mut chapters: Vec<Chapter>) -> Vec<Chapter> {
    chapters.retain(|chapter| chapter.start.is_finite() && chapter.start >= 0.0);
    chapters.sort_by(|a, b| a.start.total_cmp(&b.start));
    chapters.dedup_by(|later, earlier| later.start == earlier.start);
    chapters
}

/// A WebVTT file for a [`Video`], rendered as its `<track>`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaTrack {
    /// The `.vtt` file's URL.
    pub src: String,
    pub kind: TrackKind,
    /// Its language, such as `"en"`.
    pub srclang: String,
    /// Its name in a track menu, such as `"English"`.
    pub label: String,
    /// Shown from the start.
    pub default: bool,
}

impl MediaTrack {
    /// Its menu item: the label, else the language, else `fallback` and its 1-based `position`.
    fn menu_name(&self, fallback: &str, position: usize) -> String {
        [self.label.trim(), self.srclang.trim()]
            .into_iter()
            .find(|name| !name.is_empty())
            .map_or_else(|| format!("{fallback} {position}"), str::to_string)
    }
}

/// The track the captions button shows: the one last picked in the track menu,
/// else the default one, else the first.
fn caption_track(picked: Option<usize>, text_tracks: &[(usize, &MediaTrack)]) -> Option<usize> {
    picked
        .filter(|picked| text_tracks.iter().any(|(index, _)| index == picked))
        .or_else(|| {
            text_tracks
                .iter()
                .find(|(_, t)| t.default)
                .or(text_tracks.first())
                .map(|(index, _)| *index)
        })
}

/// Two or more text tracks: the captions button opens a menu of them (todo 1284).
fn caption_menu(text_tracks: &[(usize, &MediaTrack)], fallback: &str) -> Vec<(usize, String)> {
    if text_tracks.len() < 2 {
        return Vec::new();
    }
    text_tracks
        .iter()
        .enumerate()
        .map(|(n, (index, t))| (*index, t.menu_name(fallback, n + 1)))
        .collect()
}

/// The chapters track is read only when the prop lists none.
fn chapters_src(tracks: &[&MediaTrack], listed: bool) -> Option<String> {
    tracks
        .iter()
        .filter(|track| track.kind == TrackKind::Chapters)
        .min_by_key(|track| !track.default)
        .map(|track| track.src.clone())
        .filter(|_| !listed)
}

/// The chapters read from `src`; a late answer for an old `src` is dropped.
fn read_chapters(read: Option<(String, Vec<Chapter>)>, src: &str) -> Vec<Chapter> {
    read.filter(|(read_src, _)| read_src == src)
        .map(|(_, chapters)| chapters)
        .unwrap_or_default()
}

/// The bar's height over the picture, bar the scrim's fade: measured (todo 1389), and
/// until then a button row, a 1rem seek row and the gaps.
fn bar_style(bar: Option<f64>, size: Option<Size>, aspect_ratio: Option<&str>) -> String {
    let bar = match bar {
        Some(height) => format!("{BAR_VAR}: calc({height}px - {SCRIM_FADE});"),
        None => {
            let button = match size {
                Some(size) => BUTTON_HEIGHT.value(size),
                None => ACTION_ICON_SIZE.value(),
            };
            format!(
                "{BAR_VAR}: calc({button} + 1rem + 3 * {});",
                SizeCss::SPACING.value(Size::Xs)
            )
        }
    };
    match aspect_ratio.and_then(|ratio| declaration_value("aspect-ratio", ratio)) {
        Some(ratio) => format!("{bar}aspect-ratio: {ratio};"),
        None => bar,
    }
}

/// The controls overlay the picture and fade out while playing untouched, or as a mouse
/// leaves; a pointer or a key brings them back. A key keeps them until the next press of a
/// pointer, so keyboard focus never sits on an unseen control.
#[derive(Clone, Copy)]
struct ControlsIdle {
    idle: Signal<bool>,
    keyboard: Signal<bool>,
    timer: TimeoutHandle,
}

fn use_controls_idle(playing: bool) -> ControlsIdle {
    let mut idle = use_signal(|| false);
    let keyboard = use_signal(|| false);
    let timer = use_timeout(move || idle.set(true), CONTROLS_IDLE_MS);
    let controls = ControlsIdle {
        idle,
        keyboard,
        timer,
    };
    use_effect(use_reactive!(|playing| {
        if playing {
            controls.wake();
        }
    }));
    controls
}

impl ControlsIdle {
    fn wake(mut self) {
        if *self.idle.peek() {
            self.idle.set(false);
        }
        self.timer.start();
    }

    fn by_key(mut self) {
        if !*self.keyboard.peek() {
            self.keyboard.set(true);
        }
        self.wake();
    }

    fn by_pointer(mut self) {
        if *self.keyboard.peek() {
            self.keyboard.set(false);
        }
    }

    fn mouse_left(mut self) {
        self.timer.stop();
        self.idle.set(true);
    }

    fn hidden(self, playing: bool, errored: bool) -> bool {
        playing && (self.idle)() && !(self.keyboard)() && !errored
    }
}

base_props! {
    parts(VideoPart);
    pub struct VideoProps {
        /// The file's URL.
        #[props(into)]
        src: String,
        /// The file in other formats, tried in order before `src`, such as WebM
        /// then MP4 for Safari.
        #[props(default)]
        sources: Vec<MediaSource>,
        /// Names the player, e.g. the video's title.
        #[props(into)]
        label: String,
        /// A picture shown until playing starts.
        #[props(default, into)]
        poster: Option<String>,
        /// The picture's CSS `aspect-ratio`. Unset, `16 / 9`, the file letterboxed in it.
        /// Portrait clips: pass their ratio, or `"auto"` (the box then jumps as the file loads).
        #[props(default, into)]
        aspect_ratio: Option<String>,
        /// Captions, subtitles and more, as WebVTT files. A `Chapters` one is read
        /// for the chapters, unless `chapters` lists them.
        #[props(default)]
        tracks: Vec<MediaTrack>,
        /// Splits the seek track into chapters, named in its value, beside the time
        /// and in a chapters menu. Wins over a `Chapters` track.
        #[props(default)]
        chapters: Vec<Chapter>,
        /// Your own handle, to drive or read the player from outside.
        #[props(default)]
        media: Option<MediaHandle>,
        /// Starts on load. Browsers refuse it with sound, so pair it with `muted`.
        #[props(default)]
        autoplay: bool,
        /// Starts muted.
        #[props(default)]
        muted: bool,
        /// Starts again at the end.
        #[props(default)]
        looping: bool,
        /// How much the browser fetches before a press.
        #[props(default)]
        preload: MediaPreload,
        /// Of the buttons and sliders.
        #[props(default, into)]
        size: Input<Size>,
        /// Called when playing starts or resumes.
        #[props(default)]
        onplay: Option<EventHandler<()>>,
        /// Called when playing pauses.
        #[props(default)]
        onpause: Option<EventHandler<()>>,
        /// Called when playing reaches the end.
        #[props(default)]
        onended: Option<EventHandler<()>>,
        /// Called when the file fails to load or play.
        #[props(default)]
        onerror: Option<EventHandler<MediaError>>,
        /// Styles the portaled volume menu and its slider.
        #[props(default, into)]
        volume_parts: Input<Parts<VolumePart>>,
        /// Shown instead of the controls where nothing plays media (Blitz). Unset,
        /// a sentence and a link to the file.
        #[props(default)]
        children: Element,
    }
}

/// A video player with its own controls: play, seek, time, mute, volume,
/// captions and fullscreen.
///
/// Keys while focus is inside: K, or Space on a slider, play and pause; J and L
/// jump 10 seconds, M mutes, C toggles captions, F fullscreen, Ctrl+ArrowRight
/// and Ctrl+ArrowLeft go to the next and previous chapter; Shift+? lists them.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Video;
/// # fn app() -> Element {
/// rsx! { Video { src: "/launch.webm", label: "Launch day", poster: "/launch.jpg" } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/video>
#[component]
pub fn Video(props: VideoProps) -> Element {
    let own = use_media();
    let media = props.media.unwrap_or(own);
    let player = use_element();
    let fullscreen = use_fullscreen(player);
    // The controls' menus and tooltips portal here: the page's outlet, out of
    // fullscreen; one inside the player in it, as nothing outside it shows.
    let page_portal = use_portal_slot();
    // Owned by the root: the page's outlet outlives this scope.
    let portals =
        use_context_provider(|| PortalHost::new(Signal::new_in_scope(Vec::new(), ScopeId::ROOT)));
    use_drop(move || portals.entries.manually_drop());
    let mut portaled_out = use_hook(|| CopyValue::new(false));
    let inside = fullscreen.is_fullscreen();
    // The drawn box covers the page, which a wheel would scroll behind it (todo 1285).
    let scroll_lock = use_scroll_lock(player, fullscreen.is_drawn());
    if *portaled_out.peek() == inside {
        portaled_out.set(!inside);
        page_portal.show((!inside).then(|| rsx! { HostOutlet { host: portals } }));
    }

    use_name_warning(
        !props.label.trim().is_empty(),
        "Video: an empty `label`, so the player is announced as just \"group\".",
    );
    let autoplay_warned = props.autoplay && !props.muted;
    use_hook(move || {
        if autoplay_warned {
            warn(
                "Video: `autoplay` without `muted`: browsers refuse it, and sound out of nowhere fails WCAG 1.4.2.",
            );
        }
    });

    let tracks: Vec<&MediaTrack> = props
        .tracks
        .iter()
        .filter(|track| media_url(&track.src, "Video `tracks`").is_some())
        .collect();
    let text_tracks: Vec<(usize, &MediaTrack)> = tracks
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, t)| t.kind.shows_text())
        .collect();
    let picked = use_signal(|| None::<usize>);
    let caption_track = caption_track(picked(), &text_tracks);
    let shown_at_start = text_tracks.iter().any(|(_, t)| t.default);
    let shown = use_signal(|| shown_at_start);
    let captions = Captions {
        shown,
        track: caption_track,
        picked,
    };
    // After the `PortalHost`: in fullscreen the help shows inside the player.
    let labels = use_localization().media;
    let caption_tracks = caption_menu(&text_tracks, labels.captions);

    let chapters_src = chapters_src(&tracks, !props.chapters.is_empty());
    let mut read = use_signal(|| None::<(String, Vec<Chapter>)>);
    use_cache(chapters_src.clone(), |src| {
        if let Some(src) = src.clone() {
            let fetched = platform::fetch_text(&src);
            spawn(async move {
                let chapters = fetched
                    .await
                    .map_or_else(Vec::new, |body| Chapter::parse_vtt(&body));
                read.set(Some((src, chapters)));
            });
        }
    });
    let chapters = sorted_chapters(match &chapters_src {
        Some(src) => read_chapters(read(), src),
        None => props.chapters.clone(),
    });
    let starts: Vec<f64> = chapters.iter().map(|chapter| chapter.start).collect();
    let has_chapters = !starts.is_empty();
    // Next is the way the track runs: ArrowLeft under RTL.
    let chapter_key = move |chord: &str, ahead_in_ltr: bool| {
        let starts = starts.clone();
        Hotkey::new(chord, move || {
            let forward = ahead_in_ltr != player.is_rtl();
            if let Some(start) = chapter_jump(&starts, media.current_time(), forward) {
                media.seek(start);
            }
        })
        .when(move || has_chapters)
    };

    let sound = use_sound(media);
    use_media_keys(
        media,
        sound,
        player,
        [
            (
                Hotkey::new("c", move || captions.toggle(media))
                    .when(move || captions.track.is_some()),
                caption_track.map(|_| Shortcut::new("c", labels.shortcut_captions)),
            ),
            (
                Hotkey::new("f", move || fullscreen.toggle()),
                Some(Shortcut::new("f", labels.shortcut_fullscreen)),
            ),
            (
                chapter_key(NEXT_CHAPTER, true),
                has_chapters.then(|| Shortcut::new(NEXT_CHAPTER, labels.shortcut_next_chapter)),
            ),
            (
                chapter_key(PREVIOUS_CHAPTER, false),
                has_chapters
                    .then(|| Shortcut::new(PREVIOUS_CHAPTER, labels.shortcut_previous_chapter)),
            ),
        ],
    );
    use_media_start(media, props.muted, props.onerror);

    let unsupported = media.supported() == Some(false);
    let mut bar = use_signal(|| None::<f64>);
    let style = bar_style(
        bar(),
        props.size.as_ref().copied(),
        props.aspect_ratio.as_deref(),
    );

    let playing = !media.paused();
    let controls = use_controls_idle(playing);

    let mut pressing = use_hook(|| CopyValue::new(false));
    // A click on the picture plays or pauses, as YouTube's; a tap on hidden controls only shows
    // them, once its click is through: the bar takes hits again as soon as it shows (todo 3660).
    let mut tap_shows = use_hook(|| CopyValue::new(false));
    // Tagged, so a WebView finds the player for fullscreen and the hotkeys.
    let mut attributes = props.attributes;
    // Focus moved in without a press, by a key, code or AT, counts as a key, also while paused
    // (todos 1249, 3656): the bar then stays up once a Space on a focused button starts playing.
    attributes.extend(fullscreen.attributes_with_focusin(move |_| {
        if !*pressing.peek() {
            controls.by_key();
        }
    }));
    // Every press ends here, also one released outside or paused on the way (todo 1418);
    // a leave without capture comes after the press's own focusin.
    let released = move |_: Event<PointerData>| {
        let mut pressing = pressing;
        if *pressing.peek() {
            pressing.set(false);
        }
    };
    attributes.extend([
        listener("onpointerdown", move |_: Event<PointerData>| {
            pressing.set(true);
            controls.by_pointer();
            if playing && !*tap_shows.peek() {
                controls.wake();
            }
        }),
        listener("onpointerup", released),
        listener("onpointercancel", released),
        listener("onpointerleave", move |event: Event<PointerData>| {
            released(event.clone());
            // A mouse only: a touch leaves after every tap.
            if playing && event.pointer_type() == "mouse" {
                controls.mouse_left();
            }
        }),
    ]);
    // Only while playing, so a WebView sends no pointer moves across the IPC otherwise.
    if playing {
        attributes.extend([
            listener("onpointermove", move |_: Event<PointerData>| {
                controls.wake()
            }),
            listener("onkeyup", move |_: Event<KeyboardData>| controls.by_key()),
        ]);
    }
    // An error leaves the element unpaused: its alert keeps the controls up (todo 2681).
    let hidden = controls.hidden(playing, media.error().is_some());
    if hidden {
        attributes.push(Attribute::new(
            CONTROLS_ATTR,
            AttributeValue::Text("hidden".into()),
            None,
            false,
        ));
    }

    use_source_reload(media, &props.src, &props.sources);
    let (src, sources) = media_sources(&props.src, &props.sources);
    let body = rsx! {
        video {
            "data-slot": VideoPart::Media.slot(),
            src,
            poster: props.poster.as_deref().and_then(|poster| media_url(poster, "Video `poster`")),
            style,
            autoplay: props.autoplay,
            muted: props.muted,
            "loop": props.looping,
            preload: props.preload.attribute(),
            playsinline: true,
            onmounted: media.mount(),
            onpointerdown: move |event: PointerEvent| tap_shows.set(event.pointer_type() == "touch" && hidden),
            onclick: move |_| {
                if *tap_shows.peek() {
                    controls.wake();
                } else if media.error().is_none() {
                    media.toggle();
                }
                tap_shows.set(false);
            },
            onplay: notify(props.onplay),
            onpause: notify(props.onpause),
            onended: notify(props.onended),
            ..media.attributes(),
            {sources}
            for track in tracks.iter() {
                track {
                    src: track.src.clone(),
                    kind: track.kind.attribute(),
                    srclang: track.srclang.clone(),
                    label: track.label.clone(),
                    default: track.default,
                }
            }
        }
        if unsupported {
            MediaFallback { src: props.src.clone(), children: props.children }
        } else {
            MediaControls {
                media,
                sound,
                size: props.size.clone(),
                captions: Some(captions),
                caption_tracks,
                chapters,
                fullscreen: Some(fullscreen),
                overlay: true,
                volume_parts: props.volume_parts,
                onheight: move |height: f64| {
                    if *bar.peek() != Some(height) {
                        bar.set(Some(height));
                    }
                },
            }
        }
        if inside {
            HostOutlet { host: portals }
        }
        {scroll_lock}
    };

    use_box()
        .framework_sx(&VIDEO_SX)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .prepare()
        .element(&player)
        .attr("role", "group")
        .attr("aria-label", props.label.clone())
        .render(HtmlTag::Div, attributes, body)
}

#[cfg(test)]
mod tests {
    use super::super::media_controls::{CHAPTER, CONTROLS, MESSAGE, SEEK, TIME, VOLUME};
    use super::*;
    use crate::LiberoProvider;
    use crate::components::common::part_table;

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<VideoPart>(),
            [
                ("media", "& > [data-slot='media']"),
                ("controls", "& > [data-slot='controls']"),
                ("time", "& [data-slot='time']"),
                ("chapter", "& [data-slot='chapter']"),
                ("seek", "& [data-slot='seek']"),
                ("volume", "& [data-slot='volume']"),
                ("message", "& > [data-slot='message']"),
            ]
        );
    }

    #[test]
    fn the_shared_controls_use_the_part_names() {
        assert_eq!(
            [CONTROLS, TIME, CHAPTER, SEEK, VOLUME, MESSAGE],
            [
                VideoPart::Controls.slot(),
                VideoPart::Time.slot(),
                VideoPart::Chapter.slot(),
                VideoPart::Seek.slot(),
                VideoPart::Volume.slot(),
                VideoPart::Message.slot(),
            ]
        );
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_script_url_is_not_rendered_as_media() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    Video {
                        src: "javascript:alert(1)",
                        sources: vec![
                            MediaSource::new(" VBScript:x", "video/mp4"),
                            MediaSource::new("/ok.webm", "video/webm"),
                        ],
                        label: "Clip",
                        poster: "java\tscript:alert(2)",
                        tracks: vec![
                            MediaTrack { src: "javascript:alert(3)".into(), ..Default::default() },
                            MediaTrack { src: "/ok.vtt".into(), ..Default::default() },
                        ],
                    }
                }
            }
        });
        assert!(!html.contains("script:"), "{html}");
        assert!(!html.contains("poster"), "{html}");
        assert!(
            html.contains("/ok.webm") && html.contains("/ok.vtt"),
            "{html}"
        );
    }

    #[test]
    fn a_data_url_and_a_file_render_as_media() {
        let html = render(|| {
            rsx! {
                LiberoProvider {
                    Video {
                        src: "data:video/mp4;base64,AAAA",
                        label: "Clip",
                        poster: "data:image/gif;base64,R0lGOD",
                    }
                }
            }
        });
        assert!(
            html.contains(r#"src="data:video/mp4;base64,AAAA""#),
            "{html}"
        );
        assert!(
            html.contains(r#"poster="data:image/gif;base64,R0lGOD""#),
            "{html}"
        );
    }

    #[test]
    fn a_vtt_file_reads_as_chapters() {
        let vtt = "WEBVTT\r\n\r\nNOTE made by hand\r\n\r\nintro\r\n00:00.000 --> 00:01.500\r\nIntro\r\n\r\n\
                   00:00:01.500 --> 00:00:03.000 align:start\r\n<b>Tom &amp; Jerry</b>\r\nPart two\r\n\r\n\
                   01:02:03.250 --> 01:02:04.000\r\n\r\nbroken --> 00:05.000\r\nNo start\r\n";
        assert_eq!(
            Chapter::parse_vtt(vtt),
            [
                Chapter::new(0.0, "Intro"),
                Chapter::new(1.5, "Tom & Jerry Part two"),
            ]
        );
        assert_eq!(vtt_time("01:02:03.250"), Some(3723.25));
        assert_eq!(vtt_time("2:03"), Some(123.0));
        assert_eq!(vtt_time("3"), None);
    }

    #[test]
    fn the_faded_selectors_target_the_direct_children() {
        assert_eq!(
            when_faded(VideoPart::Controls),
            "&[data-controls='hidden'] > [data-slot='controls']"
        );
        assert_eq!(
            when_faded(VideoPart::Media),
            "&[data-controls='hidden'] > [data-slot='media']"
        );
    }

    /// `&amp;` last, so an escaped entity stays text: `&amp;lt;` is `&lt;`, not `<`.
    #[test]
    fn a_cue_decodes_its_entities_with_the_ampersand_last() {
        assert_eq!(
            cue_text("a &lt;b&gt;&nbsp;&lrm;&rlm; &amp;lt; <i>c</i>"),
            "a <b>\u{a0}\u{200e}\u{200f} &lt; c"
        );
    }

    #[test]
    fn a_track_menu_item_falls_back_to_the_language_then_a_number() {
        let track = |label: &str, srclang: &str| MediaTrack {
            label: label.into(),
            srclang: srclang.into(),
            ..Default::default()
        };
        assert_eq!(track("English", "en").menu_name("Captions", 2), "English");
        assert_eq!(track("", "en").menu_name("Captions", 2), "en");
        assert_eq!(track(" ", "").menu_name("Captions", 2), "Captions 2");
    }

    fn track(kind: TrackKind, src: &str, default: bool) -> MediaTrack {
        MediaTrack {
            src: src.into(),
            kind,
            default,
            ..Default::default()
        }
    }

    #[test]
    fn the_caption_track_is_the_picked_then_the_default_then_the_first() {
        let (a, b, c) = (
            track(TrackKind::Captions, "a", false),
            track(TrackKind::Captions, "b", true),
            track(TrackKind::Captions, "c", false),
        );
        let tracks = [(0, &a), (2, &b), (5, &c)];
        assert_eq!(caption_track(Some(5), &tracks), Some(5));
        assert_eq!(caption_track(None, &tracks), Some(2));
        assert_eq!(caption_track(Some(1), &tracks), Some(2), "a stale pick");
        assert_eq!(caption_track(None, &[(3, &a), (4, &c)]), Some(3));
        assert_eq!(caption_track(Some(0), &[]), None);
    }

    #[test]
    fn the_caption_menu_lists_two_or_more_tracks_numbered_among_the_text_ones() {
        let (a, b) = (
            track(TrackKind::Captions, "a", false),
            MediaTrack {
                label: "Deutsch".into(),
                ..track(TrackKind::Subtitles, "b", false)
            },
        );
        assert!(caption_menu(&[(0, &a)], "Captions").is_empty());
        assert_eq!(
            caption_menu(&[(1, &a), (4, &b)], "Captions"),
            [(1, "Captions 1".to_string()), (4, "Deutsch".to_string())]
        );
    }

    #[test]
    fn the_chapters_track_is_read_only_when_the_prop_lists_none() {
        let (late, first, other) = (
            track(TrackKind::Chapters, "late.vtt", false),
            track(TrackKind::Chapters, "first.vtt", true),
            track(TrackKind::Captions, "c.vtt", true),
        );
        let tracks = [&other, &late, &first];
        assert_eq!(chapters_src(&tracks, false).as_deref(), Some("first.vtt"));
        assert_eq!(chapters_src(&tracks, true), None);
        assert_eq!(chapters_src(&[&other], false), None);
    }

    #[test]
    fn a_late_chapters_answer_for_an_old_src_is_dropped() {
        let read = Some(("old.vtt".to_string(), vec![Chapter::new(0.0, "A")]));
        assert_eq!(
            read_chapters(read.clone(), "old.vtt"),
            [Chapter::new(0.0, "A")]
        );
        assert!(read_chapters(read, "new.vtt").is_empty());
        assert!(read_chapters(None, "new.vtt").is_empty());
    }

    #[test]
    fn the_bar_style_is_measured_then_estimated_and_carries_the_ratio() {
        assert_eq!(
            bar_style(Some(48.0), None, None),
            format!("{BAR_VAR}: calc(48px - {SCRIM_FADE});")
        );
        let estimate = bar_style(None, Some(Size::Md), Some("4 / 3"));
        assert!(
            estimate.starts_with(&format!("{BAR_VAR}: calc("))
                && estimate.ends_with("aspect-ratio: 4 / 3;"),
            "{estimate}"
        );
        assert!(!bar_style(None, None, None).contains("aspect-ratio"));
        let escaping = bar_style(None, None, Some("1/1;position:fixed"));
        assert!(!escaping.contains("aspect-ratio") && !escaping.contains("fixed"));
    }

    #[test]
    fn the_chapters_are_sorted_and_cleaned() {
        let chapters = sorted_chapters(vec![
            Chapter::new(5.0, "B"),
            Chapter::new(f64::NAN, "Nan"),
            Chapter::new(0.0, "A"),
            Chapter::new(5.0, "B again"),
            Chapter::new(-1.0, "Before"),
        ]);
        assert_eq!(chapters, [Chapter::new(0.0, "A"), Chapter::new(5.0, "B")]);
    }

    fn alpha(rgba: &str) -> f32 {
        rgba.trim_end_matches(')')
            .rsplit(", ")
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }

    /// Over a white frame the scrim keeps the white text at 4.5:1 and the light seek track at 3:1.
    #[test]
    fn the_scrim_holds_its_contrast_over_a_white_frame() {
        use crate::tokens::HexColor;
        let grey = |level: f32| HexColor::new(level.round() as u32 * 0x01_01_01);
        let under = 255.0 * (1.0 - alpha(SCRIM));
        let track = 255.0 * alpha(LIGHT_TRACK) + under * (1.0 - alpha(LIGHT_TRACK));
        let text = HexColor::new(0xFF_FF_FF).contrast_ratio(grey(under));
        let track = grey(track).contrast_ratio(grey(under));
        assert!(text >= 4.5, "the text is {text:.2}:1");
        assert!(track >= 3.0, "the track is {track:.2}:1");
    }
}
