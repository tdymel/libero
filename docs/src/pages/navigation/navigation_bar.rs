use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Box, Code, Indicator, NavigationBar, NavigationBarItem, NavigationBarPart, Pictogram,
        SvgData, Text,
    },
    sx::{Sx, sx},
    theme::Size,
    use_theme,
};
use pictogram_icons_lucide as lucide;

/// Label, path, lucide icon name and icon of each demo item.
const ITEMS: [(&str, &str, &str, SvgData); 4] = [
    ("Home", "/home", "house", lucide::house::outlined),
    ("Search", "/search", "search", lucide::search::outlined),
    ("Inbox", "/inbox", "inbox", lucide::inbox::outlined),
    ("Profile", "/profile", "user", lucide::user::outlined),
];
/// The item that carries the badge and, with `disabled`, is disabled.
const MARKED: usize = 2;

/// The props from the controls go on the bar; the items are spliced in under them.
fn wrap_bar(values: &DemoValues, code: &str) -> String {
    let body = code
        .strip_prefix("NavigationBar {")
        .unwrap_or(code)
        .trim_end_matches('}');
    let body = match body.is_empty() {
        true => "\n",
        false => body,
    };
    let badge = values.str("badge") == "true";
    let disabled = values.str("disabled") == "true";
    let items: String = ITEMS
        .iter()
        .enumerate()
        .map(|(index, (label, to, icon, _))| {
            let marked = index == MARKED;
            let mut lines = vec![format!("to: {to:?},")];
            if marked && badge {
                lines.push(format!("\"aria-label\": \"{label}, 3 unread\","));
            }
            lines.push(format!(
                "icon: rsx! {{ Pictogram {{ icon: lucide::{icon}::outlined }} }},"
            ));
            if marked && badge {
                lines.push("badge: rsx! { Indicator { label: 3u32 } },".to_string());
            }
            if marked && disabled {
                lines.push("disabled: true,".to_string());
            }
            lines.push(format!("{label:?}"));
            indent(&format!(
                "NavigationBarItem {{\n{}}}\n",
                indent(&lines.join("\n"))
            ))
        })
        .collect();
    format!("NavigationBar {{\n    \"aria-label\": \"Main\",{body}{items}}}")
}

/// The demo's default: a bar on a phone, a rail from the `sm` breakpoint up.
const RESPONSIVE: &str = "responsive";

/// A tablet-like frame for a rail, which spans its height, or a phone-width one for a bar.
fn frame_sx(rail: bool) -> Sx {
    match rail {
        true => sx()
            .display("flex")
            .flex_direction("row")
            .width("480px")
            .height("360px"),
        false => sx()
            .display("flex")
            .flex_direction("column-reverse")
            .width("360px"),
    }
}

