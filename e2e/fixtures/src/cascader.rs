//! `Cascader`.

use dioxus::prelude::*;
use libero::components::{Cascader, CascaderOption, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/cascader", || rsx! { CascaderPage {} })];

/// Three levels, with the middle root disabled so the keys and a click both
/// have one to pass over. `#picked` shows the committed value.
#[component]
fn CascaderPage() -> Element {
    let mut place = use_signal(|| None::<String>);
    let data = vec![
        CascaderOption::new("europe", "Europe").children(vec![
            CascaderOption::new("france", "France").children(vec![
                CascaderOption::new("paris", "Paris"),
                CascaderOption::new("lyon", "Lyon"),
            ]),
            CascaderOption::new("germany", "Germany")
                .children(vec![CascaderOption::new("berlin", "Berlin")]),
        ]),
        CascaderOption::new("asia", "Asia")
            .disabled(true)
            .children(vec![
                CascaderOption::new("japan", "Japan")
                    .children(vec![CascaderOption::new("tokyo", "Tokyo")]),
            ]),
        CascaderOption::new("oceania", "Oceania").children(vec![
            CascaderOption::new("australia", "Australia")
                .children(vec![CascaderOption::new("sydney", "Sydney")]),
        ]),
    ];

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Cascader {
                label: "Place",
                data,
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
            Text { id: "picked", {place().unwrap_or_default()} }
        }
    }
}
