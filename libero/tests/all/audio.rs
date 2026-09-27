//! `Audio`'s server render: a named group, a bare `<audio>` (no native controls),
//! and libero's own controls, idle until the element reports.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Audio, MediaPreload},
    localization::{Localization, MediaLabels},
};

static GERMAN: Localization = Localization {
    media: MediaLabels::GERMAN,
    ..Localization::ENGLISH
};

#[test]
fn the_player_is_a_named_group_round_a_bare_audio_element() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Audio {
                    src: "/a.ogg",
                    label: "Episode 1",
                    muted: true,
                    looping: true,
                    preload: MediaPreload::None,
                }
            }
        }
    }

    let html = body(&render(app));

    assert!(html.contains(r#"role="group""#), "{html}");
    assert!(html.contains(r#"aria-label="Episode 1""#), "{html}");
    let audio = attributes_of(&html, "audio");
    assert_eq!(audio["src"], "/a.ogg");
    assert_eq!(audio["preload"], "none");
    assert!(audio.contains_key("muted"), "{html}");
    assert!(audio.contains_key("loop"), "{html}");
    assert!(!audio.contains_key("controls"), "{html}");
}

#[test]
fn the_controls_start_paused_with_an_unknown_duration() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &GERMAN, Audio { src: "/a.ogg", label: "Folge 1" } }
        }
    }

    let html = body(&render(app));

    // A named group of plain Tab stops, not a roving toolbar (todo 1328).
    assert!(!html.contains(r#"role="toolbar""#), "{html}");
    assert!(
        html.contains(r#"role="group" aria-label="Wiedergabesteuerung""#),
        "{html}"
    );
    assert!(
        html.contains(r#"aria-label="Wiedergabegeschwindigkeit 1×""#) && html.contains(">1×<"),
        "the speed button shows and names the rate: {html}"
    );
    assert!(html.contains(r#"aria-label="Abspielen""#), "{html}");
    assert!(html.contains(r#"aria-label="Position""#), "{html}");
    // No mute button, the volume slider stays, one time value.
    assert!(!html.contains(r#"aria-label="Stummschalten""#), "{html}");
    assert!(html.contains(r#"aria-label="Lautstärke""#), "{html}");
    assert!(html.contains(">--:--<") && !html.contains(" / "), "{html}");
    assert!(html.contains(r#"role="status""#), "{html}");
}
