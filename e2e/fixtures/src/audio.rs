//! `Audio` over a generated 4 s silent WAV behind an unplayable `<source>`, a broken source, and `use_media` on a bare element.

use dioxus::prelude::*;
use libero::components::{Audio, Button, Flex, MediaSource, Parts, Text, VolumePart};
use libero::hooks::use_media;
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/audio", || rsx! { AudioPage {} }),
    ("/audio/swap", || rsx! { SwapPage {} }),
    // Todo 2654: every `<source>` fails, which sets no MediaError.
    ("/audio/sources-failing", || rsx! { SourcesFailingPage {} }),
    // Todo 2679: a healthy player mounted after load, its first `<source>` still to try.
    ("/audio/late", || rsx! { LatePage {} }),
];

#[component]
fn LatePage() -> Element {
    let src = use_hook(|| silent_wav(4));
    let mut shown = use_signal(|| false);
    rsx! {
        Button { id: "mount", onclick: move |_| shown.set(true), "Mount" }
        if shown() {
            div { id: "late",
                Audio {
                    src: src.clone(),
                    sources: vec![MediaSource::new("/none.xyz", "audio/x-none")],
                    label: "Late",
                }
            }
        }
    }
}

#[component]
fn SourcesFailingPage() -> Element {
    rsx! {
        div { id: "failing",
            Audio {
                src: "/missing.mp3",
                sources: vec![MediaSource::new("/none.xyz", "audio/x-none")],
                label: "Failing",
            }
        }
    }
}

/// A silent 8 kHz, 8-bit mono WAV of `seconds`, as a `data:` URL: no file to serve,
/// and every Chromium decodes PCM.
pub(crate) fn silent_wav(seconds: u32) -> String {
    let samples = 8000 * seconds;
    let mut bytes = Vec::with_capacity(44 + samples as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + samples).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // mono
    bytes.extend_from_slice(&8000u32.to_le_bytes());
    bytes.extend_from_slice(&8000u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&8u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&samples.to_le_bytes());
    bytes.resize(44 + samples as usize, 128); // 8-bit silence is the midpoint
    format!("data:audio/wav;base64,{}", base64(&bytes))
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | (*b as u32) << (16 - 8 * i));
        for i in 0..4 {
            out.push(match i <= chunk.len() {
                true => TABLE[(n >> (18 - 6 * i) & 63) as usize] as char,
                false => '=',
            });
        }
    }
    out
}

#[component]
fn AudioPage() -> Element {
    let src = use_hook(|| silent_wav(4));
    let media = use_media();
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "32rem",
            // A first source no browser plays: the WAV `src` after it is the one loaded.
            div { id: "player",
                Audio {
                    src: src.clone(),
                    sources: vec![MediaSource::new("/none.xyz", "audio/x-none")],
                    label: "Silence",
                }
            }
            div { id: "broken", Audio { src: "data:audio/wav;base64,AAAA", label: "Broken" } }
            div { id: "muted", Audio { src: src.clone(), label: "Muted", muted: true } }
            div { id: "rtl", dir: "rtl", Audio { src: src.clone(), label: "Right to left" } }
            // A shrink-wrapping parent: the bubble still takes its full width. Its volume
            // menu is styled through `volume_parts` (todo 1394).
            div { id: "inline", style: "display: inline-flex",
                Audio {
                    src: src.clone(),
                    label: "Inline",
                    volume_parts: Parts::new()
                        .part(VolumePart::Card, sx().padding("13px"))
                        .part(VolumePart::Slider, sx().width("10rem")),
                }
            }
            div { id: "custom",
                audio { src, onmounted: media.mount(), ..media.attributes() }
                Button { onclick: move |_| media.toggle(),
                    if media.paused() { "Play" } else { "Pause" }
                }
                Text { id: "custom-state",
                    "supported={media.is_supported()} duration={media.duration().unwrap_or(0.0)}"
                }
            }
        }
    }
}

/// With `sources`, `src` is the last `<source>`: a swap after mount needs a reload.
#[component]
fn SwapPage() -> Element {
    let long = use_hook(|| silent_wav(4));
    let short = use_hook(|| silent_wav(2));
    let mut swapped = use_signal(|| false);
    rsx! {
        div { id: "swap",
            Audio {
                src: if swapped() { short.clone() } else { long.clone() },
                sources: vec![MediaSource::new("/none.xyz", "audio/x-none")],
                label: "Swap",
            }
        }
        Button { id: "swap-button", onclick: move |_| swapped.set(true), "Swap" }
    }
}
