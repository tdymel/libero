use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use super::media_controls::{
    CONTROLS, MediaFallback, MediaSeek, MediaStatus, MediaVolume, SEEK, Sound, TIME, clock,
    controls_sx, icon_size, seek_sx, space_toggles, times, use_late_tooltips, use_media_keys,
    use_sound,
};
use crate::{
    components::{
        buttons::{ActionIcon, Button},
        common::{Glyph, HtmlTag, Input, Part, base_props, parts_enum, use_name_warning},
        layout::use_box,
    },
    context::IconSlot,
    hooks::{MediaError, MediaHandle, use_formats, use_localization, use_media},
    localization::fill,
    platform::waveform,
    sx::{FORCED_COLORS, StaticSx, Sx, sx},
    theme::{BUTTON_FONT_SIZE, BUTTON_HEIGHT, ColorCss, ColorShade, ICON_SIZE, Size, SizeCss},
    utils::warn,
};
use pictogram_icons_lucide as lucide;

parts_enum! {
    /// [`Audio`]'s inner parts, for its `parts` prop.
    pub enum AudioPart {
        /// The row of controls.
        Controls = "controls" => "& > [data-slot='controls']",
        /// The time: the total until playing starts, then the elapsed.
        Time = "time" => "& [data-slot='time']",
        /// The seek track's wrapper: the bars and the slider over them.
        Seek = "seek" => "& [data-slot='seek']",
        /// The mute button and the volume menu's trigger.
        Volume = "volume" => "& [data-slot='volume']",
        /// The text shown when the source fails, or nothing plays media.
        Message = "message" => "& > [data-slot='message']",
    }
}

/// The speeds the speed button steps through, as chat apps offer.
const SPEEDS: [f64; 3] = [1.0, 1.5, 2.0];

/// The seek track's bars: few enough to stay apart at the track's `2rem` minimum.
const BARS: usize = 28;
const BARS_SLOT: &str = "bars";
/// Longer files keep the drawn bars.
const DECODE_MAX_SECONDS: f64 = 10.0 * 60.0;
const AUDIO_CONTAINER: &str = "libero-audio";
const NARROW: &str = "(max-width: 15rem)";
const NARROWEST: &str = "(max-width: 13rem)";
const TINY: &str = "(max-width: 11rem)";

static AUDIO_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .min_width("0")
        // A voice message's bubble: a page-wide container does not stretch it. A
        // fixed width, not `100%`, so a shrink-wrapping parent cannot zero the container.
        .width("22rem")
        .max_width("100%")
        .container(AUDIO_CONTAINER)
        // One row at every width: the seek track gives way, then the time, then the buttons shrink.
        .selector(
            AudioPart::Controls.selector(),
            controls_sx()
                .flex_wrap("nowrap")
                .border_radius(SizeCss::RADIUS.value(Size::Xl)),
        )
        .selector(
            AudioPart::Time.selector(),
            sx().font_variant_numeric("tabular-nums")
                .white_space("nowrap")
                .color(ColorCss::MUTED.value(ColorShade::S7))
                .container_query(AUDIO_CONTAINER, NARROW, sx().display("none")),
        )
        .selector(
            AudioPart::Seek.selector(),
            seek_sx()
                .flex_shrink("1")
                .position("relative")
                // The bars draw the track; the slider keeps the thumb, the keys and the drag.
                .selector(
                    "& [data-slot='track'], & [data-slot='bar']",
                    sx().background("transparent"),
                )
                // As tall as the bars, so a press anywhere on them seeks.
                .selector("& [data-slot='track']", sx().height("1.5rem")),
        )
        .selector(
            format!("& [data-slot='{BARS_SLOT}']"),
            sx().position("absolute")
                .top("0")
                .bottom("0")
                .left("0.5rem")
                .right("0.5rem")
                .display("flex")
                .align_items("center")
                .justify_content("space-between")
                .pointer_events("none")
                .selector(
                    "& > span",
                    sx().flex("0 1 2px")
                        .min_width("1px")
                        .border_radius("999px")
                        // 3:1 against the surface, as the slider's track (WCAG 1.4.11).
                        .background("muted.6")
                        .media(FORCED_COLORS, sx().background("CanvasText")),
                )
                .selector(
                    "& > span[data-played]",
                    sx().background("primary.6")
                        .media(FORCED_COLORS, sx().background("Highlight")),
                )
                // Half the bars where the track is short, or they run together.
                .selector(
                    "& > span:nth-child(even)",
                    sx().container_query(AUDIO_CONTAINER, NARROW, sx().display("none")),
                ),
        )
        .selector(
            AudioPart::Volume.selector(),
            sx().display("flex").align_items("center"),
        )
        // Smaller buttons, not hidden ones, so mute and volume stay (WCAG 1.4.10, todo 1393).
        .container_query(AUDIO_CONTAINER, NARROWEST, compact_buttons(Size::Sm))
        .container_query(AUDIO_CONTAINER, TINY, compact_buttons(Size::Xs))
        .selector(
            AudioPart::Message.selector(),
            sx().color(ColorCss::MUTED.value(ColorShade::S7)),
        )
});

