use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Switch, Text},
    hooks::use_accessibility,
};

/// The forced setting is kept across visits, so leaving the page hands the site back to the
/// system if the demo forced it.
#[component]
pub fn Settings() -> Element {
    let accessibility = use_accessibility();
    let mut forced = use_hook(|| CopyValue::new(false));
    use_drop({
        let accessibility = accessibility.clone();
        move || {
            if forced() {
                accessibility.set_reduced_motion(None);
            }
        }
    });

    rsx! {
        Flex { direction: "column", gap: "sm",
            Switch {
                label: "Reduce motion",
                checked: accessibility.reduced_motion(),
                onchange: {
                    let accessibility = accessibility.clone();
                    move |on: bool| {
                        forced.set(true);
                        accessibility.set_reduced_motion(Some(on));
                    }
                },
            }
            Button {
                variant: "outlined",
                onclick: {
                    let accessibility = accessibility.clone();
                    move |_| accessibility.set_reduced_motion(None)
                },
                "Follow the system"
            }
            Text {
                "Contrast: {accessibility.contrast().as_str()}, forced colors: "
                "{accessibility.forced_colors()}, reduced transparency: "
                "{accessibility.reduced_transparency()}"
            }
        }
    }
}
