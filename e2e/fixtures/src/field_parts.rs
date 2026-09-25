//! The `parts` prop on the form components: one part of each, styled, and a
//! field nested in a slot that must keep its own look.

use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, CheckboxPart, Chip, ChipPart, ColorCode, ColorPicker, ColorPickerPart,
    ColorSliderPart, Combobox, ComboboxOption, ComboboxOptionArgs, DropdownPart, FieldPart,
    Fieldset, FieldsetPart, FileField, FileFieldPart, Flex, Form, FormPart, HueSlider, MultiSelect,
    OptionList, Parts, Rule, Select, SelectPart, Slider, SliderPart, Swatches, TextField, Textarea,
    TextareaPart, not_empty, use_combobox,
};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/field-parts", || rsx! { FieldPartsPage {} }),
    ("/form-parts", || rsx! { FormPartsPage {} }),
];

/// Form, Combobox and the color components, apart: their thumbs and labels
/// would be found first on the page above.
#[component]
fn FormPartsPage() -> Element {
    let state = use_combobox();
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            Form::<()> { id: "form", summary_title: "Fix these",
                parts: Parts::new()
                    .part(FormPart::Summary, sx().padding("13px"))
                    .part(FormPart::SummaryTitle, sx().font_style("italic"))
                    .part(FormPart::SummaryList, sx().letter_spacing("3px")),
                TextField { label: "Email", validate: not_empty::<String>.error("Enter your email.") }
                button { id: "submit", r#type: "submit", "Send" }
            }
            Combobox {
                state,
                options: OptionList::grouped().group("Fruit", ["Apple", "Pear"]),
                option: move |row: ComboboxOptionArgs<&'static str>| rsx! {
                    ComboboxOption { onpick: move |_| state.close(), "{row.value}" }
                },
                parts: Parts::new()
                    .part(DropdownPart::Option, sx().letter_spacing("3px"))
                    .part(DropdownPart::GroupLabel, sx().font_style("italic")),
                Button { id: "trigger", attributes: state.a11y_attributes(), onclick: move |_| state.toggle(), "Fruit" }
            }
            ColorPicker { id: "picker", value: ColorCode::hex(0x228be6), with_alpha: true,
                swatches: Swatches::new(vec![ColorCode::hex(0xfa5252)]),
                parts: Parts::new()
                    .part(ColorPickerPart::Thumb, sx().border_width("3px"))
                    .part(ColorPickerPart::Swatch, sx().margin_top("5px")),
            }
            HueSlider { id: "hue", value: 120.0, oninput: move |_| {},
                parts: Parts::new().part(ColorSliderPart::Track, sx().margin_top("7px")),
            }
        }
    }
}

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
            Chip { id: "link-chip", to: "https://example.com", target: "_blank",
                parts: Parts::new().part(ChipPart::NewTab, sx().padding_left("4px")),
                "Docs"
            }
            Fieldset::<()> { id: "fieldset", label: "Address",
                parts: Parts::new().part(FieldsetPart::Legend, sx().font_style("italic")),
                TextField { label: "Street" }
            }
        }
    }
}
