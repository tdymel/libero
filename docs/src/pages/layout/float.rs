use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Float, Text, Title},
    sx::sx,
};

#[component]
pub fn FloatPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "Float" }
                Text { "Anchors its child to a corner/edge of the nearest `position: relative` ancestor - e.g. a badge on an avatar. The parent must set `position: relative` itself." }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Placement" }
                Box {
                    sx: sx().position("relative").width("120px").height("120px").background("primary.1"),
                    Float {
                        placement: "top-start",
                        Box { sx: sx().padding("4px 8px").background("primary"), "TS" }
                    }
                    Float {
                        placement: "top-end",
                        Box { sx: sx().padding("4px 8px").background("primary"), "TE" }
                    }
                    Float {
                        placement: "bottom-start",
                        Box { sx: sx().padding("4px 8px").background("primary"), "BS" }
                    }
                    Float {
                        placement: "bottom-end",
                        Box { sx: sx().padding("4px 8px").background("primary"), "BE" }
                    }
                    Float {
                        Box { sx: sx().padding("4px 8px").background("primary"), "Center" }
                    }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Offset" }
                Text { "`offset_x`/`offset_y` nudge the floated element away from its anchor." }
                Box {
                    sx: sx().position("relative").width("120px").height("120px").background("primary.1"),
                    Float {
                        placement: "top-end",
                        offset_x: "-8px",
                        offset_y: "8px",
                        Box { sx: sx().padding("4px 8px").background("primary"), "8px in" }
                    }
                }
            }
        }
    }
}
