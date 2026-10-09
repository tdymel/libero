//! Controls under 24px and their invisible 24x24 hit areas (todos 505, 566),
//! and the framed fields whose slot buttons must not grow the frame (todo 495).
//! On a coarse pointer, small controls take presses in a 44x44 box (todo 2707).

use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Burger, Checkbox, Chip, ColorCode, ColorField, Dialog, Flex, NumberField,
        Pagination, PasswordField, Radio, Select, SliderChangeEvent, Switch, TagsField,
        TextField,
    },
    theme::Size,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[("/hit-area", || rsx! { HitAreaPage {} })];

const SIZES: [(Size, &str); 3] = [(Size::Xs, "xs"), (Size::Md, "md"), (Size::Xl, "xl")];

#[component]
fn HitAreaPage() -> Element {
    let mut topics = use_signal(|| vec!["rust".to_string(), "wasm".to_string()]);
    rsx! {
        Flex { direction: "column", gap: "lg", max_width: "360px",
            for (size, name) in SIZES {
                Fields { key: "{name}", size, name }
            }
            Dialog {
                id: "dialog",
                title: "Filters",
                onclose: move |_| {},
                sx: libero::sx::sx().margin("0"),
                "Narrow the list."
            }
            Flex { gap: "lg", align: "center",
                Burger { id: "burger-xs", size: "xs", onclick: move |_| {} }
                Burger { id: "burger-sm", size: "sm", onclick: move |_| {} }
                ActionIcon { id: "icon-xs", aria_label: "Extra small", size: "16px", {glyph()} }
                ActionIcon { id: "icon-sm", aria_label: "Small", size: "20px", {glyph()} }
                ActionIcon { id: "icon-md", aria_label: "Medium", size: "md", {glyph()} }
            }
            div { "data-case": "tags",
                TagsField {
                    label: "Topics",
                    clearable: true,
                    value: topics(),
                    onchange: move |next| topics.set(next),
                }
            }
            Coarse {}
        }
    }
}

/// Controls whose hit area grows to 44x44 on a coarse pointer (todo 2707), 48px apart
/// so no neighbour takes a share of it.
#[component]
fn Coarse() -> Element {
    let mut vegan = use_signal(|| false);
    let mut agree = use_signal(|| false);
    let mut terrace = use_signal(|| false);
    rsx! {
        Flex { direction: "column", align: "flex-start", sx: libero::sx::sx().gap("48px"),
            Chip { id: "chip-filter", checked: vegan(), onchange: move |next| vegan.set(next), "Vegan" }
            Chip { id: "chip-action", onclick: move |_| {}, "Share" }
            div { "data-case": "checkbox",
                Checkbox {
                    aria_label: "Agree",
                    checked: agree(),
                    onchange: move |next| agree.set(next),
                }
            }
            div { "data-case": "radio",
                Radio { aria_label: "Express", checked: false, onselect: move |_| {} }
            }
            div { "data-case": "switch",
                Switch {
                    aria_label: "Terrace",
                    checked: terrace(),
                    onchange: move |next| terrace.set(next),
                }
            }
            div { "data-case": "pagination",
                Pagination { total: 5, page: 2, aria_label: "Pages", onchange: move |_: u32| {} }
            }
        }
    }
}

/// One of each field with a slot button, at `size`, for their frame heights.
#[component]
fn Fields(size: Size, name: &'static str) -> Element {
    let mut quantity = use_signal(|| Some(3i32));
    let mut fruit = use_signal(|| Some(Fruit::Banana));
    let mut color = use_signal(|| "#1c7ed6".parse::<ColorCode>().unwrap());
    rsx! {
        Flex { direction: "column", gap: "sm", "data-size": name,
            div { "data-case": "text-{name}",
                TextField { label: "Text {name}", size, value: "hello", oninput: move |_| {} }
            }
            div { "data-case": "password-{name}",
                PasswordField { label: "Password {name}", size, value: "secret" }
            }
            div { "data-case": "number-{name}",
                NumberField {
                    label: "Number {name}",
                    size,
                    steppers: true,
                    value: quantity(),
                    onchange: move |next| quantity.set(next),
                }
            }
            div { "data-case": "color-{name}",
                ColorField {
                    label: "Color {name}",
                    size,
                    value: color(),
                    oninput: move |event: SliderChangeEvent<ColorCode>| {
                        if let SliderChangeEvent::Change(next) = event {
                            color.set(next);
                        }
                    },
                }
            }
            div { "data-case": "select-{name}",
                Select {
                    label: "Fruit {name}",
                    size,
                    clearable: true,
                    value: fruit(),
                    onchange: move |next| fruit.set(next),
                }
            }
        }
    }
}

fn glyph() -> Element {
    rsx! {
        svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "8" } }
    }
}
