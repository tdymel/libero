//! Responsive `size`, `radius` and card variant on the form family (todo 5652).

use dioxus::prelude::*;
use libero::components::{Button, Checkbox, Flex, Options, SegmentedControl, TextField, Variant};
use libero::sx::ThemeAwareValue;
use libero::theme::{ChoiceVariant, Size, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive-form", || rsx! { ResponsiveFormPage {} })];

#[derive(Clone, PartialEq, Options)]
enum View {
    List,
    Grid,
}

/// A field small under 62rem and large from it, beside plain `sm` and `lg` ones to measure
/// against; a checkbox that turns into a card; a segmented control whose segments, look and gap
/// switch at md; a late field registers its copies after mount.
#[component]
fn ResponsiveFormPage() -> Element {
    let mut late = use_signal(|| false);
    rsx! {
        Flex { direction: "column", gap: "md",
            div { id: "rspf-sm", TextField { label: "Small", size: "sm" } }
            div { id: "rspf-lg", TextField { label: "Large", size: "lg" } }
            div { id: "rspf-text",
                TextField { label: "Responsive", size: responsive(Size::Sm).md(Size::Lg) }
            }
            div { id: "rspf-card",
                Checkbox {
                    label: "Card from md",
                    variant: responsive(ChoiceVariant::Plain).md(ChoiceVariant::Card),
                }
            }
            div { id: "rspf-seg",
                SegmentedControl {
                    label: "View",
                    value: View::List,
                    onchange: move |_| {},
                    size: responsive(Size::Sm).md(Size::Lg),
                    variant: responsive(Variant::Filled).md(Variant::Outlined),
                    gap: responsive(ThemeAwareValue::from("2px")).md("10px".into()),
                }
            }
            Button { id: "rspf-add", onclick: move |_| late.set(true), "Add" }
            if late() {
                div { id: "rspf-late",
                    TextField { label: "Late", size: responsive(Size::Sm).md(Size::Lg) }
                }
            }
        }
    }
}
