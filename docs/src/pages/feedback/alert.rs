use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use crate::icons::CheckmarkIcon;
use dioxus::prelude::*;
use libero::{
    components::{Alert, Button, Code, Flex, Text},
    hooks::use_element,
    platform::ElementApi,
    use_theme,
};

const TITLE: &str = "Card expiring";
const MESSAGE: &str = "Your card ends 09/26. Update it before the next invoice.";

#[component]
pub fn AlertPage() -> Element {
    let theme = use_theme();
    // Mounted on demand: a live region mounted with its text on first paint
    // is the live-region shape `Notifications` avoids.
    let mut shown = use_signal(|| false);
    let trigger = use_element();

    rsx! {
        DocPage {
            title: "Alert",
            source: "libero/src/components/feedback/alert.rs",
            markdown: "/md/alert.md",
            properties: vec![props("Alert", vec![
                prop("title", "String")
                    .doc("The heading and the alert's accessible name. Text only."),
                prop("icon", "Element")
                    .doc("A leading icon of your own, hidden from screen readers."),
                prop("color", "ThemeAwareValue")
                    .default("info")
                    .doc("The tint. A theme color name or any CSS color. `error` and `warning` make the role `alert`, the rest `status`."),
                prop("variant", "Variant")
                    .default("tonal")
                    .doc("Visual style, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`."),
                prop("radius", "Size")
                    .default("md")
                    .doc("Corner radius. A size step or any CSS length."),
                prop("onclose", "EventHandler<()>")
                    .doc("Shows the close button and fires when it is pressed. Unmount the alert to close it."),
                prop("close_label", "String")
                    .default("common.close")
                    .doc("The close button's accessible name. Unset, the localization's `common.close`, \"Close\" in English."),
                prop("children", "Element")
                    .default("required")
                    .doc("The message, read as the alert's description."),
            ])],
            lead: rsx! {
                Text {
                    "A tinted surface for something the reader has to know, such as an error, "
                    "a warning or a note. It takes no focus and does not close on Escape, "
                    "because it is not an overlay. "
                    Code { source: "Form" }
                    "'s error summary is an "
                    Code { source: "Alert" }
                    " with "
                    Code { source: "color: \"error\"" }
                    "."
                }
            },
            Demo {
                component: "Alert",
                children_text: MESSAGE,
                controls: vec![
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"])
                    .default(theme.alert.variant.as_str()),
                    Control::color("color")
                    .default(theme.alert.color),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.alert.radius.as_str()),
                    Control::switch("title").default("true").code(|_, values| {
                        match values.str("title").as_str() {
                            "true" => vec![format!("title: {TITLE:?}")],
                            _ => vec![],
                        }
                    }),
                    Control::switch("icon").code(|_, values| {
                        match values.str("icon").as_str() {
                            "true" => vec!["icon: rsx! { CheckmarkIcon {} }".to_string()],
                            _ => vec![],
                        }
                    }),
                    // A closure is not a value a control can hold, so this
                    // prints the honest minimum.
                    Control::switch("onclose").code(|_, values| {
                        match values.str("onclose").as_str() {
                            "true" => vec!["onclose: move |_| {}".to_string()],
                            _ => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Flex { direction: "column", align: "center", gap: "md",
                        Button {
                            variant: "outlined",
                            onmounted: trigger.mount(),
                            onclick: move |_| shown.toggle(),
                            if shown() { "Hide alert" } else { "Show alert" }
                        }
                        if shown() {
                            Alert {
                                variant: values.str("variant"),
                                color: values.str("color"),
                                radius: values.str("radius"),
                                title: (values.str("title") == "true").then(|| TITLE.to_string()),
                                icon: (values.str("icon") == "true").then(|| rsx! { CheckmarkIcon {} }),
                                onclose: (values.str("onclose") == "true")
                                    .then(|| EventHandler::new(move |_| {
                                        shown.set(false);
                                        // The close button unmounts with it.
                                        let _ = trigger.focus();
                                    })),
                                {MESSAGE}
                            }
                        }
                    }
                },
            }
            DocSection { title: "Accessibility",
                Text {
                    "An "
                    Code { source: "error" }
                    " or "
                    Code { source: "warning" }
                    " color renders "
                    Code { source: "role=\"alert\"" }
                    ", which interrupts a screen reader. Every other color renders the polite "
                    Code { source: "role=\"status\"" }
                    ". Your own "
                    Code { source: "role" }
                    " replaces either."
                }
                Text {
                    "The icon is hidden from screen readers, so say the severity in the title "
                    "or the message too. "
                    Code { source: "outlined" }
                    " has no tint, so prefer "
                    Code { source: "tonal" }
                    " or "
                    Code { source: "filled" }
                    " for an error. Closing removes the focused close button, so move focus "
                    "somewhere sensible in "
                    Code { source: "onclose" }
                    "."
                }
            }
        }
    }
}
