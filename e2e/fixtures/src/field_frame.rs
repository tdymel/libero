//! The frame every framed field shares: its padding is part of the press target
//! (todo 462). One field per `data-case`, so the unit finds each frame.

use dioxus::prelude::*;
use libero::{
    chrono::{NaiveDate, NaiveTime},
    components::{
        ColorCode, ColorField, DateField, FileField, Files, Flex, MultiSelect, NativeSelect,
        NumberField, PhoneField, PinField, Select, SliderChangeEvent, TextField, Textarea,
        TimeField,
    },
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[("/field-frame", || rsx! { FieldFramePage {} })];

#[component]
fn FieldFramePage() -> Element {
    let mut text = use_signal(|| "hello".to_string());
    let mut quantity = use_signal(|| 3i32);
    let mut fruit = use_signal(|| Some(Fruit::Banana));
    let mut fruits = use_signal(|| vec![Fruit::Cherry]);
    let mut note = use_signal(|| "line one".to_string());
    let mut pick = use_signal(|| Fruit::Banana);
    let mut pin = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut color = use_signal(|| "#1c7ed6".parse::<ColorCode>().unwrap());
    let mut day = use_signal(|| NaiveDate::from_ymd_opt(2026, 9, 25));
    let mut files = use_signal(Files::default);
    let mut time = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { "data-case": "text",
                TextField {
                    label: "Handle",
                    leading: rsx! { "@" },
                    value: text(),
                    oninput: move |next| text.set(next),
                }
            }
            div { "data-case": "number",
                NumberField {
                    label: "Quantity",
                    steppers: true,
                    value: quantity(),
                    onchange: move |next| quantity.set(next),
                }
            }
            div { "data-case": "select",
                Select {
                    label: "Fruit",
                    value: fruit(),
                    onchange: move |next| fruit.set(next),
                }
            }
            div { "data-case": "multi",
                MultiSelect {
                    label: "Fruits",
                    value: fruits(),
                    onchange: move |next| fruits.set(next),
                }
            }
            div { "data-case": "textarea",
                Textarea {
                    label: "Note",
                    value: note(),
                    oninput: move |next| note.set(next),
                }
            }
            div { "data-case": "native",
                NativeSelect {
                    label: "Pick",
                    value: pick(),
                    onchange: move |next| pick.set(next),
                }
            }
            div { "data-case": "pin",
                PinField {
                    label: "Code",
                    length: 4usize,
                    value: pin(),
                    oninput: move |next: String| pin.set(next),
                }
            }
            div { "data-case": "phone",
                PhoneField {
                    label: "Phone",
                    value: phone(),
                    oninput: move |next: String| phone.set(next),
                }
            }
            div { "data-case": "color",
                ColorField {
                    label: "Accent",
                    value: color(),
                    oninput: move |event: SliderChangeEvent<ColorCode>| {
                        if let SliderChangeEvent::Change(next) = event {
                            color.set(next);
                        }
                    },
                }
            }
            div { "data-case": "date",
                DateField {
                    label: "Arrival",
                    value: day(),
                    onchange: move |next| day.set(next),
                }
            }
            div { "data-case": "time",
                TimeField {
                    label: "Start",
                    value: time(),
                    onchange: move |next| time.set(next),
                }
            }
            div { "data-case": "file",
                FileField {
                    label: "Attachment",
                    value: files(),
                    onchange: move |next: Files| files.set(next),
                }
            }
        }
    }
}
