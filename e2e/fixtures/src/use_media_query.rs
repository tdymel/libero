//! `use_media_query` and `use_is_mobile`, as their docs page uses them.

use dioxus::prelude::*;
use libero::hooks::{use_is_mobile, use_media_query};

use crate::Routes;

pub const ROUTES: Routes = &[("/use-media-query", || rsx! { Queries {} })];

#[component]
fn Queries() -> Element {
    let wide = use_media_query("(min-width: 1024px)");
    let mobile = use_is_mobile();

    rsx! {
        p { id: "wide", if wide() { "yes" } else { "no" } }
        p { id: "mobile", if mobile() { "yes" } else { "no" } }
    }
}
