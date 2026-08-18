use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Float, Text},
    sx::sx,
};

#[component]
pub fn FloatPage() -> Element {
    rsx! {
        DocPage {
            title: "Float",
            lead: rsx! {
                Text { "Anchors its child to a corner/edge of the nearest `position: relative` ancestor - e.g. a badge on an avatar. The parent must set `position: relative` itself." }
            },
            DocSection {
                title: "Placement",
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
            DocSection {
                title: "Offset",
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
