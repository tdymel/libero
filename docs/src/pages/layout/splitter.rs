use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Splitter, Text, Title},
    sx::sx,
};

#[component]
pub fn SplitterPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Splitter" }
                Text { "Splits two panes with a draggable/keyboard-resizable divider. Only two panes - nest another `Splitter` inside a pane for more." }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Vertical (default)" }
                Text { "Side-by-side panes, dragged left/right." }
                Box {
                    sx: sx().height("160px").border("1px solid var(--lsx-grey-3)"),
                    Splitter {
                        initial_size: 50.0,
                        Box { sx: sx().height("100%").padding("md").background("primary.1"), "A" }
                        Box { sx: sx().height("100%").padding("md").background("secondary.1"), "B" }
                    }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Horizontal" }
                Text { "Stacked panes, dragged up/down." }
                Box {
                    sx: sx().height("220px").border("1px solid var(--lsx-grey-3)"),
                    Splitter {
                        orientation: "horizontal",
                        initial_size: 30.0,
                        min_size: 15.0,
                        Box { sx: sx().height("100%").padding("md").background("primary.1"), "Top" }
                        Box { sx: sx().height("100%").padding("md").background("secondary.1"), "Bottom" }
                    }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Composed" }
                Text { "Nesting `Splitter`s composes more than 2 panes - here a horizontal split on the right of a vertical one." }
                Box {
                    sx: sx().height("260px").border("1px solid var(--lsx-grey-3)"),
                    Splitter {
                        initial_size: 35.0,
                        Box { sx: sx().height("100%").padding("md").background("primary.1"), "Sidebar" }
                        Splitter {
                            orientation: "horizontal",
                            initial_size: 65.0,
                            Box { sx: sx().height("100%").padding("md").background("secondary.1"), "Main" }
                            Box { sx: sx().height("100%").padding("md").background("info.1"), "Panel" }
                        }
                    }
                }
            }
        }
    }
}
