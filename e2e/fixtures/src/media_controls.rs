//! A posterised `Video` and an `Audio` where media plays; the unsupported fallback is native.

use dioxus::prelude::*;
use libero::components::{Audio, Video};

use crate::Routes;
use crate::audio::silent_wav;

const POSTER: &str = "data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 9' width='16' height='9'><rect width='16' height='9' fill='blue'/></svg>";

pub const ROUTES: Routes = &[("/media-controls", || rsx! { MediaControlsPage {} })];

#[component]
fn MediaControlsPage() -> Element {
    let src = use_hook(|| silent_wav(4));
    rsx! {
        div { id: "video", style: "width: 32rem",
            Video { src: src.clone(), label: "Clip", poster: POSTER }
        }
        div { id: "audio", style: "width: 32rem",
            Audio { src, label: "Clip" }
        }
    }
}
