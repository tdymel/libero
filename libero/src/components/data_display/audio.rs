use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::{ActionIcon, Toolbar},
        common::{Glyph, HtmlTag, Input, Part, base_props, parts_enum, use_name_warning},
        form::{Slider, SliderChangeEvent},
        layout::use_box,
    },
    context::IconSlot,
    hooks::{Hotkey, MediaError, MediaHandle, use_hotkeys, use_localization, use_media},
    localization::fill,
    platform::key_taken,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{ColorCss, ColorShade, Size, SizeCss},
    utils::warn,
};

parts_enum! {
    /// [`Audio`]'s inner parts, for its `parts` prop.
    pub enum AudioPart {
        /// The row of controls.
        Controls = "controls" => "& > [data-slot='controls']",
        /// The elapsed and total time.
        Time = "time" => "& [data-slot='time']",
        /// The seek slider's wrapper.
        Seek = "seek" => "& [data-slot='seek']",
        /// The volume slider's wrapper.
        Volume = "volume" => "& [data-slot='volume']",
        /// The text shown when the source fails, or nothing plays media.
        Message = "message" => "& > [data-slot='message']",
    }
}

static AUDIO_SX: StaticSx = StaticSx::new(|| {
    sx().display("flex")
        .flex_direction("column")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .min_width("0")
        .selector(AudioPart::Controls.selector(), sx().flex_wrap("nowrap"))
        .selector(
            AudioPart::Time.selector(),
            sx().font_variant_numeric("tabular-nums")
                .white_space("nowrap")
                .flex_shrink("0"),
        )
        .selector(
            AudioPart::Seek.selector(),
            sx().flex("1 1 8rem").min_width("6rem"),
        )
        .selector(
            AudioPart::Volume.selector(),
            sx().flex("0 1 6rem").min_width("4rem"),
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
    fn attribute(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Metadata => "metadata",
            Self::Auto => "auto",
        }
    }
}

base_props! {
    parts(AudioPart);
    pub struct AudioProps {
        /// The file's URL.
        #[props(into)]
        src: String,
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
        /// Of the buttons and sliders.
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

/// An audio player with its own controls: play, seek, time, mute and volume.
///
/// Keys while focus is inside: K, or Space on a slider, play and pause; J and L
/// jump 10 seconds, M mutes.
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
    let labels = use_localization().media;
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

    let jump = move |by: f64| media.seek(media.current_time() + by);
    use_hotkeys([
        Hotkey::new("k", move || media.toggle()).within(player),
        Hotkey::new("j", move || jump(-10.0)).within(player),
        Hotkey::new("l", move || jump(10.0)).within(player),
        Hotkey::new("m", move || media.set_muted(!media.muted())).within(player),
    ]);
    // Space on a slider plays and pauses; a button keeps its own Space, so no hotkey.
    let space_toggles = move |event: KeyboardEvent| {
        if event.key() == Key::Character(" ".into()) && !key_taken(&event) {
            event.prevent_default();
            media.toggle();
        }
    };

    let error = media.error();
    let onerror = props.onerror;
    use_effect(use_reactive!(|error| {
        if let (Some(error), Some(onerror)) = (error, onerror) {
            onerror.call(error);
        }
    }));

    let (onplay, onpause, onended) = (props.onplay, props.onpause, props.onended);
    let unsupported = media.supported() == Some(false);
    let size = props.size.clone();
    let icon_size: Input<ThemeAwareValue> = match props.size.as_ref() {
        Some(size) => Input::Value(ThemeAwareValue::Size(*size)),
        None => Input::None,
    };

    let body = rsx! {
        audio {
            src: props.src.clone(),
            autoplay: props.autoplay,
            muted: props.muted,
            "loop": props.looping,
            preload: props.preload.attribute(),
            onmounted: media.mount(),
            onplay: move |_| if let Some(handler) = onplay { handler.call(()) },
            onpause: move |_| if let Some(handler) = onpause { handler.call(()) },
            onended: move |_| if let Some(handler) = onended { handler.call(()) },
            ..media.attributes(),
        }
        if unsupported {
            div { "data-slot": AudioPart::Message.slot(),
                if props.children != VNode::empty() {
                    {props.children.clone()}
                } else {
                    p { {labels.unsupported} }
                    a { href: props.src.clone(), {labels.download} }
                }
            }
        } else {
            Toolbar { "aria-label": labels.controls, "data-slot": AudioPart::Controls.slot(),
                ActionIcon {
                    aria_label: if media.paused() { labels.play } else { labels.pause },
                    size: icon_size.clone(),
                    onclick: move |_| media.toggle(),
                    if media.paused() {
                        Glyph { slot: IconSlot::Play, icon: lucide::play::outlined }
                    } else {
                        Glyph { slot: IconSlot::Pause, icon: lucide::pause::outlined }
                    }
                }
                AudioTime { media }
                div { "data-slot": AudioPart::Seek.slot(), onkeydown: space_toggles,
                    AudioSeek { media, size: size.clone() }
                }
                ActionIcon {
                    aria_label: if media.muted() { labels.unmute } else { labels.mute },
                    size: icon_size.clone(),
                    onclick: move |_| media.set_muted(!media.muted()),
                    icon: if media.muted() || media.volume() == 0.0 {
                        lucide::volume_x::outlined
                    } else {
                        lucide::volume_2::outlined
                    },
                }
                div { "data-slot": AudioPart::Volume.slot(), onkeydown: space_toggles,
                    Slider::<f64> {
                        value: media.volume() * 100.0,
                        min: 0.0,
                        max: 100.0,
                        step: 5.0,
                        size: size.clone(),
                        aria_label: labels.volume,
                        oninput: move |event: SliderChangeEvent| media.set_volume(event.value() / 100.0),
                    }
                }
            }
            if error.is_some() {
                p { "data-slot": AudioPart::Message.slot(), role: "alert", {labels.error} }
            }
            // Mounted throughout, so a screen reader hears the change.
            VisuallyHidden { role: "status",
                if media.buffering() { {labels.buffering} }
            }
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
        .render(HtmlTag::Div, props.attributes, body)
}

/// Elapsed and total time; its own component, so a time tick re-renders only this.
#[component]
fn AudioTime(media: MediaHandle) -> Element {
    let time = clock(media.current_time());
    let total = media.duration().map_or_else(|| "--:--".to_string(), clock);
    rsx! {
        span { "data-slot": AudioPart::Time.slot(), "aria-hidden": "true", "{time} / {total}" }
    }
}

/// The seek slider. While dragged it shows the thumb's value, not the element's
/// older answer, which trails on a WebView.
#[component]
fn AudioSeek(media: MediaHandle, size: Input<Size>) -> Element {
    let labels = use_localization().media;
    let mut scrub = use_signal(|| None::<f64>);
    let duration = media.duration();
    let value = scrub().unwrap_or_else(|| media.current_time());
    let total = duration.map_or_else(|| "--:--".to_string(), clock);
    let format = use_callback(move |seconds: f64| {
        fill(
            labels.position,
            &[("time", &clock(seconds)), ("duration", &total)],
        )
    });
    rsx! {
        Slider::<f64> {
            value,
            min: 0.0,
            max: duration.unwrap_or(1.0).max(1.0),
            step: 1.0,
            size,
            disabled: duration.is_none(),
            aria_label: labels.seek,
            format,
            oninput: move |event: SliderChangeEvent| match event {
                SliderChangeEvent::Start(seconds) => scrub.set(Some(seconds)),
                SliderChangeEvent::Change(seconds) => {
                    if scrub.peek().is_some() {
                        scrub.set(Some(seconds));
                    }
                    media.seek(seconds);
                }
                SliderChangeEvent::End(_) => scrub.set(None),
            },
        }
    }
}

/// `m:ss`, or `h:mm:ss` from an hour.
fn clock(seconds: f64) -> String {
    let total = seconds.max(0.0).floor() as u64;
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    match hours {
        0 => format!("{minutes}:{seconds:02}"),
        _ => format!("{hours}:{minutes:02}:{seconds:02}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

    #[test]
    fn a_clock_shows_hours_only_from_an_hour() {
        assert_eq!(clock(0.0), "0:00");
        assert_eq!(clock(65.9), "1:05");
        assert_eq!(clock(3599.0), "59:59");
        assert_eq!(clock(3725.0), "1:02:05");
        assert_eq!(clock(-3.0), "0:00");
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
