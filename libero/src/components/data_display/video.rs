use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

use super::MediaPreload;
use super::media_controls::{Captions, MediaControls, MediaFallback, use_media_keys};
use crate::{
    components::{
        common::{HtmlTag, Input, Part, base_props, parts_enum, use_name_warning},
        layout::use_box,
    },
    hooks::{
        FULLSCREEN_ATTR, Hotkey, MediaError, MediaHandle, listener, use_element, use_fullscreen,
        use_media, use_timeout,
    },
    sx::{StaticSx, sx},
    theme::{ColorCss, ColorShade, NamedColorCss, Size, SizeCss, Z_INDEX_MODAL},
    utils::warn,
};

parts_enum! {
    /// [`Video`]'s inner parts, for its `parts` prop.
    pub enum VideoPart {
        /// The `<video>` element.
        Media = "media" => "& > [data-slot='media']",
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

/// Set on the player while its fullscreen controls are faded out.
const CONTROLS_ATTR: &str = "data-controls";
/// How long the fullscreen controls stay after the last pointer move or key.
const CONTROLS_IDLE_MS: u64 = 3000;
const CONTROLS_CONTAINER: &str = "libero-video-controls";
const NARROW: &str = "(max-width: 28rem)";
const NARROWEST: &str = "(max-width: 22rem)";

static VIDEO_SX: StaticSx = StaticSx::new(|| {
    let controls = VideoPart::Controls.selector();
    // The themed surface, not a dark scrim, so the controls keep their contrast.
    let overlay = sx()
        .position("absolute")
        .inset("auto 0 0 0")
        .padding(SizeCss::SPACING.value(Size::Xs))
        .background_color(format!(
            "color-mix(in srgb, {} 85%, transparent)",
            NamedColorCss::SURFACE.value()
        ))
        .transition("opacity 200ms ease")
        .media("(prefers-reduced-motion: reduce)", sx().transition("none"));
    let filled = sx()
        .background_color(NamedColorCss::SURFACE.value())
        .padding(SizeCss::SPACING.value(Size::Xs))
        .selector(
            VideoPart::Media.selector(),
            sx().flex("1 1 0").min_height("0").aspect_ratio("auto"),
        );
    sx().display("flex")
        .flex_direction("column")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .min_width("0")
        .selector(
            VideoPart::Media.selector(),
            sx().display("block")
                .width("100%")
                // Letterboxing stays black in either scheme, as every player draws it.
                .background_color("#000")
                .object_fit("contain"),
        )
        .selector(
            VideoPart::Controls.selector(),
            sx().flex_wrap("nowrap").container(CONTROLS_CONTAINER),
        )
        // A phone's width: the mute button stays, the volume slider goes (its
        // hardware keys remain), then the total time, so the row fits at 320px.
        .selector(
            VideoPart::Time.selector(),
            sx().font_variant_numeric("tabular-nums")
                .white_space("nowrap")
                .flex_shrink("0")
                .selector(
                    "& > span",
                    sx().container_query(CONTROLS_CONTAINER, NARROWEST, sx().display("none")),
                ),
        )
        .selector(
            VideoPart::Seek.selector(),
            sx().flex("1 1 8rem").min_width("6rem").container_query(
                CONTROLS_CONTAINER,
                NARROW,
                sx().min_width("4rem"),
            ),
        )
        .selector(
            VideoPart::Volume.selector(),
            sx().flex("0 1 6rem").min_width("4rem").container_query(
                CONTROLS_CONTAINER,
                NARROW,
                sx().display("none"),
            ),
        )
        .selector(
            VideoPart::Message.selector(),
            sx().color(ColorCss::MUTED.value(ColorShade::S7)),
        )
        .selector("&:fullscreen", filled.clone())
        .selector(
            format!("&[{FULLSCREEN_ATTR}='drawn']"),
            filled
                .position("fixed")
                .inset("0")
                .z_index(Z_INDEX_MODAL.value()),
        )
        .selector(format!("&[{FULLSCREEN_ATTR}] {}", &controls[2..]), overlay)
        .selector(format!("&[{CONTROLS_ATTR}='hidden']"), sx().cursor("none"))
        .selector(
            format!("&[{CONTROLS_ATTR}='hidden'] {}", &controls[2..]),
            sx().opacity("0"),
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

base_props! {
    parts(VideoPart);
    pub struct VideoProps {
        /// The file's URL.
        #[props(into)]
        src: String,
        /// Names the player, e.g. the video's title.
        #[props(into)]
        label: String,
        /// A picture shown until playing starts.
        #[props(default, into)]
        poster: Option<String>,
        /// The picture's CSS `aspect-ratio`, such as `"16 / 9"`. Unset, the file's own.
        #[props(default, into)]
        aspect_ratio: Option<String>,
        /// Captions, subtitles and more, as WebVTT files.
        #[props(default)]
        tracks: Vec<MediaTrack>,
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

/// A video player with its own controls: play, seek, time, mute, volume,
/// captions and fullscreen.
///
/// Keys while focus is inside: K, or Space on a slider, play and pause; J and L
/// jump 10 seconds, M mutes, C toggles captions, F fullscreen.
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

    // The track the captions button shows: the default text one, else the first.
    let text_tracks = props
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.kind.shows_text());
    let caption_track = text_tracks
        .clone()
        .find(|(_, t)| t.default)
        .or_else(|| text_tracks.clone().next())
        .map(|(index, _)| index);
    let shown_at_start = text_tracks.clone().any(|(_, t)| t.default);
    let shown = use_signal(|| shown_at_start);
    let captions = caption_track.map(|track| Captions { shown, track });

    use_media_keys(
        media,
        player,
        [
            Hotkey::new("f", move || fullscreen.toggle()),
            Hotkey::new("c", move || {
                if let Some(captions) = captions {
                    captions.toggle(media);
                }
            })
            .when(move || captions.is_some()),
        ],
    );

    let error = media.error();
    let onerror = props.onerror;
    use_effect(use_reactive!(|error| {
        if let (Some(error), Some(onerror)) = (error, onerror) {
            onerror.call(error);
        }
    }));

    let (onplay, onpause, onended) = (props.onplay, props.onpause, props.onended);
    let unsupported = media.supported() == Some(false);
    let aspect_ratio = props
        .aspect_ratio
        .as_ref()
        .map(|ratio| format!("aspect-ratio: {ratio}"));

    // In fullscreen the controls overlay the picture and fade out while playing
    // untouched; a pointer or a key brings them back. A key keeps them until the
    // next press of a pointer, so keyboard focus never sits on an unseen control.
    let mut idle = use_signal(|| false);
    let keyboard = use_signal(|| false);
    let idle_timer = use_timeout(move || idle.set(true), CONTROLS_IDLE_MS);
    let wake = move || {
        let mut idle = idle;
        if *idle.peek() {
            idle.set(false);
        }
        idle_timer.start();
    };
    let (filling, playing) = (fullscreen.is_fullscreen(), !media.paused());
    use_effect(use_reactive!(|filling, playing| {
        if filling && playing {
            wake();
        }
    }));

    // Tagged, so a WebView finds the player for fullscreen and the hotkeys.
    let mut attributes = props.attributes;
    attributes.extend(fullscreen.attributes());
    // Only in fullscreen, so a WebView sends no pointer moves across the IPC otherwise.
    if filling {
        attributes.extend([
            listener("onpointermove", move |_: Event<PointerData>| wake()),
            listener("onpointerdown", move |_: Event<PointerData>| {
                let mut keyboard = keyboard;
                if *keyboard.peek() {
                    keyboard.set(false);
                }
                wake();
            }),
            listener("onkeyup", move |_: Event<KeyboardData>| {
                let mut keyboard = keyboard;
                if !*keyboard.peek() {
                    keyboard.set(true);
                }
                wake();
            }),
        ]);
    }
    if filling && playing && idle() && !keyboard() {
        attributes.push(Attribute::new(
            CONTROLS_ATTR,
            AttributeValue::Text("hidden".into()),
            None,
            false,
        ));
    }

    let body = rsx! {
        video {
            "data-slot": VideoPart::Media.slot(),
            src: props.src.clone(),
            poster: props.poster.clone(),
            style: aspect_ratio,
            autoplay: props.autoplay,
            muted: props.muted,
            "loop": props.looping,
            preload: props.preload.attribute(),
            playsinline: true,
            onmounted: media.mount(),
            onplay: move |_| if let Some(handler) = onplay { handler.call(()) },
            onpause: move |_| if let Some(handler) = onpause { handler.call(()) },
            onended: move |_| if let Some(handler) = onended { handler.call(()) },
            ..media.attributes(),
            for track in props.tracks.iter() {
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
            MediaControls { media, size: props.size.clone(), captions, fullscreen: Some(fullscreen) }
        }
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
    use super::super::media_controls::{CONTROLS, MESSAGE, SEEK, TIME, VOLUME};
    use super::*;
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
                ("seek", "& [data-slot='seek']"),
                ("volume", "& [data-slot='volume']"),
                ("message", "& > [data-slot='message']"),
            ]
        );
    }

    #[test]
    fn the_shared_controls_use_the_part_names() {
        assert_eq!(
            [CONTROLS, TIME, SEEK, VOLUME, MESSAGE],
            [
                VideoPart::Controls.slot(),
                VideoPart::Time.slot(),
                VideoPart::Seek.slot(),
                VideoPart::Volume.slot(),
                VideoPart::Message.slot(),
            ]
        );
    }
}
