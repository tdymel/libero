use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::use_back,
};

#[component]
pub fn Steps() -> Element {
    let mut step = use_signal(|| 1);
    use_back(step() > 1, Callback::new(move |()| step -= 1));
    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            // A live line, so a step taken by Back is announced too.
            Text { role: "status", "Step {step} of 3" }
            Flex { gap: "sm",
                Button {
                    variant: "outlined",
                    disabled: step() == 1,
                    focusable_when_disabled: true,
                    onclick: move |_| step -= 1,
                    "Previous"
                }
                Button {
                    disabled: step() == 3,
                    focusable_when_disabled: true,
                    onclick: move |_| step += 1,
                    "Next"
                }
            }
        }
    }
}