/// The row's buttons at `size` whatever the player's; `Xs` keeps the 24px target (WCAG 2.5.8).
fn compact_buttons(size: Size) -> Sx {
    let controls = AudioPart::Controls.selector();
    sx().selector(
        format!("{controls} button"),
        sx().height(BUTTON_HEIGHT.value(size))
            .font_size(BUTTON_FONT_SIZE.value(size))
            .selector(
                "& svg",
                sx().width(ICON_SIZE.value(size))
                    .height(ICON_SIZE.value(size)),
            ),
    )
    // The icon buttons sit in their tooltips' wrappers; the chevron keeps its half width.
    .selector(
        format!("{controls} span > button"),
        sx().width(BUTTON_HEIGHT.value(size)),
    )
}

/// How much an `Audio` fetches before a press.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MediaPreload {
    /// Nothing until playing starts.
    None,
    /// The duration and first frame only.
    #[default]
    Metadata,
    /// As much as the browser likes.
    Auto,
}

impl MediaPreload {
    pub(super) fn attribute(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Metadata => "metadata",
            Self::Auto => "auto",
        }
    }
}

/// The file in another format, for the `sources` of [`Audio`] and
/// [`Video`](super::Video): a browser plays the first whose `mime` it supports.
#[derive(Clone, Debug, PartialEq)]
pub struct MediaSource {
    /// The file's URL.
    pub src: String,
    /// Its MIME type, such as `"video/mp4"` or `"audio/ogg; codecs=opus"`.
    pub mime: String,
}

impl MediaSource {
    pub fn new(src: impl Into<String>, mime: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            mime: mime.into(),
        }
    }
}

