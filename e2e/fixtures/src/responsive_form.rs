//! Responsive `size`, `radius` and card variant on the form family (todo 5652).

use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, ColorCode, ColorField, ColorPicker, DatePicker, FieldStatus, Fields, Flex, Form, Options, SegmentedControl, Select, TagsField,
    TextField, Variant,
};
use libero::sx::ThemeAwareValue;
use libero::theme::{ChoiceVariant, Size, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/responsive-form", || rsx! { ResponsiveFormPage {} }),
    ("/responsive-pickers", || rsx! { ResponsivePickersPage {} }),
];

#[derive(Clone, PartialEq, Options)]
enum View {
    List,
    Grid,
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Draft {
    name: String,
}

/// A field small under 62rem and large from it, beside plain `sm` and `lg` ones to measure
/// against; a checkbox that turns into a card; a segmented control whose segments, look and gap
/// switch at md; a late field registers its copies after mount.
#[component]
fn ResponsiveFormPage() -> Element {
    let mut late = use_signal(|| false);
    let draft = use_store(Draft::default);
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
            // Two responsive axes crossing in one rule: the card's padding per size (todo 5666).
            div { id: "rspf-cross",
                Checkbox {
                    label: "Large card from md",
                    size: responsive(Size::Sm).md(Size::Xl),
                    variant: responsive(ChoiceVariant::Plain).md(ChoiceVariant::Card),
                }
            }
            div { id: "rspf-cross-xl",
                Checkbox { label: "Large card", size: "xl", variant: "card" }
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
            div { id: "rspf-tags",
                TagsField {
                    label: "Tags",
                    value: vec!["alpha".to_string()],
                    size: responsive(Size::Sm).md(Size::Lg),
                    onchange: move |_: Vec<String>| {},
                }
            }
            div { id: "rspf-select",
                Select {
                    label: "View",
                    value: View::List,
                    size: responsive(Size::Sm).md(Size::Lg),
                    radius: responsive(Size::Xs).md(Size::Xl),
                    onchange: move |_| {},
                }
            }
            div { id: "rspf-pending",
                Form { value: draft, pending_loader: true,
                    TextField {
                        label: "Name",
                        status: FieldStatus::Pending,
                        size: responsive(Size::Sm).md(Size::Lg),
                    }
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

/// A color picker, a calendar and a color field small under 62rem and large from it (todo 5705).
#[component]
fn ResponsivePickersPage() -> Element {
    let mut color = use_signal(ColorCode::default);
    rsx! {
        Flex { direction: "column", gap: "md",
            div { id: "rspp-color",
                ColorPicker {
                    value: color(),
                    with_alpha: true,
                    swatches: ["#fa5252"],
                    size: responsive(Size::Sm).md(Size::Lg),
                    oninput: move |event: libero::components::SliderChangeEvent<ColorCode>| color.set(event.value()),
                }
            }
            div { id: "rspp-date",
                DatePicker { size: responsive(Size::Sm).md(Size::Lg) }
            }
            div { id: "rspp-color-field",
                ColorField {
                    label: "Accent",
                    value: ColorCode::default(),
                    swatches: ["#fa5252"],
                    size: responsive(Size::Sm).md(Size::Lg),
                    oninput: move |_| {},
                }
            }
        }
    }
}
