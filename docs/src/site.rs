use dioxus::prelude::*;

pub(crate) static LOGO: Asset = asset!("/assets/logo.svg");
/// The logo's svg in the text color, for the header.
pub(crate) static LOGO_INLINE: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    include_str!("../assets/logo.svg").replace("#228be6", "currentColor")
});
pub(crate) const REPO: &str = "tdymel/libero";
pub(crate) const GITHUB: &str = "https://github.com/tdymel/libero";
/// A 16:9 landscape, for docs examples where the logo's square shape hides
/// what the example is about.
pub(crate) static SAMPLE_IMAGE: Asset = asset!("/assets/sample.svg");
/// "Wikipedia guitar solo", CC0, from Wikimedia Commons: no binary in the repo.
pub(crate) const SAMPLE_AUDIO: &str = "https://upload.wikimedia.org/wikipedia/commons/transcoded/b/b6/Wikipedia_guitar_solo.ogg/Wikipedia_guitar_solo.ogg.mp3";
/// "Big Buck Bunny", CC BY 3.0 Blender Foundation, a 480p transcode from Wikimedia Commons.
pub(crate) const SAMPLE_VIDEO: &str = "https://upload.wikimedia.org/wikipedia/commons/transcoded/c/c0/Big_Buck_Bunny_4K.webm/Big_Buck_Bunny_4K.webm.480p.vp9.webm";
/// Its Commons poster frame.
pub(crate) const SAMPLE_POSTER: &str = "https://upload.wikimedia.org/wikipedia/commons/thumb/c/c0/Big_Buck_Bunny_4K.webm/960px--Big_Buck_Bunny_4K.webm.jpg";
/// Stands in for a source that failed to load.
pub(crate) static FALLBACK_IMAGE: Asset = asset!("/assets/fallback.svg");
