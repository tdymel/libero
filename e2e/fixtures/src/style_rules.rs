//! Sheets registered after mount, which the web inserts as rules into the outlet's
//! layer blocks instead of a `<style>` each (todo 2186).

use dioxus::prelude::*;
use libero::components::{Box, Button};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/style-rules", || rsx! { StyleRulesPage {} })];

/// "Show" mounts a plain class-scoped box and one with a media query; "Hide" unmounts both.
#[component]
fn StyleRulesPage() -> Element {
    let mut shown = use_signal(|| false);

    rsx! {
        Button { id: "toggle", onclick: move |_| shown.toggle(),
            if shown() { "Hide" } else { "Show" }
        }
        if shown() {
            Box {
                id: "plain",
                sx: sx().background("rgb(1, 2, 3)").color("rgb(250, 250, 250)").hover(sx().color("rgb(4, 5, 6)")),
                "Plain"
            }
            Box {
                id: "media",
                sx: sx().media("(min-width: 1px)", sx().color("rgb(7, 8, 9)")),
                "Media"
            }
        }
    }
}
