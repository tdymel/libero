use crate::{Route, components::DocPage};
use dioxus::prelude::*;
use libero::components::{Anchor, Code, Kbd, Text};

/// Any URL no route matches: a mistyped address, an old bookmark, a dead link.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = format!("/{}", segments.join("/"));

    rsx! {
        DocPage {
            title: "Page not found",
            lead: rsx! {
                Text {
                    "There is no page at "
                    Code { source: path }
                    ". Start again from the "
                    Anchor { to: Route::Home {}, "home page" }
                    ", pick one in the navigation, or search the docs with the Search button or "
                    Kbd { "Ctrl K" }
                    " ("
                    Kbd { "Cmd K" }
                    " on a Mac)."
                }
            },
        }
    }
}
