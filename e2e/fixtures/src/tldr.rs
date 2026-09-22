//! `Tldr`: a labelled and an icon-only trigger, German words, and a custom provider list.

use dioxus::prelude::*;
use libero::{
    components::{SummaryProvider, Tldr},
    hooks::use_localization_handle,
    localization::Localization,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/tldr", || rsx! { TldrPage {} }),
    ("/tldr/icon", || rsx! { TldrIconPage {} }),
    ("/tldr/german", || rsx! { TldrGermanPage {} }),
    ("/tldr/custom", || rsx! { TldrCustomPage {} }),
];

const PAGE: &str = "https://libero-ui.dev/md/menu.md";

#[component]
fn TldrPage() -> Element {
    rsx! {
        h1 { "A page" }
        Tldr { url: PAGE }
    }
}

#[component]
fn TldrIconPage() -> Element {
    rsx! {
        h1 { "A page" }
        Tldr { url: PAGE, icon_only: true }
    }
}

#[component]
fn TldrGermanPage() -> Element {
    let localization = use_localization_handle();
    use_effect(move || localization.set(&Localization::GERMAN));
    rsx! {
        h1 { "Eine Seite" }
        Tldr { url: PAGE, icon_only: true }
    }
}

/// Google dropped, one provider without a mark added, the docs' own prompt.
#[component]
fn TldrCustomPage() -> Element {
    let mut providers = SummaryProvider::defaults();
    providers.retain(|provider| provider.name != "Google AI");
    providers.push(SummaryProvider::new(
        "Example",
        "https://example.test/chat?q=",
    ));
    rsx! {
        h1 { "A page" }
        Tldr {
            url: PAGE,
            providers,
            // Doubled braces: rsx would format a bare `{url}`.
            prompt: "Read {{url}}",
            label: "Summary",
        }
    }
}
