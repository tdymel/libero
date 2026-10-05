//! Sheets registered after mount, which the web inserts as rules into the outlet's
//! layer blocks instead of a `<style>` each (todo 2186).

use dioxus::prelude::*;
use libero::components::{Box, Button};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/style-rules", || rsx! { StyleRulesPage {} })];

/// The media and container boxes' CSS as flat rules in a `<style>`, for the `flat-` twins.
const FLAT: &str = "#flat-media{color:rgb(10, 11, 12)}\
    @media (min-width: 1px){#flat-media{color:rgb(7, 8, 9)}}\
    #flat-media-out{color:rgb(10, 11, 12)}\
    @media (max-width: 1px){#flat-media-out{color:rgb(13, 14, 15)}}\
    #flat-probe{container:probe / inline-size;width:400px}\
    #flat-container{color:rgb(10, 11, 12)}\
    @container probe (min-width: 300px){#flat-container{color:rgb(16, 17, 18)}}\
    #flat-container-out{color:rgb(10, 11, 12)}\
    @container probe (min-width: 500px){#flat-container-out{color:rgb(13, 14, 15)}}";

/// "Show" mounts a plain class-scoped box and boxes in and out of a media and a container
/// query (todo 2231); "Hide" unmounts them. The `flat-` twins hold the same CSS unnested.
#[component]
fn StyleRulesPage() -> Element {
    let mut shown = use_signal(|| false);
    let base = || sx().color("rgb(10, 11, 12)");

    rsx! {
        style { dangerous_inner_html: FLAT }
        Button { id: "toggle", onclick: move |_| shown.toggle(),
            if shown() { "Hide" } else { "Show" }
        }
        div { id: "flat-media", "Flat media" }
        div { id: "flat-media-out", "Flat media out" }
        div { id: "flat-probe",
            div { id: "flat-container", "Flat container" }
            div { id: "flat-container-out", "Flat container out" }
        }
        if shown() {
            Box {
                id: "plain",
                sx: sx().background("rgb(1, 2, 3)").color("rgb(250, 250, 250)").hover(sx().color("rgb(4, 5, 6)")),
                "Plain"
            }
            Box {
                id: "media",
                sx: base().media("(min-width: 1px)", sx().color("rgb(7, 8, 9)")),
                "Media"
            }
            Box {
                id: "media-out",
                sx: base().media("(max-width: 1px)", sx().color("rgb(13, 14, 15)")),
                "Media out"
            }
            Box { sx: sx().container("probe").width("400px"),
                Box {
                    id: "container",
                    sx: base().container_query("probe", "(min-width: 300px)", sx().color("rgb(16, 17, 18)")),
                    "Container"
                }
                Box {
                    id: "container-out",
                    sx: base().container_query("probe", "(min-width: 500px)", sx().color("rgb(13, 14, 15)")),
                    "Container out"
                }
            }
        }
    }
}
