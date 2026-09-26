//! `Video` over a generated 4 s silent WAV (no codec to ship), with an inline
//! subtitle track, plus one without tracks.

use dioxus::prelude::*;
use libero::components::{Flex, MediaTrack, Text, TrackKind, Video};

use crate::Routes;
use crate::audio::silent_wav;

pub const ROUTES: Routes = &[
    ("/video", || rsx! { VideoPage {} }),
    ("/video/refused", || rsx! { VideoPage { refused: true } }),
    // Long enough for the fullscreen controls to fade out while playing.
    ("/video/long", || rsx! { VideoPage { seconds: 15 } }),
];

const SUBTITLES: &str = "data:text/vtt,WEBVTT%0A%0A00:00.000 --> 00:04.000%0ASilence";

/// The player refuses the Fullscreen API, as a WebView without it does; marks `#player` once set.
const REFUSE: &str = "const player = document.querySelector('#player [role=group]');
    player.requestFullscreen = () => Promise.reject(new TypeError('refused'));
    document.querySelector('#player').dataset.refused = 'true';";

#[component]
fn VideoPage(#[props(default)] refused: bool, #[props(default = 4)] seconds: u32) -> Element {
    let src = use_hook(|| silent_wav(seconds));
    use_effect(move || {
        if refused {
            document::eval(REFUSE);
        }
    });
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "32rem",
            Text { "A silent clip with subtitles, and one without." }
            div { id: "player",
                Video {
                    src: src.clone(),
                    label: "Silence",
                    aspect_ratio: "16 / 9",
                    tracks: vec![MediaTrack {
                        src: SUBTITLES.into(),
                        kind: TrackKind::Subtitles,
                        srclang: "en".into(),
                        label: "English".into(),
                        default: false,
                    }],
                }
            }
            div { id: "plain", Video { src, label: "Plain" } }
        }
    }
}
