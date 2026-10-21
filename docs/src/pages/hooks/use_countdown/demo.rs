use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text, VisuallyHidden},
    hooks::{format_duration, use_countdown},
    localization::DurationStyle,
    platform::clock,
};

fn in_90_seconds() -> SystemTime {
    clock().map_or(UNIX_EPOCH, |clock| clock.now()) + Duration::from_secs(90)
}

#[component]
pub fn Offer() -> Element {
    let mut end = use_signal(in_90_seconds);
    let left = use_countdown(end());

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Text {
                "Offer ends in "
                span { aria_hidden: "true", {format_duration(left(), DurationStyle::Clock)} }
                VisuallyHidden { {format_duration(left(), DurationStyle::Spoken)} }
            }
            Text { "In short units: {format_duration(left(), DurationStyle::Short)}" }
            Button { variant: "outlined", onclick: move |_| end.set(in_90_seconds()), "Restart" }
        }
    }
}