/// `src` as the element's attribute, or with `sources` the last `<source>` after them.
pub(super) fn media_sources(src: &str, sources: &[MediaSource]) -> (Option<String>, Element) {
    if sources.is_empty() {
        return (Some(src.to_string()), VNode::empty());
    }
    let list = rsx! {
        for other in sources {
            source { src: other.src.clone(), r#type: other.mime.clone() }
        }
        source { src: src.to_string() }
    };
    (None, list)
}

/// Reloads `media` when the `<source>` list changes after mount; a changed `src`
/// attribute alone the browser loads by itself.
pub(super) fn use_source_reload(media: MediaHandle, src: &str, sources: &[MediaSource]) {
    let key = (!sources.is_empty()).then(|| (src.to_string(), sources.to_vec()));
    let last = use_hook(|| Rc::new(RefCell::new(key.clone())));
    use_effect(use_reactive!(|key| {
        if *last.borrow() != key {
            *last.borrow_mut() = key;
            media.reload();
        }
    }));
}

base_props! {
    parts(AudioPart);
    pub struct AudioProps {
        /// The file's URL.
        #[props(into)]
        src: String,
        /// The file in other formats, tried in order before `src`.
        #[props(default)]
        sources: Vec<MediaSource>,
        /// Names the player, e.g. the track's title.
        #[props(into)]
        label: String,
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
        #[props(default)]
        preload: MediaPreload,
        /// Of the buttons and the seek slider.
        #[props(default, into)]
        size: Input<Size>,
        #[props(default)]
        onplay: Option<EventHandler<()>>,
        #[props(default)]
        onpause: Option<EventHandler<()>>,
        #[props(default)]
        onended: Option<EventHandler<()>>,
        #[props(default)]
        onerror: Option<EventHandler<MediaError>>,
        /// Shown instead of the controls where nothing plays media (Blitz). Unset,
        /// a sentence and a link to the file.
        #[props(default)]
        children: Element,
    }
}

/// An audio player in one compact row, as a chat app's voice message: play, a
/// track of bars to seek, the time, mute with a volume menu, and a speed button
/// stepping 1×, 1.5× and 2×.
///
/// Keys while focus is inside: K, or Space on a slider, play and pause; J and L
/// jump 10 seconds; M mutes; Shift+? lists them.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Audio;
/// # fn app() -> Element {
/// rsx! { Audio { src: "/episode-12.ogg", label: "Episode 12: Borrowing" } }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/audio>
#[component]
pub fn Audio(props: AudioProps) -> Element {
    let own = use_media();
    let media = props.media.unwrap_or(own);
    let player = crate::hooks::use_element();

    use_name_warning(
        !props.label.trim().is_empty(),
        "Audio: an empty `label`, so the player is announced as just \"group\".",
    );
    let autoplay_warned = props.autoplay && !props.muted;
    use_hook(move || {
        if autoplay_warned {
            warn(
                "Audio: `autoplay` without `muted`: browsers refuse it, and sound out of nowhere fails WCAG 1.4.2.",
            );
        }
    });

    let sound = use_sound(media);
    use_media_keys(media, sound, player, []);
    // A `muted` attribute set after parsing mutes nothing, so the property is set once mounted.
    let start_muted = props.muted;
    use_effect(move || {
        if start_muted && media.supported() == Some(true) {
            media.set_muted(true);
        }
    });
    let error = media.error();
    let onerror = props.onerror;
    use_effect(use_reactive!(|error| {
        if let (Some(error), Some(onerror)) = (error, onerror) {
            onerror.call(error);
        }
    }));

    let (onplay, onpause, onended) = (props.onplay, props.onpause, props.onended);
    let unsupported = media.supported() == Some(false);

    // Tagged, so a WebView finds the player for the hotkeys.
    let mut attributes = props.attributes;
    attributes.extend(player.attributes());

    use_source_reload(media, &props.src, &props.sources);
    // `preload: None` promises no fetch before a press, so it keeps the drawn bars.
    let decoded = use_waveform(
        &props.src,
        props.preload != MediaPreload::None && !unsupported && decodable(media.duration()),
    );
    let heights = decoded.unwrap_or_else(|| bar_heights(&props.src));
    let (src, sources) = media_sources(&props.src, &props.sources);
    let body = rsx! {
        audio {
            src,
            autoplay: props.autoplay,
            muted: props.muted,
            "loop": props.looping,
            preload: props.preload.attribute(),
            onmounted: media.mount(),
            onplay: move |_| if let Some(handler) = onplay { handler.call(()) },
            onpause: move |_| if let Some(handler) = onpause { handler.call(()) },
            onended: move |_| if let Some(handler) = onended { handler.call(()) },
            ..media.attributes(),
            {sources}
        }
        if unsupported {
            MediaFallback { src: props.src.clone(), children: props.children }
        } else {
            AudioControls { media, sound, size: props.size.clone(), heights }
        }
    };

    use_box()
        .framework_sx(&AUDIO_SX)
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

/// One row: play, seek, time, mute and volume, speed; then the error and buffering messages.
#[component]
fn AudioControls(
    media: MediaHandle,
    sound: Sound,
    size: Input<Size>,
    heights: [f64; BARS],
) -> Element {
    use_late_tooltips();
    let labels = use_localization().media;
    rsx! {
        div { role: "group", "aria-label": labels.controls, "data-slot": CONTROLS,
            ActionIcon {
                aria_label: if media.paused() { labels.play } else { labels.pause },
                variant: "filled",
                radius: "50%",
                tooltip: true,
                shortcut: "k",
                size: icon_size(&size),
                onclick: move |_| media.toggle(),
                if media.paused() {
                    Glyph { slot: IconSlot::Play, icon: lucide::play::outlined }
                } else {
                    Glyph { slot: IconSlot::Pause, icon: lucide::pause::outlined }
                }
            }
            div { "data-slot": SEEK, onkeydown: space_toggles(media),
                AudioBars { media, heights }
                MediaSeek { media, size: size.clone() }
            }
            AudioTime { media }
            MediaVolume { media, sound, size: size.clone() }
            AudioSpeed { media, size }
        }
        MediaStatus { media }
    }
}

/// The bars behind the seek slider, those played filled; its own component, so
/// a time tick re-renders only this.
#[component]
fn AudioBars(media: MediaHandle, heights: [f64; BARS]) -> Element {
    let played = played_bars(media.current_time(), media.duration());
    rsx! {
        div { "data-slot": BARS_SLOT, "aria-hidden": "true",
            for (index, height) in heights.into_iter().enumerate() {
                span {
                    key: "{index}",
                    "data-played": (index < played).then_some(""),
                    style: "height: {height * 100.0:.0}%",
                }
            }
        }
    }
}

/// The bars' heights decoded from `src` once it answers; `None` until then, or
/// where it fails (a cross-origin file without CORS, no Web Audio, Blitz).
fn use_waveform(src: &str, decode: bool) -> Option<[f64; BARS]> {
    let mut decoded = use_signal(|| None::<(String, [f64; BARS])>);
    let src = src.to_string();
    use_effect(use_reactive!(|src, decode| {
        let Some(api) = waveform().filter(|_| decode) else {
            return;
        };
        // Started here, not in the task: a WebView's eval needs the calling scope.
        let peaks = api.peaks(&src, BARS);
        spawn(async move {
            if let Some(heights) = peaks.await.as_deref().and_then(scaled_heights) {
                decoded.set(Some((src, heights)));
            }
        });
    }));
    let decoded = decoded.read();
    decoded
        .as_ref()
        .filter(|(of, _)| *of == src)
        .map(|(_, heights)| *heights)
}

/// Decodes once the metadata gives a duration of at most [`DECODE_MAX_SECONDS`]:
/// a long podcast would be fetched and decoded whole for 28 bars.
fn decodable(duration: Option<f64>) -> bool {
    duration.is_some_and(|total| total.is_finite() && total <= DECODE_MAX_SECONDS)
}

/// Loudness per bar as heights from 0.2 to 1, the loudest full; silence stays at 0.2.
fn scaled_heights(peaks: &[f64]) -> Option<[f64; BARS]> {
    if peaks.len() != BARS || peaks.iter().any(|peak| !peak.is_finite()) {
        return None;
    }
    let loudest = peaks.iter().copied().fold(0.0, f64::max);
    Some(std::array::from_fn(|index| {
        if loudest > 0.0 {
            0.2 + 0.8 * peaks[index] / loudest
        } else {
            0.2
        }
    }))
}

/// Heights from 0.2 to 1, drawn from `src` until the decode answers: a file
/// looks the same each time.
fn bar_heights(src: &str) -> [f64; BARS] {
    // FNV-1a seeds an xorshift; neighbours blend in lightly so it reads as a waveform.
    let mut state = src.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    }) | 1;
    let raw: [f64; BARS] = std::array::from_fn(|_| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    });
    std::array::from_fn(|index| {
        let before = raw[index.saturating_sub(1)];
        let after = raw[(index + 1).min(BARS - 1)];
        0.2 + 0.8 * (before + 4.0 * raw[index] + after) / 6.0
    })
}

