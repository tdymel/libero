use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Paper, Text, Transition, TransitionKind},
    sx::sx,
};

/// Its own component: a hook in `render` would land in `Demo`'s scope.
#[component]
pub fn TransitionDemo(
    kind: TransitionKind,
    duration: Option<u32>,
    filter: Option<String>,
) -> Element {
    // demo-code: state start
    let mut show = use_signal(|| false);
    // demo-code: state end

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Button { aria_expanded: show(), onclick: move |_| show.toggle(), "Toggle" }
            Transition { kind, duration, open: show(), from: filter.map(|filter| sx().filter(filter)),
                Paper { Text { "Hello" } }
            }
        }
    }
}
