//! Responsive props of the data_display family (todo 5654): each responsive element beside a
//! plain twin per breakpoint value, so a test compares the two.

use dioxus::prelude::*;
use libero::components::{Badge, Flex, Icon, Image, Indicator, List, ListItem, Timeline, TimelineEvent};
use libero::sx::ThemeAwareValue;
use libero::theme::{Size, TimelineAlign, Variant, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive-data", || rsx! { ResponsiveDataPage {} })];

const PICTURE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='40' height='40'/%3E";

fn events() -> Vec<TimelineEvent> {
    vec![TimelineEvent::new("Ordered"), TimelineEvent::new("Shipped")]
}

/// `rsp-*` switches at 62rem (`md`); `narrow-*` is its look below, `wide-*` from it.
#[component]
fn ResponsiveDataPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", align: "flex-start",
            Flex { gap: "md", align: "center",
                Icon { id: "rsp-icon", size: responsive(Size::Sm).md(Size::Xl), variant: responsive(Variant::Standard).md(Variant::Filled), svg { view_box: "0 0 24 24" } }
                Icon { id: "narrow-icon", size: "sm", variant: "standard", svg { view_box: "0 0 24 24" } }
                Icon { id: "wide-icon", size: "xl", variant: "filled", svg { view_box: "0 0 24 24" } }
            }
            Flex { gap: "md", align: "center",
                Indicator { id: "rsp-indicator", size: responsive(Size::Xs).md(Size::Xl) }
                Indicator { id: "narrow-indicator", size: "xs" }
                Indicator { id: "wide-indicator", size: "xl" }
            }
            Flex { gap: "md", align: "center",
                Badge { id: "rsp-badge", radius: responsive(ThemeAwareValue::from(Size::Xs)).md("0".into()), "B" }
                Badge { id: "narrow-badge", radius: "xs", "B" }
                Badge { id: "wide-badge", radius: "0", "B" }
            }
            Flex { gap: "md", align: "center",
                Image { id: "rsp-image", src: PICTURE, alt: "", radius: responsive(ThemeAwareValue::from("0")).md("12px".into()) }
                Image { id: "narrow-image", src: PICTURE, alt: "", radius: "0" }
                Image { id: "wide-image", src: PICTURE, alt: "", radius: "12px" }
            }
            Flex { gap: "md",
                List { id: "rsp-list", size: responsive(Size::Xs).md(Size::Xl), ListItem { "a" } ListItem { "b" } }
                List { id: "narrow-list", size: "xs", ListItem { "a" } ListItem { "b" } }
                List { id: "wide-list", size: "xl", ListItem { "a" } ListItem { "b" } }
            }
            Timeline {
                id: "rsp-timeline",
                align: responsive(TimelineAlign::Start).md(TimelineAlign::End),
                gap: responsive(ThemeAwareValue::from(Size::Xs)).md("40px".into()),
                bullet_size: responsive(Size::Xs).md(Size::Xl),
                items: events(),
            }
            Timeline { id: "narrow-timeline", align: "start", gap: "xs", bullet_size: "xs", items: events() }
            Timeline { id: "wide-timeline", align: "end", gap: "40px", bullet_size: "xl", items: events() }
        }
    }
}