#[component]
pub fn NavigationBarPage() -> Element {
    let theme = use_theme();
    let mut current = use_signal(|| 0usize);

    rsx! {
        DocPage {
            title: "NavigationBar",
            source: "libero/src/components/navigation/navigation_bar.rs",
            markdown: "/md/navigation_bar.md",
            properties: vec![
                props("NavigationBar", vec![
                    prop("variant", "NavigationBarVariant")
                        .default("bar")
                        .doc("`\"bar\"`, a row along the bottom edge for a phone, or `\"rail\"`, an 88px column on the inline-start edge for a tablet or a desktop: the right edge in a right-to-left page. A rail spans the viewport's height and sticks to its top, or docks to the viewport with `position: \"fixed\"`, and publishes its measured width as `--lsx-navigation-rail-width` in place of the bar's height. `\"responsive\"` is a bar below the theme's `sm` breakpoint and a rail from it up, switched by a CSS media query, so the first paint is right without a script; it publishes the bar's height or the rail's width, whichever shows. Keep the same items in the same order at every width."),
                    prop("position", "NavigationBarPosition")
                        .default("static")
                        .doc("`\"static\"` stays in the flow. `\"sticky\"` holds it at the bottom of its scroller, `\"fixed\"` at the viewport's; both publish the bar's measured height as `--lsx-navigation-bar-height` and keep focus clear of the bar. Pad the page's end by `var(--lsx-navigation-bar-height)`. `\"fixed\"` clears a phone's home indicator, and needs `viewport-fit=cover` in the page's viewport meta, or the browser reports no safe area. A native app docks both the same way."),
                    prop("show_labels", "LabelVisibility")
                        .default("always")
                        .doc("`\"always\"`, `\"selected\"` (only the selected item's) or `\"never\"`. A hidden label still names its item."),
                    prop("color", "ThemeAwareValue").default("primary").doc("Colours the selected item's pill. Only the color family counts: the pill is its lightest shade."),
                    prop("z_index", "ThemeAwareValue").default("the header's").doc("Stacking order of a sticky or fixed bar."),
                    prop("parts", "Parts<NavigationBarPart>").doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").default("required").doc("Three to five `NavigationBarItem`s."),
                ])
                .parts("NavigationBarPart", vec![
                    (NavigationBarPart::Item, "Every item."),
                    (NavigationBarPart::Icon, "The pill holding an item's icon and badge."),
                    (NavigationBarPart::Label, "An item's label."),
                ]),
                props("NavigationBarItem", vec![
                    prop("to", "NavigationTarget").doc("A path, a URL or a typed route: the item is a link. Wins over `onclick`."),
                    prop("onclick", "EventHandler<MouseEvent>").doc("Without `to`, the item is a button that calls this."),
                    prop("selected", "bool")
                        .default("follows the route")
                        .doc("Unset, it compares `to` with the current route. Set it for `onclick` items."),
                    prop("icon", "Element").default("required").doc("Drawn in the pill above the label, hidden from screen readers."),
                    prop("badge", "Element").doc("A count or dot over the icon, such as an `Indicator`. Put its count in the item's `aria-label` too."),
                    prop("disabled", "bool").default("false").doc("Dims the item and takes it out of the Tab order."),
                    prop("children", "Element").default("required").doc("The label."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "The bar is a `<nav>` landmark, and each item its own Tab stop, in reading order.",
                    "The selected item carries `aria-current=\"page\"`.",
                    "The icon is hidden from screen readers, so the label names the item. A label hidden by `show_labels` stays in the accessibility tree.",
                    "Every item is at least 56px tall and shares the bar's width equally. A long label wraps to two lines, then ends in an ellipsis.",
                    "A rail keeps its items in the same reading and Tab order, top to bottom. At 200% text it stays 88px wide: labels wrap and the rail scrolls rather than covering the page.",
                    "A sticky or fixed bar pads the page's scroll, so a focused element is not hidden under it.",
                    "In forced colors the selected pill takes the system highlight.",
                    "More than five items logs a warning in debug builds.",
                ])
                .must([
                    "Name the bar with an `aria-label`, such as \"Main\"; a debug build warns without one.",
                    "Put a badge's count in the item's `aria-label`, starting with the visible label: \"Inbox, 3 unread\".",
                    "With `position: \"fixed\"`, pad the page by `var(--lsx-navigation-bar-height)`, or a fixed rail's inline-start by `var(--lsx-navigation-rail-width)`.",
                    "Switch to the rail with the same items in the same order, so the page keeps its landmarks and current item at every width.",
                    "A sticky bar closing a scrolling pane of its own: give the pane `scroll-padding-bottom: var(--lsx-navigation-bar-height)`, so a focused element in it scrolls clear of the bar.",
                ])
                .limits([
                    "A native app has no line clamp: a long label wraps onto more lines and grows the bar.",
                    "A native app's focus scroll ignores that padding, so a link reached with Tab can sit under a sticky bar.",
                ])
                .example("An app bar, `NavigationBar { \"aria-label\": \"Main\" }` with Home, Search and an Inbox item labelled `\"aria-label\": \"Inbox, 3 unread\"`: a screen reader lists the \"Main\" navigation, and the current item reads as the current page."),
            lead: rsx! {
                Text {
                    "A phone's bar of three to five top-level destinations, a "
                    Code { source: "<nav>" }
                    " of icons over labels, that turns into a rail down the side from a tablet up. The selected item gets "
                    Code { source: "aria-current" }
                    " and a pill in a light tint of its "
                    Code { source: "color" }
                    ". Left unset, "
                    Code { source: "selected" }
                    " compares "
                    Code { source: "to" }
                    " with the current route, as in the code below; the preview has no "
                    "routes, so its items select on click instead."
                }
            },
            Demo {
                component: "NavigationBar",
                children_text: "",
                controls: vec![
                    Control::toggle("variant", [RESPONSIVE, "bar", "rail"])
                        .labels(["Responsive", "Bar", "Rail"])
                        .default(RESPONSIVE)
                        .code(|_, values| match values.str("variant").as_str() {
                            RESPONSIVE => vec!["variant: \"responsive\"".to_string()],
                            "rail" => vec!["variant: \"rail\"".to_string()],
                            _ => vec![],
                        }),
                    Control::toggle("show_labels", ["always", "selected", "never"])
                        .labels(["Always", "Selected", "Never"]),
                    Control::color("color").default(theme.navigation_bar.color.as_str()),
                    Control::switch("badge").code(|_, _| vec![]),
                    Control::switch("disabled").code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| {
                    let show_labels = values.str("show_labels");
                    let color = values.str("color");
                    let badge = values.str("badge") == "true";
                    let disabled = values.str("disabled") == "true";
                    let variant = values.str("variant");
                    let wide = format!("(min-width: {})", Size::Sm.breakpoint_value());
                    // The frame follows the bar's own query, so neither waits for a script.
                    let (frame, nav) = match variant.as_str() {
                        RESPONSIVE => (
                            frame_sx(false).media(wide.clone(), frame_sx(true)),
                            sx().media(wide, sx().height("100%")),
                        ),
                        "rail" => (frame_sx(true), sx().height("100%")),
                        _ => (frame_sx(false), sx()),
                    };
                    let frame = frame
                        .max_width("100%")
                        .border("1px solid")
                        .border_color("muted.4")
                        .border_radius("md")
                        .overflow("hidden");
                    rsx! {
                        // A frame, not the viewport: a fixed bar would leave the page.
                        Box { sx: frame,
                            NavigationBar {
                                "aria-label": "Main",
                                variant: variant.as_str(),
                                sx: nav,
                                show_labels: show_labels.as_str(),
                                color,
                                for (index, (label, _, _, icon)) in ITEMS.into_iter().enumerate() {
                                    NavigationBarItem {
                                        key: "{label}",
                                        selected: current() == index,
                                        onclick: move |_| current.set(index),
                                        "aria-label": (index == MARKED && badge).then(|| format!("{label}, 3 unread")),
                                        icon: rsx! { Pictogram { icon } },
                                        badge: (index == MARKED && badge).then(|| rsx! { Indicator { label: 3u32 } }),
                                        disabled: index == MARKED && disabled,
                                        "{label}"
                                    }
                                }
                            }
                            Box { sx: sx().padding("md").flex("1").min_height("120px"), "{ITEMS[current()].0}" }
                        }
                    }
                },
                wrap: Wrap(wrap_bar),
            }
        }
    }
}
