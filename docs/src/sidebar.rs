use dioxus::prelude::*;
use libero::{
    components::{Drawer, Flex, Icon, List, ListItem, NavLink, Title},
    sx::{Sx, sx},
    theme::{Size, SizeCss},
};

use crate::Route;
use crate::icons::ChevronIcon;

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

// `NavLink`'s own base already sets display:flex/align-items:center - only
// the gap/weight this sidebar wants on top needs restating here.
#[component]
fn SidebarNavLink(to: NavigationTarget, children: Element) -> Element {
    rsx! {
        ListItem {
            NavLink {
                to,
                scroll_into_view: true,
                sx: sx().gap("6px").font_weight("300"),
                Icon { variant: "transparent", size: "xs", color: "primary", ChevronIcon {} }
                {children}
            }
        }
    }
}

#[component]
fn NavGroup(title: &'static str, children: Element) -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "8px",
            Title { variant: "h3", size: "h4", component: "p", {title} }
            List { {children} }
        }
    }
}

#[component]
pub fn Sidebar(mut open: Signal<bool>) -> Element {
    rsx! {
        Drawer {
            variant: "static",
            anchor: "left",
            role: "navigation",
            sx: sidebar_responsive_sx(open()),
            Flex {
                direction: "column",
                gap: "20px",
                // Closes on any click inside - good enough for "tap a link,
                // the panel closes" without threading a callback through
                // `NavLink` (which drops `onclick` in its link-navigating
                // mode, same as `Button`). Only matters below `Sm`; at
                // desktop widths `open` never becomes true in the first
                // place, since the toggle that sets it is hidden there.
                onclick: move |_| open.set(false),
                List {
                    SidebarNavLink { to: NavigationTarget::from(Route::GettingStarted {}), "Getting Started" }
                }
                NavGroup { title: "A11y",
                    SidebarNavLink { to: NavigationTarget::from(Route::FocusTrapPage {}), "Focus Trap" }
                    SidebarNavLink { to: NavigationTarget::from(Route::VisuallyHiddenPage {}), "Visually Hidden" }
                }
                NavGroup { title: "Data Display",
                    SidebarNavLink { to: NavigationTarget::from(Route::IconPage {}), "Icon" }
                    SidebarNavLink { to: NavigationTarget::from(Route::ImagePage {}), "Image" }
                    SidebarNavLink { to: NavigationTarget::from(Route::ListPage {}), "List" }
                    SidebarNavLink { to: NavigationTarget::from(Route::QrCodePage {}), "QrCode" }
                }
                NavGroup { title: "Inputs",
                    SidebarNavLink { to: NavigationTarget::from(Route::ActionIconPage {}), "ActionIcon" }
                    SidebarNavLink { to: NavigationTarget::from(Route::ButtonPage {}), "Button" }
                    SidebarNavLink { to: NavigationTarget::from(Route::SelectPage {}), "Select" }
                }
                NavGroup { title: "Layout",
                    SidebarNavLink { to: NavigationTarget::from(Route::BoxPage {}), "Box" }
                    SidebarNavLink { to: NavigationTarget::from(Route::ContainerPage {}), "Container" }
                    SidebarNavLink { to: NavigationTarget::from(Route::DividerPage {}), "Divider" }
                    SidebarNavLink { to: NavigationTarget::from(Route::FlexPage {}), "Flex" }
                    SidebarNavLink { to: NavigationTarget::from(Route::HeaderPage {}), "Header" }
                }
                NavGroup { title: "Navigation",
                    SidebarNavLink { to: NavigationTarget::from(Route::AnchorPage {}), "Anchor" }
                    SidebarNavLink { to: NavigationTarget::from(Route::NavLinkPage {}), "NavLink" }
                }
                NavGroup { title: "Overlay",
                    SidebarNavLink { to: NavigationTarget::from(Route::DialogPage {}), "Dialog" }
                    SidebarNavLink { to: NavigationTarget::from(Route::DrawerPage {}), "Drawer" }
                    SidebarNavLink { to: NavigationTarget::from(Route::ModalPage {}), "Modal" }
                    SidebarNavLink { to: NavigationTarget::from(Route::OverlayPage {}), "Overlay" }
                }
                NavGroup { title: "Typography",
                    SidebarNavLink { to: NavigationTarget::from(Route::CodePage {}), "Code" }
                    SidebarNavLink { to: NavigationTarget::from(Route::KbdPage {}), "Kbd" }
                    SidebarNavLink { to: NavigationTarget::from(Route::MarkPage {}), "Mark" }
                    SidebarNavLink { to: NavigationTarget::from(Route::TextPage {}), "Text" }
                    SidebarNavLink { to: NavigationTarget::from(Route::TitlePage {}), "Title" }
                }
            }
        }
    }
}
