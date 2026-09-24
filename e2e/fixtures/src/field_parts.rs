//! The `parts` prop on the form components: one part of each, styled, and a
//! field nested in a slot that must keep its own look.

use dioxus::prelude::*;
use libero::components::{
    Checkbox, CheckboxPart, Chip, ChipPart, FieldPart, Fieldset, FieldsetPart, FileField,
    FileFieldPart, Flex, MultiSelect, Parts, Select, SelectPart, Slider, SliderPart, TextField,
    Textarea, TextareaPart,
};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/field-parts", || rsx! { FieldPartsPage {} })];

#[component]
fn FieldPartsPage() -> Element {
    let fruits = vec!["Apple".to_string(), "Cherry".to_string()];
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            TextField { id: "text", label: "Name", required: true, helper: "Your full name",
                parts: Parts::new()
                    .part(FieldPart::Label, sx().font_style("italic"))
                    .part(FieldPart::Required, sx().letter_spacing("3px"))
                    .part(FieldPart::Helper, sx().letter_spacing("2px"))
                    .part(FieldPart::Frame, sx().border_width("3px"))
                    .part(FieldPart::Control, sx().letter_spacing("5px"))
                    .part(FieldPart::Leading, sx().padding_right("7px")),
                leading: rsx! {
                    TextField { id: "nested", label: "Nested", value: "x" }
                },
            }
            Textarea { id: "textarea", label: "Bio", counter: true, maxlength: "20",
                parts: Parts::new().part(TextareaPart::Counter, sx().letter_spacing("3px")),
            }
            Select { id: "select", label: "Fruit", options: fruits.clone(), placeholder: "Pick one",
                onchange: move |_| {},
                parts: Parts::new()
                    .part(SelectPart::Frame, sx().border_width("3px"))
                    .part(SelectPart::Value, sx().font_style("italic")),
            }
            MultiSelect { id: "multi", label: "Fruits", options: fruits.clone(),
                value: vec!["Cherry".to_string()],
                onchange: move |_| {},
                parts: Parts::new().part(SelectPart::Chip, sx().padding_left("6px")),
            }
            Checkbox { id: "checkbox", label: "Agree",
                parts: Parts::new().part(CheckboxPart::Box, sx().border_width("3px")),
            }
            Slider { label: "Volume", value: 40.0, oninput: move |_| {},
                parts: Parts::new().part(SliderPart::Thumb, sx().border_width("3px")),
            }
            FileField { id: "file", label: "Resume", onchange: move |_| {},
                parts: Parts::new().part(FileFieldPart::Browse, sx().letter_spacing("3px")),
            }
            Chip { id: "chip", icon: rsx! { "*" },
                parts: Parts::new().part(ChipPart::Icon, sx().padding_left("6px")),
                "Tagged"
            }
            Fieldset::<()> { id: "fieldset", label: "Address",
                parts: Parts::new().part(FieldsetPart::Legend, sx().font_style("italic")),
                TextField { label: "Street" }
            }
        }
    }
}
