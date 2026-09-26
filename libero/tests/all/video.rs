//! `Video`'s server render: a named group, a bare `<video>` with its poster and
//! tracks, and libero's controls with captions and fullscreen buttons.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{MediaSource, MediaTrack, TrackKind, Video},
    localization::{Localization, MediaLabels},
};

static GERMAN: Localization = Localization {
    media: MediaLabels::GERMAN,
    ..Localization::ENGLISH
};

#[test]
fn the_player_is_a_named_group_round_a_bare_video_element() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Video {
                    src: "/a.webm",
                    label: "Launch",
                    poster: "/a.jpg",
                    aspect_ratio: "16 / 9",
                    tracks: vec![MediaTrack {
                        src: "/a.en.vtt".into(),
                        kind: TrackKind::Subtitles,
                        srclang: "en".into(),
                        label: "English".into(),
                        default: true,
                    }],
                }
            }
        }
    }

    let html = body(&render(app));

    assert!(html.contains(r#"role="group""#), "{html}");
    assert!(html.contains(r#"aria-label="Launch""#), "{html}");
    let video = attributes_of(&html, "video");
    assert_eq!(video["src"], "/a.webm");
    assert_eq!(video["poster"], "/a.jpg");
    assert_eq!(video["data-slot"], "media");
    assert!(video["style"].contains("aspect-ratio: 16 / 9"), "{html}");
    assert!(!video.contains_key("controls"), "{html}");
    let track = attributes_of(&html, "track");
    assert_eq!(track["kind"], "subtitles");
    assert_eq!(track["srclang"], "en");
    assert!(track.contains_key("default"), "{html}");
    // A default track starts shown, so the toggle starts pressed.
    assert!(html.contains(r#"aria-pressed="true""#), "{html}");
}

/// Todo 1247. With `sources`, `<source>` children in order, `src` last and
/// untyped, and no `src` attribute that would win over them; tracks after.
#[test]
fn sources_render_in_order_before_src_and_the_tracks() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Video {
                    src: "/a.ogv",
                    sources: vec![
                        MediaSource::new("/a.webm", "video/webm"),
                        MediaSource::new("/a.mp4", "video/mp4"),
                    ],
                    label: "Launch",
                    tracks: vec![MediaTrack { src: "/a.vtt".into(), ..Default::default() }],
                }
            }
        }
    }

    let html = body(&render(app));

    assert!(!attributes_of(&html, "video").contains_key("src"), "{html}");
    let order: Vec<usize> = [
        r#"<source src="/a.webm" type="video/webm""#,
        r#"<source src="/a.mp4" type="video/mp4""#,
        r#"<source src="/a.ogv""#,
        "<track",
    ]
    .iter()
    .map(|tag| html.find(tag).unwrap_or_else(|| panic!("{tag} in {html}")))
    .collect();
    assert!(order.is_sorted(), "{html}");
    assert!(!html.contains(r#"src="/a.ogv" type"#), "{html}");
}

#[test]
fn the_controls_add_fullscreen_and_no_captions_button_without_a_track() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &GERMAN, Video { src: "/a.webm", label: "Start" } }
        }
    }

    let html = body(&render(app));

    assert!(
        html.contains(r#"aria-label="Wiedergabesteuerung""#),
        "{html}"
    );
    assert!(html.contains(r#"aria-label="Abspielen""#), "{html}");
    assert!(html.contains(r#"aria-label="Vollbild""#), "{html}");
    assert!(!html.contains(r#"aria-label="Untertitel""#), "{html}");
    assert!(html.contains("0:00") && html.contains(" / --:--"), "{html}");
    assert!(!html.contains("data-fullscreen"), "{html}");
}
