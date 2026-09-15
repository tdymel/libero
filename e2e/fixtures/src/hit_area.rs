//! Controls under 24px and their invisible 24x24 hit areas (todos 505, 566),
//! and the framed fields whose slot buttons must not grow the frame (todo 495).

use dioxus::prelude::*;
use libero::{
    components::{
        ActionIcon, Burger, ColorCode, ColorField, Dialog, Flex, NumberField, PasswordField,
        Select, SliderChangeEvent, TagsField, TextField,
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
                ActionIcon { id: "icon-xs", aria_label: "Extra small", size: "xs", {glyph()} }
                ActionIcon { id: "icon-sm", aria_label: "Small", size: "sm", {glyph()} }
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
