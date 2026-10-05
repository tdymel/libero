//! Media the docs demos show.

use dioxus::prelude::*;

/// A 16:9 landscape, for docs examples where the logo's square shape hides
/// what the example is about.
pub(crate) static SAMPLE_IMAGE: Asset = asset!("/assets/sample.svg");
/// "Wikipedia guitar solo", CC0, from Wikimedia Commons: no binary in the repo.
pub(crate) const SAMPLE_AUDIO: &str = "https://upload.wikimedia.org/wikipedia/commons/transcoded/b/b6/Wikipedia_guitar_solo.ogg/Wikipedia_guitar_solo.ogg.mp3";
/// "Big Buck Bunny", CC BY 3.0 Blender Foundation, a 480p transcode from Wikimedia Commons.
pub(crate) const SAMPLE_VIDEO: &str = "https://upload.wikimedia.org/wikipedia/commons/transcoded/c/c0/Big_Buck_Bunny_4K.webm/Big_Buck_Bunny_4K.webm.480p.vp9.webm";
/// One placeholder cue over the whole film, so the captions button shows something
/// wherever it is pressed; inline, so no `.vtt` file ships. Not the film's sound.
pub(crate) const SAMPLE_CAPTIONS: &str = "data:text/vtt,WEBVTT%0A%0A00:00.000 --> 10:00.000%0A[Placeholder caption, not the film's sound]";
/// The demo's second track, so its captions button shows the track menu.
pub(crate) const SAMPLE_UNTERTITEL: &str =
    "data:text/vtt,WEBVTT%0A%0A00:00.000 --> 10:00.000%0A[Platzhalter, nicht der Ton des Films]";
/// Its Commons poster frame.
pub(crate) const SAMPLE_POSTER: &str = "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c0/Big_Buck_Bunny_4K.webm/960px--Big_Buck_Bunny_4K.webm.jpg";
/// Stands in for a source that failed to load.
pub(crate) static FALLBACK_IMAGE: Asset = asset!("/assets/fallback.svg");
