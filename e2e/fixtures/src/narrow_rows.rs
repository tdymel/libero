//! Components as items of a flex row narrower than their content (todos 1493, 1499, 1507).

use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, Fieldset, OptionLabel, Options, RadioGroup, SegmentedControl, Switch,
    TextField, Video,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/narrow-rows", || rsx! { NarrowRowsPage {} })];

const LONG: &str = "Versandkostenberechnungsgrundlagenverordnungsentwurfsbearbeitungsstelle";
const ROW: &str = "display: flex; width: 200px; margin-bottom: 16px;";

#[component]
fn NarrowRowsPage() -> Element {
    rsx! {
        div { "data-case": "button", style: ROW, Button { "{LONG}" } }
        div { "data-case": "fieldset", style: ROW,
            Fieldset::<()> { label: "Address",
                TextField { label: "Street", value: "Hauptstrasse 1" }
            }
        }
        div { "data-case": "radio-group", style: ROW,
            RadioGroup {
                label: "Billing",
                variant: "card",
                orientation: "horizontal",
                value: Some(Period::Weekly),
                onchange: |_: Period| {},
            }
        }
        div { "data-case": "checkbox-card", style: ROW,
            Checkbox {
                variant: "card",
                label: "Priority support",
                description: "Answers within four hours, around the clock.",
                checked: false,
                onchange: |_| {},
            }
        }
        div { "data-case": "switch-card", style: ROW,
            Switch {
                variant: "card",
                label: "Bluetooth",
                description: "Finds nearby devices and pairs with them.",
            }
        }
        div { "data-case": "segmented", style: ROW,
            SegmentedControl {
                aria_label: "Billing",
                value: Period::Weekly,
                onchange: |_| {},
                option_label: |period: Period| OptionLabel::from(period.name()),
            }
        }
        // Todo 1507: the Video page's advice for a shrink-wrapping column holds.
        div { "data-case": "video", style: ROW,
            div { style: "display: flex; flex-direction: column; min-width: 0;",
                Video { src: "/missing.webm", label: "Unsized" }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Period {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Period {
    fn name(self) -> &'static str {
        match self {
            Period::Daily => "Daily",
            Period::Weekly => "Weekly",
            Period::Monthly => "Monthly",
            Period::Yearly => "Yearly",
        }
    }
}
