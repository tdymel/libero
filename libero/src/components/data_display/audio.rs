use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;

use super::media_controls::{
    CONTROLS, MediaFallback, MediaSeek, MediaStatus, SEEK, Sound, TIME, VOLUME, clock, controls_sx,
    icon_size, seek_sx, space_toggles, times, use_sound,
};
use crate::{
    components::{
        buttons::{ActionIcon, Button},
        common::{Glyph, HtmlTag, Input, Part, base_props, parts_enum, use_name_warning},
        form::{Slider, SliderChangeEvent},
        layout::use_box,
    },
    context::IconSlot,
    hooks::{
        Hotkey, MediaError, MediaHandle, use_formats, use_hotkeys, use_localization, use_media,
    },
    localization::fill,
    sx::{StaticSx, sx},
    theme::{ColorCss, ColorShade, Size, SizeCss},
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
        /// The seek slider's wrapper.
        Seek = "seek" => "& [data-slot='seek']",
        /// The volume slider's wrapper.
        Volume = "volume" => "& [data-slot='volume']",
        /// The text shown when the source fails, or nothing plays media.
        Message = "message" => "& > [data-slot='message']",
    }
}

/// The speeds the speed button steps through, as chat apps offer.
const SPEEDS: [f64; 3] = [1.0, 1.5, 2.0];

static AUDIO_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .min_width("0")
        // A voice message's bubble: a page-wide container does not stretch it.
        .width("100%")
        .max_width("22rem")
        .selector(
            AudioPart::Controls.selector(),
            controls_sx().border_radius(SizeCss::RADIUS.value(Size::Xl)),
        )
        .selector(
            AudioPart::Time.selector(),
            sx().font_variant_numeric("tabular-nums")
                .white_space("nowrap")
                .color(ColorCss::MUTED.value(ColorShade::S7)),
        )
        .selector(AudioPart::Seek.selector(), seek_sx())
        // Gives way only after the seek track is down to its minimum; then the row wraps.
        .selector(
            AudioPart::Volume.selector(),
            sx().flex("0 1 6rem")
                .min_width("4.5rem")
                .display("flex")
                .align_items("center")
                .gap("0.125rem")
                .color(ColorCss::MUTED.value(ColorShade::S7))
                .selector("& > svg", sx().flex_shrink("0").width("1em").height("1em"))
                .selector("& > div", sx().flex("1 1 0").min_width("0")),
        )
        .selector(
            AudioPart::Message.selector(),
            sx().color(ColorCss::MUTED.value(ColorShade::S7)),
        )
});

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
/// seek track, the time, volume and a speed button stepping 1×, 1.5× and 2×.
///
/// Keys while focus is inside: K, or Space on a slider, play and pause; J and L
/// jump 10 seconds.
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

    // No M: without a mute button nothing would show why it is silent.
    let jump = move |by: f64| media.seek(media.current_time() + by);
    use_hotkeys(
        [
            Hotkey::new("k", move || media.toggle()),
            Hotkey::new("j", move || jump(-10.0)),
            Hotkey::new("l", move || jump(10.0)),
        ]
        .map(|hotkey| hotkey.within(player)),
    );

    let sound = use_sound(media);
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
            AudioControls { media, sound, size: props.size.clone() }
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

/// One row: play, seek, time, volume, speed; then the error and buffering messages.
#[component]
fn AudioControls(media: MediaHandle, sound: Sound, size: Input<Size>) -> Element {
    let labels = use_localization().media;
    // Muted shows as 0, so a `muted` player says why it is silent; moving it up unmutes.
    let volume = if media.muted() {
        0.0
    } else {
        media.volume() * 100.0
    };
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
                MediaSeek { media, size: size.clone() }
            }
            AudioTime { media }
            div { "data-slot": VOLUME, onkeydown: space_toggles(media),
                // Tells the two tracks apart; the slider's name says it to a screen reader.
                if volume == 0.0 {
                    Glyph { slot: IconSlot::VolumeOff, icon: lucide::volume_x::outlined }
                } else {
                    Glyph { slot: IconSlot::Volume, icon: lucide::volume_2::outlined }
                }
                Slider::<f64> {
                    value: volume,
                    min: 0.0,
                    max: 100.0,
                    step: 5.0,
                    size: size.clone(),
                    aria_label: labels.volume,
                    oninput: move |event: SliderChangeEvent| sound.set_volume(event.value() / 100.0),
                }
            }
            AudioSpeed { media, size }
        }
        MediaStatus { media }
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
