use std::collections::HashSet;

use dioxus::prelude::*;
use libero::{
    components::{Flex, NavLink, Sidebar, Tree, TreeNode, TreeNodeRenderArgs, default_tree_render},
    hooks::{
        ElementHandle, SwipeDirection, SwipeEvent, SwipeOptions, use_back, use_direction, use_swipe,
    },
    platform::ElementApi,
    sx::{Sx, sx},
    theme::{SIDEBAR_SIZE, Size},
};

use crate::Route;

mod aliases;
mod drag;
mod pager;
mod search;
mod tree;

pub(crate) use drag::use_nav_drag;
pub use pager::{neighbours, page_label};
pub use search::page_actions;
#[cfg(test)]
pub use tree::page_title;
use tree::{NavEntry, nav_tree};

// Below `Sm` an off-canvas panel toggled by `open` (`visibility` drops closed links from tab
// order); from `Sm` up the sticky sidebar, ignoring `open`. With `drawer` off-canvas everywhere.
fn nav_responsive_sx(open: bool, drawer: bool) -> Sx {
    // Only closing delays `visibility`, so the panel slides away instead of vanishing.
    let transition = if open {
        "transform 200ms ease, visibility 0s"
    } else {
        "transform 200ms ease, visibility 0s 200ms"
    };

    // "ColorSchemeButton", the longest label, needs 4px past `Sm` to stay on one row.
    let width = format!("calc({} + 8px)", SIDEBAR_SIZE.value(Size::Sm));
    // Absolute in the row below the header, not fixed at the header's height:
    // Blitz lays a fixed box out like an absolute one, which put it a header lower.
    let base = sx()
        .position("absolute")
        .top("0")
        .height("100%")
        .width("100%")
        // Overlapping the content, so it needs its own background.
        .background("surface")
        // `auto` would paint below any positioned content; well under `Modal`'s 1000.
        .z_index("10")
        .transform(if open {
            "translateX(0)"
        } else {
            "translateX(-100%)"
        })
        // Out past the start edge under RTL too: off the left it overflowed into the
        // scrollable side, and a phone widened the page to show it.
        .selector(
            ":where([dir=rtl]) &",
            sx().transform(if open {
                "translateX(0)"
            } else {
                "translateX(100%)"
            }),
        )
        .visibility(if open { "visible" } else { "hidden" })
        // A sideways swipe reaches the close swipe, also on the inner scroll area.
        .selector("&, & *", sx().touch_action("pan-y pinch-zoom"))
        .transition(transition)
        .media("(prefers-reduced-motion: reduce)", sx().transition("none"));
    if drawer {
        return base.breakpoint(Size::Sm, sx().width(width));
    }
    base.breakpoint(
        Size::Sm,
        sx().position("sticky")
            .top("0")
            .height("100%")
            .width(width)
            .transform("none")
            .selector(":where([dir=rtl]) &", sx().transform("none"))
            .visibility("visible"),
    )
}

/// The closed drawer `px` out from its start edge, the right one under RTL.
fn pulled_style(px: f64, rtl: bool) -> String {
    let at = if rtl {
        format!("calc(100% - {px}px)")
    } else {
        format!("calc(-100% + {px}px)")
    };
    format!("transform: translateX({at}); transition: none; visibility: visible;")
}

fn ancestor_group(data: &[TreeNode<NavEntry>], target: &str) -> Option<String> {
    data.iter()
        .find(|node| node.children.iter().any(|child| child.id == target))
        .map(|node| node.id.clone())
}

#[component]
pub fn DocsNav(
    open: Signal<bool>,
    burger: ElementHandle,
    /// An off-canvas drawer at every width, not a sidebar from `Sm` up.
    #[props(default)]
    drawer: bool,
    /// How far an edge swipe pulls the closed drawer out, px: it follows the finger.
    #[props(default)]
    pulled: Option<f64>,
    /// The drag's handle: its tag on the `nav` is how a WebView measures it.
    measure: ElementHandle,
) -> Element {
    let data = nav_tree();
    let rtl = use_direction().is_rtl();
    let current_path = use_route::<Route>().to_string();
    let group = ancestor_group(&data, &current_path);

    // `Tree`'s open sections. A page in a closed section (the search reaches
    // any) opens it; only a new page does, so the reader can close it again.
    let mut expanded = use_signal(|| group.iter().cloned().collect::<HashSet<_>>());
    use_effect(use_reactive!(|group| {
        if let Some(group) = group
            && !expanded.peek().contains(&group)
        {
            expanded.write().insert(group);
        }
    }));

    let mut close = move || {
        if *open.peek() {
            open.set(false);
            let _ = burger.focus();
        }
    };
    // Android's Back, the button or the gesture, closes the drawer as Escape does (2583).
    use_back(open(), use_callback(move |()| close()));
    let swipe = use_swipe(
        use_callback(move |event: SwipeEvent| {
            let outward = if rtl {
                SwipeDirection::Right
            } else {
                SwipeDirection::Left
            };
            if event.direction == outward {
                close();
            }
        }),
        SwipeOptions::default(),
    );

    rsx! {
        // A swipe toward the start edge closes it; events bubble here from the `nav`.
        div {
            display: "contents",
            onpointerdown: move |event| swipe.onpointerdown.call(event),
            onpointermove: move |event| swipe.onpointermove.call(event),
            onpointerup: move |event| swipe.onpointerup.call(event),
            onpointercancel: move |event| swipe.onpointercancel.call(event),
            Sidebar {
                // What the header's `Burger` names in its `aria-controls`.
                id: "docs-nav",
                side: "start",
                component: "nav",
                // Names the landmark and, when it overflows, its scroll area's tab stop.
                aria_label: "Documentation",
                sx: nav_responsive_sx(open(), drawer),
                // Inline, not a class per finger position; the class's transition takes over on release.
                style: pulled.map(|px| pulled_style(px, rtl)),
                attributes: measure.attributes(),
                Flex { direction: "column", gap: "sm",
                    Tree {
                        // Remounts between drawer and column: Blitz kept the
                        // drawer's text layout, one letter per line (838).
                        key: "{drawer}",
                        aria_label: "Documentation pages",
                        size: "xs",
                        // The tree's guide under each section's chevron marks the current page.
                        guides: true,
                        sx: sx().gap("0").selector("& ul", sx().gap("0")),
                        data,
                        expanded: expanded(),
                        onexpandedchange: move |open: HashSet<String>| expanded.set(open),
                        // The tab stop starts on the current page, not "Guides".
                        current: current_path,
                        render_node: move |args: TreeNodeRenderArgs<NavEntry>| {
                            if args.expanded.is_some() {
                                return default_tree_render(args);
                            }
                            // A page link (not a chevron) closes the panel and focuses the burger. `NavLink`
                            // has no `onclick`, so a `display: contents` wrapper catches the bubble.
                            rsx! {
                                div {
                                display: "contents",
                                onclick: move |_| close(),
                                    NavLink {
                                        to: NavigationTarget::Internal(args.id),
                                        // `Tree`'s roving `<li>` is the only tab stop.
                                        tabindex: args.tabindex,
                                        scroll_into_view: true,
                                        // The guide is the one active indicator, so `NavLink`'s start bar goes.
                                        sx: sx()
                                                                                .align_self("stretch")
                                                                                .when("active", sx().with("background-image", "none")),
                                        "{args.data.label}"
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
