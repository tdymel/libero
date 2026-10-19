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

/// Todo 3844: the mute button keeps one name and says its state with `aria-pressed`.
#[test]
fn the_mute_button_keeps_its_name_and_says_whether_it_is_on() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Audio { src: "/a.ogg", label: "Episode 1" } }
        }
    }

    let html = body(&render(app));
    assert!(!html.contains(r#"aria-label="Unmute""#), "{html}");
    let at = html.find(r#"aria-label="Mute""#).expect(&html);
    let tag = &html[html[..at].rfind("<button").unwrap()..];
    let tag = &tag[..tag.find('>').unwrap()];
    assert!(tag.contains(r#"aria-pressed="false""#), "{tag}");
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
    // A mute button and the volume menu's closed trigger (todo 1385), one time value.
    assert!(html.contains(r#"aria-label="Stummschalten""#), "{html}");
    assert!(
        html.contains(r#"aria-label="Lautstärke""#) && html.contains(r#"aria-haspopup="dialog""#),
        "{html}"
    );
    assert!(!html.contains(r#"role="dialog""#), "{html}");
    assert!(html.contains(">--:--<") && !html.contains(" / "), "{html}");
    assert!(html.contains(r#"role="status""#), "{html}");
}
