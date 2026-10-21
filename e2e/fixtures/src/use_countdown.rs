//! `use_countdown`: fifty countdowns on the one ticker they share.

use std::time::{Duration, SystemTime};

use dioxus::prelude::*;
use libero::{
    components::VisuallyHidden,
    hooks::{format_duration, use_countdown},
    localization::DurationStyle,
    platform::clock,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/use-countdown", || rsx! { Countdowns {} })];

pub const COUNT: usize = 50;

/// Mounted on a press, so a test holds the ticker's interval before it is set.
#[component]
fn Countdowns() -> Element {
    let mut end = use_signal(|| None::<SystemTime>);
    rsx! {
        button {
            id: "start",
            onclick: move |_| end.set(clock().map(|clock| clock.now() + Duration::from_millis(1200))),
            "Start"
        }
        if let Some(end) = end() {
            for index in 0..COUNT {
                Countdown {
                    key: "{index}",
                    index,
                    end: end + Duration::from_millis(index as u64 % 3 * 400),
                }
            }
        }
    }
}

#[component]
fn Countdown(index: usize, end: SystemTime) -> Element {
    let left = use_countdown(end);
    rsx! {
        p {
            span {
                id: "c{index}",
                aria_hidden: "true",
                "data-zero": left().is_zero(),
                {format_duration(left(), DurationStyle::Clock)}
            }
            VisuallyHidden { {format_duration(left(), DurationStyle::Spoken)} }
        }
    }
}