/// How many of the [`BARS`] are played at `elapsed`.
fn played_bars(elapsed: f64, duration: Option<f64>) -> usize {
    match duration {
        Some(total) if total > 0.0 => {
            ((elapsed / total).clamp(0.0, 1.0) * BARS as f64).round() as usize
        }
        _ => 0,
    }
}

/// Its own component, so a time tick re-renders only this.
#[component]
fn AudioTime(media: MediaHandle) -> Element {
    let time = shown_time(media.paused(), media.current_time(), media.duration());
    rsx! {
        // The seek slider's value text says it to a screen reader.
        span { "data-slot": TIME, "aria-hidden": "true", "{time}" }
    }
}

/// The total until playing starts, then the elapsed time.
fn shown_time(paused: bool, elapsed: f64, duration: Option<f64>) -> String {
    match (paused && elapsed == 0.0, duration) {
        (false, _) => clock(elapsed),
        (true, Some(total)) => clock(total),
        (true, None) => "--:--".to_string(),
    }
}

/// "1×"; a press steps to the next of [`SPEEDS`].
#[component]
fn AudioSpeed(media: MediaHandle, size: Input<Size>) -> Element {
    let labels = use_localization().media;
    let shown = times(media.rate(), use_formats().decimal_separator);
    rsx! {
        Button {
            aria_label: fill(labels.speed, &[("rate", &shown)]),
            // "1×" stays "1×" in a right-to-left page, not "×1".
            dir: "ltr",
            variant: "tonal",
            radius: "xl",
            size,
            // Narrow padding and a fixed width: "1.5×" must not shift the track.
            sx: sx().padding_inline(SizeCss::SPACING.value(Size::Xs)).min_width("3.25em"),
            onclick: move |_| media.set_rate(next_speed(media.rate())),
            "{shown}"
        }
    }
}

