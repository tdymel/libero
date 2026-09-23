//! A filled `Icon` in a theme colour, for its cached colour variables, and a
//! named one beside the unnamed one (todo 612), a `standard` one (786), and
//! one drawn from `SvgData` (1094).

use dioxus::prelude::*;
use libero::components::{Button, Icon, SvgData};

use crate::Routes;

pub const ROUTES: Routes = &[("/icon", || rsx! { IconPage {} })];

#[component]
fn IconPage() -> Element {
    let mut error = use_signal(|| true);

    rsx! {
        Icon { id: "icon", variant: "filled", color: if error() { "error" } else { "success" },
            svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "8" } }
        }
        Button { id: "swap", onclick: move |_| error.toggle(), "Swap colour" }
        Icon { id: "named", aria_label: "Verified",
            svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "8" } }
        }
        Icon { id: "bare", variant: "standard",
            svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "8" } }
        }
        Icon { id: "pictogram", variant: "standard", color: "error", svg: RING }
    }
}

const RING: SvgData = SvgData::new(
    r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="8"/></svg>"#,
);
