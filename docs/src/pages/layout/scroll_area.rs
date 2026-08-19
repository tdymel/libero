use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, ScrollArea, ScrollPositionEvent, Text},
    sx::sx,
};

#[component]
pub fn ScrollAreaPage() -> Element {
    let mut position = use_signal(|| (0.0, 0.0));
    let mut jump_target = use_signal(|| None::<f64>);

    rsx! {
        DocPage {
            title: "ScrollArea",
            lead: rsx! {
                Text { "Scrolls its content, filling the parent by default. Reports live scroll position/edges via events, and can be scrolled to a percent imperatively." }
            },
            DocSection {
                title: "Default (vertical)",
                Box {
                    sx: sx().height("160px").border("1px solid var(--lsx-grey-3)"),
                    ScrollArea {
                        Flex {
                            direction: "column",
                            gap: "md",
                            sx: sx().padding("md"),
                            for i in 0..20 {
                                Text { key: "{i}", "Item {i}" }
                            }
                        }
                    }
                }
            }
            DocSection {
                title: "Both axes, hover-visible scrollbars",
                Box {
                    sx: sx().height("160px").width("240px").border("1px solid var(--lsx-grey-3)"),
                    ScrollArea {
                        scrollbars: "both",
                        scrollbar_visibility: "hover",
                        Box {
                            sx: sx().width("500px").padding("md"),
                            for i in 0..20 {
                                Text { key: "{i}", "Wide item {i}" }
                            }
                        }
                    }
                }
            }
            DocSection {
                title: "Live position + reached events",
                Text { "x: {position().0:.0}%, y: {position().1:.0}%" }
                Box {
                    sx: sx().height("160px").width("320px").border("1px solid var(--lsx-grey-3)"),
                    ScrollArea {
                        scrollbars: "both",
                        scroll_position_y: jump_target(),
                        on_scroll: move |event| {
                            let (ScrollPositionEvent::Start(x, y)
                            | ScrollPositionEvent::Change(x, y)
                            | ScrollPositionEvent::End(x, y)) = event;
                            position.set((x, y));
                        },
                        on_top_reached: move |_| jump_target.set(None),
                        Flex {
                            direction: "column",
                            gap: "md",
                            sx: sx().padding("md").width("640px"),
                            for i in 0..30 {
                                Text { key: "{i}", "Row {i} - some extra trailing text to force horizontal overflow too" }
                            }
                        }
                    }
                }
                Flex {
                    gap: "sm",
                    Box {
                        sx: sx()
                            .padding("sm")
                            .border("1px solid var(--lsx-grey-3)")
                            .cursor("pointer"),
                        onclick: move |_| jump_target.set(Some(0.0)),
                        "Scroll to top"
                    }
                    Box {
                        sx: sx()
                            .padding("sm")
                            .border("1px solid var(--lsx-grey-3)")
                            .cursor("pointer"),
                        onclick: move |_| jump_target.set(Some(100.0)),
                        "Scroll to bottom"
                    }
                }
            }
        }
    }
}
