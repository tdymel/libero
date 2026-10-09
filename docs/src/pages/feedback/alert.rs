use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::{
    components::{Alert, AlertPart, Button, Code, Flex, Text},
    hooks::use_element,
    platform::ElementApi,
    use_theme,
};

const TITLE: &str = "Card expiring";
const MESSAGE: &str = "Your card ends 09/26. Update it before the next invoice.";
const ACTION_CODE: &str =
    "Button { variant: \"standard\", color: \"currentColor\", size: \"xs\", \"Update card\" }";

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
                prop("icon", "Option<Element>")
                    .doc("A leading icon of your own, hidden from screen readers."),
                prop("color", "ThemeAwareValue")
                    .default("info")
                    .doc("The tint. A theme color name or any CSS color. `error` and `warning` make the role `alert`, the rest `status`."),
                prop("variant", "Variant")
                    .default(theme.alert.variant.as_str())
                    .doc("Visual style, shared with `Button`: `filled`, `tonal`, `elevated`, `outlined`, `standard`."),
                prop("radius", "ThemeAwareValue")
                    .default(theme.alert.radius.as_str())
                    .doc("Corner radius, a size step from `xs` to `xxl`, or any CSS, e.g. `radius: \"0\"`."),
                prop("actions", "Option<Element>")
                    .doc("Buttons or links beside the message, before the close button. Give each a label that says what it does: `Button { variant: \"standard\", color: \"currentColor\", size: \"xs\" }` reads on every variant."),
                prop("onclose", "EventHandler<()>")
                    .doc("Shows the close button and fires when it is pressed. Unmount the alert to close it."),
                prop("close_label", "String")
                    .default("common.close")
                    .doc("The close button's accessible name. Unset, the localization's `common.close`, \"Close\" in English."),
                prop("parts", "Parts<AlertPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(AlertPart::Title, sx().font_weight(\"700\"))`."),
                prop("children", "Element")
                    .default("required")
                    .doc("The message, read as the alert's description."),
            ])
            .parts("AlertPart", vec![
                (AlertPart::Icon, "The icon wrapper."),
                (AlertPart::Body, "The column holding the title and the message."),
                (AlertPart::Title, "The title."),
                (AlertPart::Message, "The message."),
                (AlertPart::Actions, "The row holding `actions`."),
                (AlertPart::Close, "The close button."),
            ])],
            accessibility: a11y()
                .handles([
                    "An `error` or `warning` color renders `role=\"alert\"`, which interrupts a screen reader. Every other color renders the polite `role=\"status\"`. Your own `role` replaces either.",
                    "The icon is hidden from screen readers.",
                    "`actions` come after the message and before the close button in `Tab` order.",
                ])
                .must([
                    "Name each action by what it does, such as \"Update card\": the alert takes no focus, so a reader finds the button by `Tab`.",
                    "Say the severity in the title or the message too, as the icon is not read.",
                    "Prefer `tonal` or `filled` for an error: `outlined` has no tint.",
                    "Move the focus somewhere sensible in `onclose`: closing removes the focused close button.",
                    "Mount the Alert after the page loads to have it announced: one present at first paint is read only when the reader reaches it.",
                ])
                .example("A failed save, `Alert { color: \"error\", title: \"Error: not saved\" }` mounted after the click: a screen reader interrupts to read it, and the title says the severity in words."),
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
                    Control::sizes("radius")
                        .default(theme.alert.radius.as_str()),
                    Control::switch("title").default("true").code(|_, values| {
                        match values.str("title").as_str() {
                            "true" => vec![format!("title: {TITLE:?}")],
                            _ => vec![],
                        }
                    }),
                    Control::switch("icon").code(|_, values| {
                        match values.str("icon").as_str() {
                            "true" => vec!["icon: rsx! { Pictogram { icon: lucide::check::outlined } }".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("actions").default("true").code(|_, values| {
                        match values.str("actions").as_str() {
                            "true" => vec![format!("actions: rsx! {{ {ACTION_CODE} }}")],
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
                                icon: (values.str("icon") == "true").then(|| rsx! { Pictogram { icon: lucide::check::outlined } }),
                                actions: (values.str("actions") == "true").then(|| rsx! {
                                    Button {
                                        variant: "standard",
                                        color: "currentColor",
                                        size: "xs",
                                        "Update card"
                                    }
                                }),
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