/// The first of [`SPEEDS`] above `rate`, else the first: a rate set from outside
/// steps on from where it is.
fn next_speed(rate: f64) -> f64 {
    SPEEDS
        .into_iter()
        .find(|&speed| speed > rate)
        .unwrap_or(SPEEDS[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    #[test]
    fn the_speed_button_cycles_and_steps_on_from_an_outside_rate() {
        assert_eq!(next_speed(1.0), 1.5);
        assert_eq!(next_speed(1.5), 2.0);
        assert_eq!(next_speed(2.0), 1.0);
        assert_eq!(next_speed(0.75), 1.0);
        assert_eq!(next_speed(1.25), 1.5);
        assert_eq!(next_speed(3.0), 1.0);
    }

    #[test]
    fn the_time_is_the_total_until_playing_starts() {
        assert_eq!(shown_time(true, 0.0, None), "--:--");
        assert_eq!(shown_time(true, 0.0, Some(65.0)), "1:05");
        assert_eq!(shown_time(false, 0.0, Some(65.0)), "0:00");
        assert_eq!(shown_time(true, 3.0, Some(65.0)), "0:03");
    }

    #[test]
    fn the_bars_are_stable_per_file_and_in_range() {
        let heights = bar_heights("/voice.ogg");
        assert_eq!(heights, bar_heights("/voice.ogg"));
        assert_ne!(heights, bar_heights("/other.ogg"));
        assert!(heights.iter().all(|height| (0.2..=1.0).contains(height)));
    }

    #[test]
    fn decoded_peaks_scale_to_the_loudest() {
        let mut peaks = [0.25; BARS];
        peaks[3] = 0.5;
        let heights = scaled_heights(&peaks).unwrap();
        assert_eq!(heights[3], 1.0);
        assert!((heights[0] - 0.6).abs() < 1e-9);
        assert_eq!(scaled_heights(&[0.0; BARS]), Some([0.2; BARS]));
        assert_eq!(scaled_heights(&[0.5; 3]), None);
        assert_eq!(scaled_heights(&[f64::NAN; BARS]), None);
    }

    #[test]
    fn only_a_short_file_with_a_known_duration_decodes() {
        assert!(!decodable(None));
        assert!(decodable(Some(42.0)));
        assert!(decodable(Some(DECODE_MAX_SECONDS)));
        assert!(!decodable(Some(DECODE_MAX_SECONDS + 1.0)));
        assert!(!decodable(Some(f64::INFINITY)));
        assert!(!decodable(Some(f64::NAN)));
    }

    #[test]
    fn the_played_bars_follow_the_time() {
        assert_eq!(played_bars(5.0, None), 0);
        assert_eq!(played_bars(0.0, Some(60.0)), 0);
        assert_eq!(played_bars(30.0, Some(60.0)), BARS / 2);
        assert_eq!(played_bars(90.0, Some(60.0)), BARS);
    }

    /// The slot names are public: a rename here is a breaking change.
    #[test]
    fn the_part_table_is_stable() {
        assert_eq!(
            part_table::<AudioPart>(),
            [
                ("controls", "& > [data-slot='controls']"),
                ("time", "& [data-slot='time']"),
                ("seek", "& [data-slot='seek']"),
                ("volume", "& [data-slot='volume']"),
                ("message", "& > [data-slot='message']"),
            ]
        );
    }
}
