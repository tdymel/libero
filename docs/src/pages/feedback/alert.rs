use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
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
    // Mounted on demand: `role="alert"` mounted with its text on first paint
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
                    .doc("The heading, and the alert's accessible name through `aria-labelledby`. Text, not markup - markup in a name is dropped from it."),
                prop("icon", "Element")
                    .doc("A leading glyph, rendered `aria-hidden`. The library ships no icon set; this is your own."),
                prop("color", "ThemeAwareValue")
                    .default("info")
                    .doc("The tint; a theme color name or a literal CSS color. Severity is yours to state - `Form`'s summary passes `error`."),
                prop("variant", "Variant")
                    .default("tonal")
                    .doc("Chrome, shared with `Button` and `Badge`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. No hover response - an alert is not a target."),
                prop("radius", "Size")
                    .default("md")
                    .doc("A step on the radius scale. Anything else goes through `sx`."),
                prop("onclose", "EventHandler<()>")
                    .doc("Its presence is what shows the close button. Closing is yours: unmount the alert."),
                prop("close_label", "String")
                    .default("Close")
                    .doc("The close button's accessible name."),
                prop("children", "Element").doc("The message, and the alert's description through `aria-describedby`."),
            ])],
            lead: rsx! {
                Text {
                    "A tinted surface for something the reader has to know. It renders "
                    Code { source: "role=\"alert\"" }
                    " as a default your own "
                    Code { source: "role" }
                    " replaces - a message that should wait its turn takes "
                    Code { source: "role: \"status\"" }
                    ". It takes no focus and does not close on Escape: it is not an overlay. "
                    "The icon is "
                    Code { source: "aria-hidden" }
                    ": it repeats what the title already says. "
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
        }
    }
}
