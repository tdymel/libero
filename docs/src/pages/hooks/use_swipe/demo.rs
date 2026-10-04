use dioxus::prelude::*;
use libero::{
    components::{Box, Button, ButtonGroup, Flex},
    hooks::{SwipeDirection, SwipeEvent, SwipeOptions, use_swipe},
    sx::sx,
};

#[component]
pub fn SwipePad() -> Element {
    let mut last = use_signal(|| None::<SwipeDirection>);
    let swipe = use_swipe(
        Callback::new(move |event: SwipeEvent| last.set(Some(event.direction))),
        SwipeOptions::default(),
    );
    let said = match last() {
        Some(direction) => format!("Swiped {direction:?}"),
        None => "No swipe yet".to_string(),
    };

    rsx! {
        Flex { direction: "column", align: "stretch", gap: "sm",
            Box {
                // All four directions reach the hook; the page does not scroll from here.
                sx: sx()
                    .touch_action("none")
                    .height("160px")
                    .border("1px dashed")
                    .border_radius("md")
                    .display("grid")
                    .place_items("center"),
                onpointerdown: move |event| swipe.onpointerdown.call(event),
                onpointermove: move |event| swipe.onpointermove.call(event),
                onpointerup: move |event| swipe.onpointerup.call(event),
                onpointercancel: move |event| swipe.onpointercancel.call(event),
                "Swipe here with a finger or a pen"
            }
            // The same choices without a gesture (WCAG 2.5.1).
            ButtonGroup { "aria-label": "Swipe by button",
                Button { onclick: move |_| last.set(Some(SwipeDirection::Left)), "Left" }
                Button { onclick: move |_| last.set(Some(SwipeDirection::Right)), "Right" }
            }
            div { role: "status", "{said}" }
        }
    }
}
