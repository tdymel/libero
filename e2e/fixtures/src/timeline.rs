//! `Timeline` in every alignment, with and without custom bullets.

use dioxus::prelude::*;
use libero::components::{Flex, OptionLabel, Text, Timeline, TimelineEvent};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/timeline", || rsx! { TimelinePage {} }),
    ("/timeline/rich", || rsx! { RichTitlePage {} }),
    ("/timeline/opposite", || rsx! { OppositePage {} }),
];

/// Todo 4148: a date across the rail from each body, in every alignment.
#[component]
fn OppositePage() -> Element {
    let dated = || -> Vec<TimelineEvent> {
        ["Ordered", "Shipped"]
            .into_iter()
            .map(|title| {
                TimelineEvent::new(title)
                    .content(rsx! { Text { size: "sm", "Left the depot." } })
                    .opposite(rsx! { Text { size: "sm", "12 May" } })
            })
            .collect()
    };
    rsx! {
        Flex { direction: "column", gap: "xl", max_width: "480px",
            Timeline { id: "start", items: dated() }
            Timeline { id: "end", align: "end", items: dated() }
            Timeline { id: "alternate", align: "alternate", items: dated() }
        }
    }
}

/// Todo 2429: a rich title is drawn, and a screen reader reads its name.
#[component]
fn RichTitlePage() -> Element {
    let rich = OptionLabel::rich("Shipped by air", rsx! { b { "Shipped" } " ✈" });
    rsx! {
        Timeline { id: "rich", items: vec![TimelineEvent::new(rich), TimelineEvent::new("Delivered")] }
    }
}

fn events() -> Vec<TimelineEvent> {
    ["Ordered", "Packed", "Shipped", "Delivered"]
        .into_iter()
        .map(|title| TimelineEvent::new(title).content(rsx! { Text { size: "sm", "On 12 May" } }))
        .collect()
}

#[component]
fn TimelinePage() -> Element {
    let with_bullets: Vec<_> = events()
        .into_iter()
        .map(|event| event.bullet(rsx! { "*" }))
        .collect();
    let mut coloured = events();
    coloured[1] = coloured[1].clone().color("error");

    rsx! {
        Flex { direction: "column", gap: "xl", max_width: "480px",
            Timeline { id: "left", items: events(), active: 1 }
            Timeline { id: "right", align: "end", items: coloured, active: 2 }
            Timeline { id: "alternate", align: "alternate", items: with_bullets, active: 1 }
            Timeline { id: "none", items: events() }
        }
    }
}
