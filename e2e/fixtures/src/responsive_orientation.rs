//! Responsive `orientation` (todo 5656): a toolbar and a data list, a column under 62rem and a
//! row from it.

use dioxus::prelude::*;
use libero::components::{ActionIcon, DataList, DataListItem, Orientation, Toolbar};
use libero::theme::responsive;

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive-orientation", || {
    rsx! { ResponsiveOrientationPage {} }
})];

#[component]
fn ResponsiveOrientationPage() -> Element {
    let stacked = responsive(Orientation::Vertical).md(Orientation::Horizontal);
    rsx! {
        Toolbar { id: "rso-toolbar", "aria-label": "Formatting", orientation: stacked,
            ActionIcon { id: "rso-bold", aria_label: "Bold", "B" }
            ActionIcon { id: "rso-italic", aria_label: "Italic", "I" }
            ActionIcon { id: "rso-under", aria_label: "Underline", "U" }
        }
        DataList { id: "rso-list", orientation: stacked,
            DataListItem { label: rsx! { "Name" }, "Ada Lovelace" }
        }
    }
}
