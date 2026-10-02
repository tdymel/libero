use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;

use super::media_controls::{Captions, MediaControls, MediaFallback, use_media_keys, use_sound};
use super::{
    MediaPreload, MediaSource,
    audio::{media_sources, use_source_reload},
};
use crate::{
    components::{
        common::{HtmlTag, Input, Part, base_props, parts_enum, use_name_warning},
        layout::use_box,
        overlay::Shortcut,
    },
    context::{HostOutlet, PortalHost},
    hooks::{
        FULLSCREEN_ATTR, Hotkey, MediaError, MediaHandle, listener, use_element, use_fullscreen,
        use_localization, use_media, use_portal_slot, use_scroll_lock, use_timeout,
    },
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
        /// The seek slider's wrapper.
        Seek = "seek" => "& [data-slot='seek']",
        /// The mute button and the volume menu's trigger.
        Volume = "volume" => "& [data-slot='volume']",
        /// The text shown when the source fails, or nothing plays media.
        Message = "message" => "& > [data-slot='message']",
    }
}

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

static VIDEO_SX: StaticSx = StaticSx::new(|| {
    let controls = VideoPart::Controls.selector();
    let media = VideoPart::Media.selector();
    let filled = sx().background_color(BLACK).selector(
        VideoPart::Media.selector(),
        sx().height("100%").aspect_ratio("auto"),
    );
    let light_track = sx().background(LIGHT_TRACK);
    sx().position("relative")
        // A size container has no content width: in a shrink-wrapping parent it collapsed (todo 1369).
        .width("100%")
        .with("contain-intrinsic-inline-size", "40rem")
        .min_width("0")
        .container(PLAYER_CONTAINER)
        .selector(
            VideoPart::Media.selector(),
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
            format!("&[{CONTROLS_ATTR}='hidden'] {}{CUES}", &media[2..]),
            sx().transform("none"),
        )
        // YouTube's bar: over the bottom of the picture on a black scrim, white
        // text and icons, the seek track a row of its own above the buttons.
        .selector(
            VideoPart::Controls.selector(),
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
                .selector("& [data-slot='track']", light_track)
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
                VideoPart::Controls.selector(),
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
/// jump 10 seconds, M mutes, C toggles captions, F fullscreen; Shift+? lists them.
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

    // The track the captions button shows: the one last picked in the track menu,
    // else the default text one, else the first.
    let text_tracks: Vec<(usize, &MediaTrack)> = props
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.kind.shows_text())
        .collect();
    let picked = use_signal(|| None::<usize>);
    let caption_track = picked()
        .filter(|picked| text_tracks.iter().any(|(index, _)| index == picked))
        .or_else(|| {
            text_tracks
                .iter()
                .find(|(_, t)| t.default)
                .or(text_tracks.first())
                .map(|(index, _)| *index)
        });
    let shown_at_start = text_tracks.iter().any(|(_, t)| t.default);
    let shown = use_signal(|| shown_at_start);
    let captions = Captions {
        shown,
        track: caption_track,
        picked,
    };
    // Two or more: the captions button opens a menu of them (todo 1284).
    let caption_tracks: Vec<(usize, String)> = match text_tracks.len() {
        0 | 1 => Vec::new(),
        _ => text_tracks
            .iter()
            .map(|(index, t)| {
                let name = if t.label.is_empty() {
                    &t.srclang
                } else {
                    &t.label
                };
                (*index, name.clone())
            })
            .collect(),
    };

    let sound = use_sound(media);
    // After the `PortalHost`: in fullscreen the help shows inside the player.
    let labels = use_localization().media;
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
        ],
    );
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
    // The bar's height over the picture, bar the scrim's fade: measured (todo 1389), and
    // until then a button row, a 1rem seek row and the gaps.
    let mut bar = use_signal(|| None::<f64>);
    let mut style = match bar() {
        Some(height) => format!("{BAR_VAR}: calc({height}px - {SCRIM_FADE});"),
        None => {
            let button = match props.size.as_ref() {
                Some(size) => BUTTON_HEIGHT.value(*size),
                None => ACTION_ICON_SIZE.value(),
            };
            format!(
                "{BAR_VAR}: calc({button} + 1rem + 3 * {});",
                SizeCss::SPACING.value(Size::Xs)
            )
        }
    };
    if let Some(ratio) = &props.aspect_ratio {
        style.push_str(&format!("aspect-ratio: {ratio};"));
    }

    // The controls overlay the picture and fade out while playing untouched, or
    // as a mouse leaves; a pointer or a key brings them back. A key keeps them until
    // the next press of a pointer, so keyboard focus never sits on an unseen control.
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
    let playing = !media.paused();
    use_effect(use_reactive!(|playing| {
        if playing {
            wake();
        }
    }));

    let mut pressing = use_hook(|| CopyValue::new(false));
    let by_key = move || {
        let mut keyboard = keyboard;
        if !*keyboard.peek() {
            keyboard.set(true);
        }
        wake();
    };
    // Tagged, so a WebView finds the player for fullscreen and the hotkeys.
    let mut attributes = props.attributes;
    // Focus moved in without a press, by a key, code or AT, counts as a key (todo 1249).
    attributes.extend(fullscreen.attributes_with_focusin(move |_| {
        if playing && !*pressing.peek() {
            by_key();
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
        listener("onpointerup", released),
        listener("onpointercancel", released),
        listener("onpointerleave", move |event: Event<PointerData>| {
            released(event.clone());
            // A mouse only: a touch leaves after every tap.
            if playing && event.pointer_type() == "mouse" {
                idle_timer.stop();
                idle.set(true);
            }
        }),
    ]);
    // Only while playing, so a WebView sends no pointer moves across the IPC otherwise.
    if playing {
        attributes.extend([
            listener("onpointermove", move |_: Event<PointerData>| wake()),
            listener("onpointerdown", move |_: Event<PointerData>| {
                pressing.set(true);
                let mut keyboard = keyboard;
                if *keyboard.peek() {
                    keyboard.set(false);
                }
                wake();
            }),
            listener("onkeyup", move |_: Event<KeyboardData>| by_key()),
        ]);
    }
    let hidden = playing && idle() && !keyboard();
    // A click on the picture plays or pauses, as YouTube's; a tap on hidden controls only shows them.
    let mut tap_shows = use_hook(|| CopyValue::new(false));
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
            poster: props.poster.clone(),
            style,
            autoplay: props.autoplay,
            muted: props.muted,
            "loop": props.looping,
            preload: props.preload.attribute(),
            playsinline: true,
            onmounted: media.mount(),
            onpointerdown: move |event: PointerEvent| tap_shows.set(event.pointer_type() == "touch" && hidden),
            onclick: move |_| {
                if !*tap_shows.peek() {
                    media.toggle();
                }
                tap_shows.set(false);
            },
            onplay: move |_| if let Some(handler) = onplay { handler.call(()) },
            onpause: move |_| if let Some(handler) = onpause { handler.call(()) },
            onended: move |_| if let Some(handler) = onended { handler.call(()) },
            ..media.attributes(),
            {sources}
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
            MediaControls {
                media,
                sound,
                size: props.size.clone(),
                captions: Some(captions),
                caption_tracks,
                fullscreen: Some(fullscreen),
                overlay: true,
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
