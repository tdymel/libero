use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Splitter, Text},
    sx::sx,
};

#[component]
pub fn SplitterPage() -> Element {
    rsx! {
        DocPage {
            title: "Splitter",
            lead: rsx! {
                Text { "Splits two panes with a draggable/keyboard-resizable divider. Panes go in `panel_a` and `panel_b`; nest another `Splitter` in a pane for more than two." }
            },
            DocSection {
                title: "Vertical (default)",
                Text { "Side-by-side panes, dragged left/right." }
                Box {
                    sx: sx().height("160px").border("1px solid var(--lsx-grey-3)"),
                    Splitter {
                        initial_size: 50.0,
                        panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "A" } },
                        panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "B" } },
                    }
                }
            }
            DocSection {
                title: "Horizontal",
                Text { "Stacked panes, dragged up/down." }
                Box {
                    sx: sx().height("220px").border("1px solid var(--lsx-grey-3)"),
                    Splitter {
                        orientation: "horizontal",
                        initial_size: 30.0,
                        min_size: 15.0,
                        panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "Top" } },
                        panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "Bottom" } },
                    }
                }
            }
            DocSection {
                title: "Composed",
                Text { "Nesting `Splitter`s composes more than 2 panes - here a horizontal split on the right of a vertical one." }
                Box {
                    sx: sx().height("260px").border("1px solid var(--lsx-grey-3)"),
                    Splitter {
                        initial_size: 35.0,
                        panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "Sidebar" } },
                        panel_b: rsx! {
                            Splitter {
                                orientation: "horizontal",
                                initial_size: 65.0,
                                panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "Main" } },
                                panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("info.1"), "Panel" } },
                            }
                        },
                    }
                }
            }
        }
    }
}
