//! Picker fields followed by radio groups, for the page's next Tab stop: a group whose checked
//! radio is hidden (todo 4601) and one tied to a form by its `form` attribute (4600).

use dioxus::prelude::*;
use libero::{
    chrono::NaiveDate,
    components::{ChronoField, Flex},
};

use crate::Routes;

pub const ROUTES: Routes = &[("/tab-stops", || rsx! { TabStopsPage {} })];

#[component]
fn TabStopsPage() -> Element {
    let day = |day| NaiveDate::from_ymd_opt(2026, 9, day);
    let mut hidden = use_signal(move || day(14));
    let mut owned = use_signal(move || day(14));

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            div { id: "hidden",
                ChronoField::<NaiveDate> {
                    label: "Hidden",
                    today: day(9),
                    value: hidden(),
                    onchange: move |next| hidden.set(next),
                }
                input {
                    r#type: "radio",
                    name: "size",
                    value: "s",
                    checked: true,
                    style: "display: none",
                    aria_label: "Small",
                }
                input {
                    id: "medium",
                    r#type: "radio",
                    name: "size",
                    value: "m",
                    aria_label: "Medium",
                }
                input {
                    r#type: "radio",
                    name: "size",
                    value: "l",
                    aria_label: "Large",
                }
            }
            div { id: "owned",
                ChronoField::<NaiveDate> {
                    label: "Owned",
                    today: day(9),
                    value: owned(),
                    onchange: move |next| owned.set(next),
                }
                input {
                    id: "loose",
                    r#type: "radio",
                    name: "tone",
                    value: "warm",
                    form: "toned",
                    aria_label: "Warm",
                }
                form { id: "toned",
                    input {
                        id: "held",
                        r#type: "radio",
                        name: "tone",
                        value: "cool",
                        checked: true,
                        aria_label: "Cool",
                    }
                }
            }
        }
    }
}
