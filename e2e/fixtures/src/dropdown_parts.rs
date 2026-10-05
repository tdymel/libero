//! The `dropdown_parts` prop on the portaled dropdowns: a list field's, a date
//! field's calendar and a color field's picker.

use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{
    Cascader, CascaderOption, ChronoDropdownPart, ColorCode, ColorDropdownPart, ColorField,
    DateField, DropdownPart, Flex, Parts, Select, Swatches,
};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/dropdown-parts", || rsx! { DropdownPartsPage {} })];

#[component]
fn DropdownPartsPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            Select { id: "select", label: "Fruit", searchable: true,
                options: vec!["Apple".to_string(), "Cherry".to_string()],
                onchange: move |_| {},
                dropdown_parts: Parts::new()
                    .part(DropdownPart::Panel, sx().padding("11px"))
                    .part(DropdownPart::Search, sx().letter_spacing("2px"))
                    .part(DropdownPart::Option, sx().letter_spacing("3px")),
            }
            Cascader { id: "cascader", label: "Category", onchange: move |_: Option<String>| {},
                data: vec![CascaderOption::<String>::new("fruit", "Fruit").children(vec![
                    CascaderOption::new("apple", "Apple"),
                ])],
                dropdown_parts: Parts::new().part(DropdownPart::Column, sx().padding_top("9px")),
            }
            DateField { id: "date", label: "Day", value: NaiveDate::from_ymd_opt(2026, 9, 14),
                onchange: move |_| {},
                dropdown_parts: Parts::new()
                    .part(ChronoDropdownPart::Panel, sx().padding("11px"))
                    .part(ChronoDropdownPart::Header, sx().padding_top("7px"))
                    .part(ChronoDropdownPart::Day, sx().letter_spacing("3px")),
            }
            ColorField { id: "color", label: "Accent", value: ColorCode::hex(0x228be6),
                oninput: move |_| {},
                swatches: Swatches::new(vec![ColorCode::hex(0xfa5252)]),
                dropdown_parts: Parts::new()
                    .part(ColorDropdownPart::Swatch, sx().margin_top("5px"))
                    .part(ColorDropdownPart::Thumb, sx().border_width("3px")),
            }
        }
    }
}
