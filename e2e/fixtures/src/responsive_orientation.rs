//! Responsive `orientation` (todo 5656): a toolbar, a data list, radio groups, a segmented
//! control, a divider, a marquee, a slider and a carousel, a column under 62rem and a row from it.

use dioxus::prelude::*;
use libero::components::{
    ActionIcon, Carousel, DataList, DataListItem, Divider, Marquee, Orientation, RadioGroup,
    SegmentedControl, Slider, Toolbar,
};
use libero::theme::{ChoiceVariant, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive-orientation", || {
    rsx! { ResponsiveOrientationPage {} }
})];

#[component]
fn ResponsiveOrientationPage() -> Element {
    let stacked = responsive(Orientation::Vertical).md(Orientation::Horizontal);
    rsx! {
        Toolbar { id: "rso-toolbar", "aria-label": "Formatting", orientation: stacked,
            ActionIcon { id: "rso-bold", aria_label: "Bold", "B" }
            ActionIcon { id: "rso-italic", aria_label: "Italic", "I" }
            ActionIcon { id: "rso-under", aria_label: "Underline", "U" }
        }
        DataList { id: "rso-list", orientation: stacked,
            DataListItem { label: rsx! { "Name" }, "Ada Lovelace" }
        }
        div { id: "rso-radio",
            RadioGroup::<String> {
                label: "Plan",
                options: vec!["Free".to_string(), "Pro".to_string()],
                orientation: stacked,
                onchange: |_| {},
            }
        }
        // Cards only from md, where the row starts: both axes per breakpoint (todo 5675).
        div { id: "rso-cards",
            RadioGroup::<String> {
                label: "Tier",
                options: vec!["Free".to_string(), "Pro".to_string()],
                orientation: stacked,
                variant: responsive(ChoiceVariant::Plain).md(ChoiceVariant::Card),
                onchange: |_| {},
            }
        }
        div { id: "rso-segmented",
            SegmentedControl::<String> {
                "aria-label": "Align",
                options: vec!["Left".to_string(), "Right".to_string()],
                value: "Left".to_string(),
                orientation: stacked,
                onchange: |_| {},
            }
        }
        Divider { id: "rso-divider", orientation: stacked }
        // Wider and taller than the view on both axes, so the debug check never warns.
        Marquee { id: "rso-marquee", orientation: stacked, sx: libero::sx::sx().height("160px"),
            span { style: "display:inline-block;width:1400px;height:1400px", "Rust" }
        }
        div { id: "rso-slider",
            Slider::<f64> { label: "Volume", value: 40.0, oninput: |_| {}, orientation: stacked }
        }
        div { id: "rso-carousel",
            Carousel {
                aria_label: "Photos",
                orientation: stacked,
                height: "200px",
                slides: vec![rsx! { "One" }, rsx! { "Two" }],
            }
        }
    }
}
