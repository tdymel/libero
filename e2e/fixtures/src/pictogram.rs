//! `Pictogram` on its own: decorative, named, named by reference and given an empty name.

use dioxus::prelude::*;
use libero::components::{Pictogram, Text};
use pictogram_icons_lucide as lucide;

use crate::Routes;

pub const ROUTES: Routes = &[("/pictogram", || rsx! { PictogramPage {} })];

#[component]
fn PictogramPage() -> Element {
    rsx! {
        Text { id: "page-text",
            Pictogram { id: "decorative", icon: lucide::house::outlined, width: "24px", height: "24px" }
            Pictogram { id: "named", icon: lucide::house::outlined, aria_label: "Home", width: "24px", height: "24px" }
            Pictogram { id: "labelled", icon: lucide::star::outlined, "aria-labelledby": "caption", width: "24px", height: "24px" }
            Pictogram { id: "empty", icon: lucide::bell::outlined, aria_label: "", width: "24px", height: "24px" }
            span { id: "caption", "Starred" }
        }
    }
}
