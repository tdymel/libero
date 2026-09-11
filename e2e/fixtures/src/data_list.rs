//! A `DataList` whose term carries a caller class and id.

use dioxus::prelude::*;
use libero::components::{DataList, DataListItem};

use crate::Routes;

pub const ROUTES: Routes = &[("/data-list", || rsx! { DataListPage {} })];

#[component]
fn DataListPage() -> Element {
    rsx! {
        DataList { id: "list", orientation: "horizontal",
            DataListItem { id: "term", label: rsx! { "Status" }, "Active" }
        }
    }
}
