//! `NativeSelect` and `Textarea`: wiring, placeholder, refused pick, groups, disabled and
//! read-only states. One field per `data-case`.

use dioxus::prelude::*;
use libero::components::{
    Flex, NativeSelect, OptionItem, OptionList, Options, Rule, Textarea, not_empty,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[("/native-select", || rsx! { NativeSelectPage {} })];

#[component]
fn NativeSelectPage() -> Element {
    let mut pick = use_signal(|| None::<Fruit>);
    let mut kept = use_signal(|| Fruit::Banana);
    let mut grouped = use_signal(|| Fruit::Apple);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { "data-case": "wired",
                NativeSelect {
                    label: "Fruit",
                    description: "For the smoothie.",
                    helper: "One per order.",
                    placeholder: "Pick one",
                    required: true,
                    validate: not_empty.error("Fruit needed"),
                    value: pick(),
                    onchange: move |next| pick.set(Some(next)),
                }
                span { "data-echo": "wired",
                    "{pick().map(|fruit| fruit.value()).unwrap_or_default()}"
                }
                button { id: "clear", onclick: move |_| pick.set(None), "Clear" }
            }
            // The caller keeps `Banana` whatever is picked.
            div { "data-case": "refused",
                NativeSelect {
                    label: "Locked fruit",
                    value: kept(),
                    onchange: move |_| kept.set(Fruit::Banana),
                }
            }
            // One option refused, the rest in named groups.
            div { "data-case": "grouped",
                NativeSelect {
                    label: "Grouped fruit",
                    value: grouped(),
                    onchange: move |next| grouped.set(next),
                    options: OptionList::grouped()
                        .group("Pome", [Fruit::Apple])
                        .group(
                            "Stone",
                            [OptionItem::new(Fruit::Cherry).disabled(true), Fruit::Damson.into()],
                        ),
                }
            }
            div { "data-case": "disabled",
                NativeSelect { label: "Off", disabled: true, value: Fruit::Apple, onchange: |_| {} }
            }
            div { "data-case": "textarea",
                Textarea {
                    label: "Notes",
                    description: "Anything the team should know.",
                    helper: "Markdown is not rendered.",
                    placeholder: "Start typing",
                    required: true,
                    rows: 5,
                }
            }
            div { "data-case": "textarea-readonly",
                Textarea { label: "Terms", readonly: true, value: "Read me." }
            }
            div { "data-case": "textarea-disabled",
                Textarea { label: "Closed", disabled: true, value: "Too late." }
            }
        }
    }
}
