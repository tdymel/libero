use dioxus::prelude::*;

use super::media_controls::{MediaControls, MediaFallback, use_media_keys};
use crate::{
    components::{
        common::{HtmlTag, Input, Part, base_props, parts_enum, use_name_warning},
        layout::use_box,
    },
    hooks::{MediaError, MediaHandle, use_media},
    sx::{StaticSx, sx},
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
    pub(super) fn attribute(self) -> &'static str {
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

    use_media_keys(media, player, []);

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
            MediaFallback { src: props.src.clone(), children: props.children }
        } else {
            MediaControls { media, size: props.size.clone() }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::common::part_table;

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
