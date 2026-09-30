//! Unbreakable labels for narrow-width reflow (todos 518, 543) in trees, menus, tabs and lists,
//! and in `Button`, `Chip`, `SegmentedControl`, which keep one line (todo 481).

use dioxus::prelude::*;
use libero::components::{
    Button, Chip, DataList, DataListItem, Flex, Menu, MenuItem, Menubar, MenubarMenu, NavLink,
    OptionLabel, Options, SegmentedControl, Tabs, Timeline, TimelineEvent, Tree, TreeItem,
    TreeNode, TreeNodeRenderArgs, use_menu,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/long-labels", || rsx! { LongLabelsPage {} })];

const LONG: &str = "Versandkostenberechnungsgrundlagenverordnungsentwurfsbearbeitungsstelle";

#[component]
fn LongLabelsPage() -> Element {
    let menu = use_menu();
    let nodes = || {
        vec![
            TreeNode::new("docs", "docs").children(vec![TreeNode::new("docs/long", LONG)]),
            TreeNode::new("long", LONG),
        ]
    };
    let expanded = || ["docs".to_string()].into_iter().collect();

    rsx! {
        Flex { direction: "column", gap: "md",
            Tree {
                id: "tree-default",
                aria_label: "Default rows",
                data: nodes(),
                default_expanded: expanded(),
            }
            Tree {
                id: "tree-item",
                aria_label: "TreeItem rows",
                data: nodes(),
                default_expanded: expanded(),
                render_node: move |args: TreeNodeRenderArgs<&'static str>| rsx! {
                    TreeItem { tabindex: args.tabindex, "{args.data}" }
                },
            }
            nav { id: "nav",
                NavLink { to: "https://example.com", active: true, "{LONG}" }
            }
            Menu {
                state: menu,
                items: vec![MenuItem::new("Short").into(), MenuItem::new(LONG).into()],
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Open" }
            }
            // Todo 543.
            Tabs {
                id: "tabs",
                aria_label: "Picks",
                value: Pick::Short,
                onchange: move |_| {},
                option_label: move |pick: Pick| OptionLabel::from(pick.name()),
                panel: |_| rsx! { "Panel" },
            }
            // Todo 481: one line, cut at the edge (662, 674).
            div { id: "single-line",
                Button { id: "button", "{LONG}" }
                Button { id: "button-full", full_width: true, "{LONG}" }
                Chip { id: "chip", "{LONG}" }
                Chip { id: "chip-filter", checked: false, onchange: move |_| {}, "{LONG}" }
                Chip { id: "chip-icon",
                    icon: rsx! {
                        svg { width: "16", height: "16", view_box: "0 0 24 24",
                            circle { cx: "12", cy: "12", r: "8" }
                        }
                    },
                    "{LONG}"
                }
                // The chip's documented ellipsis: a text span of the caller's own.
                Chip { id: "chip-trailing",
                    trailing: rsx! { button { r#type: "button", aria_label: "Remove", "x" } },
                    span {
                        "data-text": "",
                        style: "min-width: 0; overflow: hidden; text-overflow: ellipsis",
                        "{LONG}"
                    }
                }
                Flex { direction: "row", gap: "sm",
                    Button { id: "button-row", "{LONG}" }
                    Chip { id: "chip-row", "{LONG}" }
                }
                SegmentedControl {
                    id: "segmented",
                    aria_label: "Long",
                    value: Pick::Short,
                    onchange: move |_| {},
                    option_label: move |pick: Pick| OptionLabel::from(pick.name()),
                }
                SegmentedControl {
                    id: "segmented-full",
                    aria_label: "Long, full width",
                    full_width: true,
                    value: Pick::Short,
                    onchange: move |_| {},
                    option_label: move |pick: Pick| OptionLabel::from(pick.name()),
                }
            }
            Timeline {
                id: "timeline",
                items: vec![
                    TimelineEvent::new(LONG).content(rsx! { "{LONG}" }),
                    TimelineEvent::new("Short"),
                ],
            }
            // A multi-word term wider than a phone, and an unbreakable description.
            DataList { id: "data-list", orientation: "horizontal",
                DataListItem {
                    label: rsx! { "Estimated delivery window for international parcels" },
                    "{LONG}"
                }
                DataListItem { label: rsx! { "{LONG}" }, "Short" }
            }
            Menubar {
                id: "menubar",
                aria_label: "Long",
                menus: vec![MenubarMenu::new(LONG, vec![MenuItem::new("Item").into()])],
            }
            Menubar {
                id: "menubar-crowded",
                aria_label: "Crowded",
                menus: ["File", "Edit", "View", "Insert", "Format", "Tools", "Window", "Help"]
                    .map(|name| MenubarMenu::new(name, vec![MenuItem::new("Item").into()]))
                    .to_vec(),
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Pick {
    Short,
    Long,
}

impl Pick {
    fn name(self) -> &'static str {
        match self {
            Pick::Short => "Short",
            Pick::Long => LONG,
        }
    }
}
