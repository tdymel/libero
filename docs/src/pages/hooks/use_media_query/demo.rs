use dioxus::prelude::*;
use libero::{
    components::{Flex, Text},
    hooks::{use_is_mobile, use_media_query},
};

#[component]
pub fn Layout() -> Element {
    let mobile = use_is_mobile();
    let wide = use_media_query("(min-width: 1024px)");

    rsx! {
        Flex { direction: "column", gap: "sm",
            Text {
                if mobile() {
                    "Mobile: under 768px"
                } else {
                    "Not mobile"
                }
            }
            Text {
                if wide() {
                    "Wide: 1024px or more"
                } else {
                    "Narrower than 1024px"
                }
            }
        }
    }
}
