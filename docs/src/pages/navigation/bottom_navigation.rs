use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        BottomNavigation, BottomNavigationItem, BottomNavigationPart, Box, Code, Indicator,
        Pictogram, SvgData, Text,
    },
    sx::sx,
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
        .strip_prefix("BottomNavigation {")
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
                "BottomNavigationItem {{\n{}}}\n",
                indent(&lines.join("\n"))
            ))
        })
        .collect();
    format!("BottomNavigation {{\n    \"aria-label\": \"Main\",{body}{items}}}")
}

#[component]
pub fn BottomNavigationPage() -> Element {
    let theme = use_theme();
    let mut current = use_signal(|| 0usize);

    rsx! {
        DocPage {
            title: "BottomNavigation",
            source: "libero/src/components/navigation/bottom_navigation.rs",
            markdown: "/md/bottom_navigation.md",
            properties: vec![
                props("BottomNavigation", vec![
                    prop("position", "BottomNavigationPosition")
                        .default("static")
                        .doc("`\"static\"` stays in the flow. `\"sticky\"` holds it at the bottom of its scroller, `\"fixed\"` at the viewport's; both publish the bar's measured height as `--lsx-bottom-navigation-height` and keep focus clear of the bar. In a native app `fixed` sits at the page's end for now; use `sticky` on the page's last child."),
                    prop("show_labels", "LabelVisibility")
                        .default("always")
                        .doc("`\"always\"`, `\"selected\"` (only the selected item's) or `\"never\"`. A hidden label still names its item."),
                    prop("color", "ThemeAwareValue").default("primary").doc("Colours the selected item's pill. Only the color family counts: the pill is its lightest shade."),
                    prop("z_index", "ThemeAwareValue").default("the header's").doc("Stacking order of a sticky or fixed bar."),
                    prop("parts", "Parts<BottomNavigationPart>").doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").default("required").doc("Three to five `BottomNavigationItem`s."),
                ])
                .parts("BottomNavigationPart", vec![
                    (BottomNavigationPart::Item, "Every item."),
                    (BottomNavigationPart::Icon, "The pill holding an item's icon and badge."),
                    (BottomNavigationPart::Label, "An item's label."),
                ]),
                props("BottomNavigationItem", vec![
                    prop("to", "NavigationTarget").doc("A path, a URL or a typed route: the item is a link. Wins over `onclick`."),
                    prop("onclick", "EventHandler<MouseEvent>").doc("Without `to`, the item is a button that calls this."),
                    prop("selected", "bool")
                        .default("follows the route")
                        .doc("Unset, it compares `to` with the current route. Set it for `onclick` items."),
                    prop("icon", "Element").default("required").doc("Drawn in the pill above the label, hidden from screen readers."),
                    prop("badge", "Element").doc("A count or dot over the icon, such as an `Indicator`."),
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
                    "A sticky or fixed bar pads the page's scroll, so a focused element is not hidden under it.",
                    "In forced colors the selected pill takes the system highlight.",
                    "More than five items logs a warning in debug builds.",
                ])
                .must([
                    "Name the bar with an `aria-label`, such as \"Main\"; a debug build warns without one.",
                    "Put a badge's count in the item's `aria-label`, starting with the visible label: \"Inbox, 3 unread\".",
                    "With `position: \"fixed\"`, pad the page by `var(--lsx-bottom-navigation-height)`.",
                ])
                .limits([
                    "A native app has no line clamp: a long label wraps onto more lines and grows the bar.",
                    "A native app's focus scroll ignores that padding, so a link reached with Tab can sit under a sticky bar.",
                ]),
            lead: rsx! {
                Text {
                    "A phone's bar of three to five top-level destinations, a "
                    Code { source: "<nav>" }
                    " of icons over labels. The selected item gets "
                    Code { source: "aria-current" }
                    " and a pill in a light tint of its "
                    Code { source: "color" }
                    ". Left unset, "
                    Code { source: "selected" }
                    " compares "
                    Code { source: "to" }
                    " with the current route; here the items select on click."
                }
            },
            Demo {
                component: "BottomNavigation",
                children_text: "",
                controls: vec![
                    Control::toggle("show_labels", ["always", "selected", "never"])
                        .labels(["Always", "Selected", "Never"]),
                    Control::color("color").default(theme.bottom_navigation.color.as_str()),
                    Control::switch("badge").code(|_, _| vec![]),
                    Control::switch("disabled").code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| {
                    let show_labels = values.str("show_labels");
                    let color = values.str("color");
                    let badge = values.str("badge") == "true";
                    let disabled = values.str("disabled") == "true";
                    rsx! {
                        // A phone-width frame: a fixed bar would leave the page.
                        Box {
                            sx: sx()
                                .width("360px")
                                .max_width("100%")
                                .border("1px solid")
                                .border_color("muted.4")
                                .border_radius("md")
                                .overflow("hidden"),
                            Box { sx: sx().padding("md").min_height("120px"), "{ITEMS[current()].0}" }
                            BottomNavigation {
                                "aria-label": "Main",
                                show_labels: show_labels.as_str(),
                                color,
                                for (index, (label, _, _, icon)) in ITEMS.into_iter().enumerate() {
                                    BottomNavigationItem {
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
                        }
                    }
                },
                wrap: Wrap(wrap_bar),
            }
        }
    }
}
