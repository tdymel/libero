//! One label with no break opportunity in each of `Tree` (default row and
//! `TreeItem`), `NavLink` and `Menu`, for reflow at a narrow width (todo 518),
//! and in `Tabs`, `Timeline`, `Menubar` (todo 543).

use dioxus::prelude::*;
use libero::components::{
    Button, Flex, Menu, MenuItem, Menubar, MenubarMenu, NavLink, OptionLabel, Options, Tabs,
    Timeline, TimelineEvent, Tree, TreeItem, TreeNode, TreeNodeRenderArgs, use_menu,
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
                value: Pick::Short,
                onchange: move |_| {},
                option_label: move |pick: Pick| OptionLabel::from(pick.name()),
                panel: |_| rsx! { "Panel" },
            }
            // SegmentedControl and Chip keep Button's fixed height: todo 481.
            Timeline {
                id: "timeline",
                items: vec![
                    TimelineEvent::new(LONG).content(rsx! { "{LONG}" }),
                    TimelineEvent::new("Short"),
                ],
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
