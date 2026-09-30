use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, NavLink, NavLinkPart, Text},
    sx::sx,
    use_theme,
};

const DESCRIPTION: &str = "Where to start";

const NESTED_CODE: &str = "nested: rsx! {
    NavLink { to: Route::NavLinkPage {}, active: false, \"Install\" }
    NavLink { to: Route::NavLinkPage {}, active: false, \"Theming\" }
}";

/// Two links, so `active: auto` can be seen deciding between them. Only `to`
/// and the label differ, so each is spliced into its own copy of the rsx.
fn wrap_links(_: &DemoValues, code: &str) -> String {
    let body = code
        .strip_prefix("NavLink {")
        .unwrap_or(code)
        .trim_end_matches('}');
    // No props left collapses to `NavLink {}`, and the child still needs its
    // own line.
    let body = match body.is_empty() {
        true => "\n",
        false => body,
    };
    let link = |to: &str, label: &str| format!("NavLink {{\n    to: {to},{body}    {label:?}\n}}");

    format!(
        "Flex {{\n    direction: \"column\",\n    gap: \"xs\",\n    sx: sx().width(\"240px\"),\n{}{}}}",
        indent(&link("Route::GettingStarted {}", "Getting Started")),
        indent(&link("Route::NavLinkPage {}", "NavLink")),
    )
}

#[component]
pub fn NavLinkPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "NavLink",
            source: "libero/src/components/navigation/nav_link.rs",
            markdown: "/md/nav_link.md",
            properties: vec![props("NavLink", vec![
                prop("to", "NavigationTarget").default("required").doc("A path, a URL or a typed route, as in `Anchor::to`."),
                prop("target", "String").doc("The link's `target` attribute. `\"_blank\"` adds a small external icon and a hidden \"(opens in a new tab)\"."),
                prop("new_tab_hint", "bool")
                    .default("true")
                    .doc("`false` drops the icon and the hidden text a `\"_blank\"` target adds."),
                prop("color", "ThemeAwareValue").default("primary").doc("Colours the active link's tint and start bar. Only the color family counts: the tint is its lightest shade."),
                prop("active", "bool")
                    .default("follows the route")
                    .doc("Unset, it compares `to` with the current route, which needs an internal target and a router. Set it for a section's parent item, or where there is no route to compare."),
                prop("disabled", "bool").default("false").doc("Dims the link and stops navigation."),
                prop("scroll_into_view", "bool")
                    .default("false")
                    .doc("Scrolls the link into view when it becomes active. It scrolls the nearest scrollable ancestor, else the page, so use it in a sidebar."),
                prop("description", "String").doc("A dimmed line under the label, read as the link's description."),
                prop("nested", "Element").doc("Child `NavLink`s, shown under this one by a toggle button beside it. The link itself still goes to `to`."),
                prop("opened", "bool").doc("Whether `nested` shows. Setting it makes it controlled, so pair it with `onchange`."),
                prop("default_opened", "bool").default("false").doc("Whether `nested` shows at first, when `opened` is unset."),
                prop("onchange", "EventHandler<bool>").doc("Called with the new `opened` when the toggle is pressed."),
                prop("parts", "Parts<NavLinkPart>").doc("Styles for the inner parts in the Style API tab, under `sx`."),
                prop("children", "Element").default("required").doc("The link's content."),
            ])
            .parts("NavLinkPart", vec![
                (NavLinkPart::Body, "The column holding the label and the description, with `description` only."),
                (NavLinkPart::Label, "The link's content, with `description` only."),
                (NavLinkPart::Description, "The dimmed line under the label."),
                (NavLinkPart::NewTab, "The new-tab icon after the label, with `target: \"_blank\"` only."),
            ])],
            accessibility: a11y()
                .handles([
                    "`description` is read as the link's description, not its name, so \"Docs\" stays \"Docs\" in a links list.",
                    "With `nested`, a disclosure button follows the link with its own tab stop. It carries `aria-expanded` and `aria-controls`, and its name is the localized \"Show links\" plus the link's name (\"Show links Docs\").",
                    "Enter or Space on the disclosure button toggles the panel, and the link still navigates.",
                    "A `target: \"_blank\"` link draws a small external icon and reads a hidden \"(opens in a new tab)\". `new_tab_hint: false` drops both.",
                ])
                .must(["Wrap a list of them in a `<nav>` to make a navigation landmark."])
                .limits(["In a native app, Tab still enters the nested links of a closed panel."]),
            lead: rsx! {
                Text {
                    "A navigation list item for a sidebar or nav bar. It is an "
                    Code { source: "Anchor" }
                    " that marks the current page with "
                    Code { source: "aria-current" }
                    ", a light tint of its "
                    Code { source: "color" }
                    " and a 2px bar in a darker shade at its start edge. Left unset, "
                    Code { source: "active" }
                    " compares "
                    Code { source: "to" }
                    " with the current route."
                }
            },
            // snippet: item #[derive(Clone, PartialEq, Routable)] enum Route { #[route("/")] GettingStarted {}, #[route("/nav-link")] NavLinkPage {} }
            // snippet: item #[component] fn GettingStarted() -> Element { rsx! {} }
            // snippet: item #[component] fn NavLinkPage() -> Element { rsx! {} }
            Demo {
                component: "NavLink",
                children_text: "",
                controls: vec![
                    // Unset compares `to` against the current route, which
                    // is why the second link reads active on its own.
                    Control::toggle("active", ["auto", "false", "true"])
                        .labels(["Auto", "Off", "On"])
                        .code(
                        |_, values| match values.str("active").as_str() {
                            "auto" => vec![],
                            active => vec![format!("active: {active}")],
                        },
                    ),
                    Control::color("color")
                    .default(theme.nav_link.color.as_str()),
                    Control::switch("disabled"),
                    Control::switch("description").code(|_, values| match values.str("description").as_str() {
                        "true" => vec![format!("description: {DESCRIPTION:?}")],
                        _ => vec![],
                    }),
                    Control::switch("nested").code(|_, values| match values.str("nested").as_str() {
                        "true" => vec![NESTED_CODE.to_string()],
                        _ => vec![],
                    }),
                ],
                render: move |values: DemoValues| {
                    let active = match values.str("active").as_str() {
                        "auto" => None,
                        active => Some(active == "true"),
                    };
                    let color = values.str("color");
                    let disabled = (values.str("disabled") == "true").then_some(true);
                    let description = (values.str("description") == "true").then(|| DESCRIPTION.to_string());
                    let nested = || {
                        (values.str("nested") == "true").then(|| rsx! {
                            NavLink { to: crate::Route::NavLinkPage {}, active: false, "Install" }
                            NavLink { to: crate::Route::NavLinkPage {}, active: false, "Theming" }
                        })
                    };
                    rsx! {
                        Flex {
                            direction: "column",
                            gap: "xs",
                            sx: sx().width("240px"),
                            NavLink {
                                to: crate::Route::GettingStarted {},
                                active,
                                color: color.clone(),
                                disabled,
                                description: description.clone(),
                                nested: nested(),
                                "Getting Started"
                            }
                            NavLink {
                                to: crate::Route::NavLinkPage {},
                                active,
                                color,
                                disabled,
                                description,
                                nested: nested(),
                                "NavLink"
                            }
                        }
                    }
                },
                wrap: Wrap(wrap_links),
            }
        }
    }
}
