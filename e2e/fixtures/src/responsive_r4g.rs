//! Responsive leftovers (bundle r4g): Timeline align beside an opposite (5673), AvatarGroup
//! size (5672), Table size (5671), ButtonGroup size and variant (5667), line_clamp 0 range
//! with a truncate (5659), a masonry GridZone's gap per breakpoint (5660).

use dioxus::prelude::*;
use libero::components::{
    AvatarGroup, AvatarSpec, Button, ButtonGroup, GridItem, GridZone, Table, Text, Timeline,
    TimelineEvent, column,
};
use libero::sx::ThemeAwareValue;
use libero::theme::{Size, TimelineAlign, Variant, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive-r4g", || rsx! { ResponsiveR4gPage {} })];

#[component]
fn ResponsiveR4gPage() -> Element {
    let events: Vec<TimelineEvent> = ["Ordered", "Shipped", "Delivered"]
        .into_iter()
        .map(|title| TimelineEvent::new(title).opposite(rsx! { "09:00" }))
        .collect();
    rsx! {
        Timeline {
            id: "r4g-timeline",
            items: events,
            align: responsive(TimelineAlign::Start).md(TimelineAlign::Alternate),
        }
        AvatarGroup {
            id: "r4g-avatars",
            people: ["Ada", "Alan", "Grace", "Edsger"].into_iter().map(AvatarSpec::from).collect(),
            max: 3,
            size: responsive(Size::Xs).md(Size::Lg),
        }
        Table {
            id: "r4g-table",
            caption: "People",
            data: vec!["Ada", "Alan"],
            columns: vec![column("Name").value(|name: &&str| name.to_string())],
            size: responsive(Size::Xs).md(Size::Md),
        }
        ButtonGroup {
            id: "r4g-group",
            "aria-label": "Style",
            size: responsive(Size::Xs).md(Size::Lg),
            variant: responsive(Variant::Filled).md(Variant::Outlined),
            Button { "Bold" }
            Button { id: "r4g-italic", "Italic" }
        }
        Text {
            id: "r4g-text",
            truncate: "end",
            line_clamp: responsive(0).md(3),
            sx: libero::sx::sx().max_width("120px"),
            "A sentence long enough to wrap over several lines in a narrow column of text."
        }
        // 500px wide: the 4% cap, 20px, stays over both gaps.
        div { style: "width: 500px",
            GridZone { masonry: true, gap: responsive(ThemeAwareValue::from("4px")).md("16px".into()),
                GridItem { id: "r4g-stone-a", div { style: "height: 30px", "A" } }
                GridItem { id: "r4g-stone-b", div { style: "height: 30px", "B" } }
            }
        }
    }
}
