//! `use_back`: a three-step counter Back steps down, a menu opened over it, and the
//! docs shell's route history.

use dioxus::prelude::*;
use libero::components::{Button, Menu, MenuItem, Text, use_menu};
use libero::hooks::use_back;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-back", || rsx! { UseBackPage {} }),
    ("/use-back/route/0", || rsx! { RoutePage { depth: 0 } }),
    ("/use-back/route/1", || rsx! { RoutePage { depth: 1 } }),
    ("/use-back/route/2", || rsx! { RoutePage { depth: 2 } }),
    ("/use-back/route/3", || rsx! { RoutePage { depth: 3 } }),
    ("/use-back/route/4", || rsx! { RoutePage { depth: 4 } }),
];

#[component]
fn UseBackPage() -> Element {
    let mut step = use_signal(|| 1);
    use_back(step() > 1, Callback::new(move |()| step -= 1));
    let menu = use_menu();
    rsx! {
        Text { id: "step", "{step}" }
        Button { id: "next", onclick: move |_| step += 1, "Next" }
        Menu { state: menu, items: vec![MenuItem::new("Copy").into()],
            Button { id: "more", "More" }
        }
    }
}

/// The docs shell's wiring: Back steps the router back while it can. Depth 0 is the
/// home: the harness's own push onto it is no history the docs have.
#[component]
fn RoutePage(depth: u8) -> Element {
    let history = router();
    use_back(
        depth > 0 && history.can_go_back(),
        use_callback(move |()| history.go_back()),
    );
    rsx! {
        Text { id: "depth", "{depth}" }
        Button {
            id: "deeper",
            onclick: move |_| {
                navigator().push(format!("/use-back/route/{}", depth + 1));
            },
            "Deeper"
        }
    }
}
