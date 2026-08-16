use std::collections::HashSet;

use dioxus::prelude::*;
use libero::{
    components::{Drawer, Flex, Tree, TreeLabel, TreeNode},
    sx::{Sx, sx},
    theme::{Size, SizeCss},
};

use crate::Route;

// One `Drawer`, one responsive `sx` - no second drawer, no viewport
// detection. Below the `Sm` breakpoint it's a fixed, off-canvas panel
// toggled by `open` (slides via `transform`, `visibility` hidden when
// closed so its links drop out of tab order/the a11y tree instead of just
// being invisible); at `Sm` and up it's back to the persistent sticky
// sidebar from before, with `open` irrelevant - the breakpoint override
// hardcodes `transform`/`visibility` regardless of its value, so nothing
// odd happens if the viewport crosses `Sm` while it happens to be open.
fn sidebar_responsive_sx(open: bool) -> Sx {
    let header_height = SizeCss::HEADER_HEIGHT.value(Size::Md);
    // `visibility` shouldn't flip to hidden until the slide-out finishes,
    // or the panel would vanish mid-animation instead of sliding away;
    // opening has no such concern, so only closing gets the delay.
    let transition = if open {
        "transform 200ms ease, visibility 0s"
    } else {
        "transform 200ms ease, visibility 0s 200ms"
    };

    sx().position("fixed")
        .top(header_height.clone())
        .height(format!("calc(100vh - {header_height})"))
        .width("100%")
        // `Drawer`'s own base has no background - fine sitting adjacent to
        // content in normal flow (desktop), but this mode overlaps the main
        // content, which would otherwise show through underneath it.
        .background("white")
        // `position: fixed` alone only creates a stacking context - without
        // an explicit z-index it's `auto`, which paints below anything else
        // on the page that happens to have a real (even low, even `0`)
        // z-index, letting that content's hit-testing win instead. Well
        // under `Modal`'s own range (starts at 1000) so an actual modal
        // still stacks above this.
        .z_index("10")
        .transform(if open {
            "translateX(0)"
        } else {
            "translateX(-100%)"
        })
        .visibility(if open { "visible" } else { "hidden" })
        .transition(transition)
        .breakpoint(
            Size::Sm,
            sx().position("sticky")
                .top("0")
                .height("100%")
                .width(SizeCss::DRAWER_SIZE.value(Size::Sm))
                .transform("none")
                .visibility("visible"),
        )
}

#[derive(Clone, PartialEq)]
struct SidebarEntry {
    label: &'static str,
}

impl TreeLabel for SidebarEntry {
    fn tree_label(&self) -> String {
        self.label.to_string()
    }
}

fn page(route: Route, label: &'static str) -> TreeNode<SidebarEntry> {
    TreeNode::new(route.to_string(), SidebarEntry { label })
}

// A synthetic id (never a real route path, which always starts with `/`) -
// distinguishes a group header from a page in `onselectedchange` without
// needing a separate lookup structure.
fn group(
    id: &'static str,
    label: &'static str,
    children: Vec<TreeNode<SidebarEntry>>,
) -> TreeNode<SidebarEntry> {
    TreeNode::new(format!("group:{id}"), SidebarEntry { label }).children(children)
}

fn sidebar_tree() -> Vec<TreeNode<SidebarEntry>> {
    vec![
        page(Route::GettingStarted {}, "Getting Started"),
        group(
            "a11y",
            "A11y",
            vec![
                page(Route::FocusTrapPage {}, "Focus Trap"),
                page(Route::VisuallyHiddenPage {}, "Visually Hidden"),
            ],
        ),
        group(
            "data-display",
            "Data Display",
            vec![
                page(Route::IconPage {}, "Icon"),
                page(Route::ImagePage {}, "Image"),
                page(Route::ListPage {}, "List"),
                page(Route::TreePage {}, "Tree"),
                page(Route::QrCodePage {}, "QrCode"),
            ],
        ),
        group(
            "inputs",
            "Inputs",
            vec![
                page(Route::ActionIconPage {}, "ActionIcon"),
                page(Route::ButtonPage {}, "Button"),
                page(Route::SelectPage {}, "Select"),
            ],
        ),
        group(
            "layout",
            "Layout",
            vec![
                page(Route::BoxPage {}, "Box"),
                page(Route::ContainerPage {}, "Container"),
                page(Route::DividerPage {}, "Divider"),
                page(Route::FlexPage {}, "Flex"),
                page(Route::HeaderPage {}, "Header"),
            ],
        ),
        group(
            "navigation",
            "Navigation",
            vec![
                page(Route::AnchorPage {}, "Anchor"),
                page(Route::NavLinkPage {}, "NavLink"),
            ],
        ),
        group(
            "overlay",
            "Overlay",
            vec![
                page(Route::DialogPage {}, "Dialog"),
                page(Route::DrawerPage {}, "Drawer"),
                page(Route::ModalPage {}, "Modal"),
                page(Route::OverlayPage {}, "Overlay"),
            ],
        ),
        group(
            "typography",
            "Typography",
            vec![
                page(Route::CodePage {}, "Code"),
                page(Route::KbdPage {}, "Kbd"),
                page(Route::MarkPage {}, "Mark"),
                page(Route::TextPage {}, "Text"),
                page(Route::TitlePage {}, "Title"),
            ],
        ),
    ]
}

fn ancestor_group(data: &[TreeNode<SidebarEntry>], target: &str) -> Option<String> {
    data.iter()
        .find(|node| node.children.iter().any(|child| child.id == target))
        .map(|node| node.id.clone())
}

#[component]
pub fn Sidebar(mut open: Signal<bool>) -> Element {
    let data = sidebar_tree();

    let current_path = try_router()
        .map(|router| router.full_route_string())
        .unwrap_or_default();

    // Seeded once from the initial route, so deep-linking to a page opens
    // its section - after that, purely the user's own expand/collapse
    // clicks, including collapsing the section the active page is in.
    let mut expanded = use_signal(|| {
        let mut set = HashSet::new();
        if let Some(group_id) = ancestor_group(&data, &current_path) {
            set.insert(group_id);
        }
        set
    });

    rsx! {
        Drawer {
            variant: "static",
            anchor: "left",
            role: "navigation",
            sx: sidebar_responsive_sx(open()),
            Flex {
                direction: "column",
                gap: "8px",
                // Closes on any click inside - good enough for "tap a link,
                // the panel closes" without threading a callback through
                // `Tree`. Only matters below `Sm`; at desktop widths `open`
                // never becomes true in the first place, since the toggle
                // that sets it is hidden there.
                onclick: move |_| open.set(false),
                Tree {
                    aria_label: "Documentation pages",
                    size: "xs",
                    data,
                    expanded: expanded(),
                    onexpandedchange: move |next| expanded.set(next),
                    selected: Some(current_path.clone()),
                    onselectedchange: move |id: Option<String>| {
                        if let Some(id) = id
                            && !id.starts_with("group:")
                        {
                            navigator().push(id);
                        }
                    },
                }
            }
        }
    }
}
