use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, NavLink, Text},
    sx::sx,
    use_theme,
};

/// Two links, so `active: auto` can be seen deciding *between* them - one is
/// this very page, the other is not. Only `to` and the label differ, so each
/// is spliced into its own copy of the generated rsx.
fn wrap_links(_: &DemoValues, code: &str) -> String {
    let body = code
        .strip_prefix("NavLink {")
        .unwrap_or(code)
        .trim_end_matches('}');
    // No props left at all collapses to `NavLink {}` - the child still needs
    // its own line.
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
                prop("to", "NavigationTarget").doc("A plain path/URL or a typed route, same as `Anchor::to`."),
                prop("target", "String").doc("The link's `target` attribute."),
                prop("color", "ThemeAwareValue").default("primary").doc("Tints the active/hover background."),
                prop("active", "bool")
                    .default("follows the route")
                    .doc("Unset, it compares `to` against the current route, so it is only ever true for an internal target with a router mounted. Set it explicitly for a section-level parent item, or anywhere auto-detection has nothing to compare against."),
                prop("disabled", "bool").default("false").doc("Dims the link and disables navigation."),
                prop("scroll_into_view", "bool")
                    .default("false")
                    .doc("Scrolls this link into view when it becomes active, if it isn't already visible. Acts on whatever scrollable ancestor happens to exist, which only suits a sidebar."),
                prop("children", "Element").doc("The link's content."),
            ])],
            lead: rsx! {
                Text {
                    "A navigation list item - "
                    Code { source: "Anchor" }
                    " plus a themed active/hover background and "
                    Code { source: "aria-current" }
                    ", for a sidebar or nav bar link. Colors come from the theme ("
                    Code { source: "Theme::nav_link" }
                    ") by default, and only show once a link is active."
                }
            },
            Demo {
                component: "NavLink",
                children_text: "",
                controls: vec![
                    // Unset compares `to` against the current route, which
                    // is why the second link reads active on its own.
                    Control::toggle("active", ["auto", "true", "false"]).code(
                        |_, values| match values.str("active").as_str() {
                            "auto" => vec![],
                            active => vec![format!("active: {active}")],
                        },
                    ),
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info"],
                    )
                    .default(theme.nav_link.color.as_str()),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| {
                    let active = match values.str("active").as_str() {
                        "auto" => None,
                        active => Some(active == "true"),
                    };
                    let color = values.str("color");
                    let disabled = (values.str("disabled") == "true").then_some(true);
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
                                "Getting Started"
                            }
                            NavLink {
                                to: crate::Route::NavLinkPage {},
                                active,
                                color,
                                disabled,
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
