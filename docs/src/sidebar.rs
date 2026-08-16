use dioxus::prelude::*;
use libero::{
    components::{Drawer, Flex, Icon, List, ListItem, NavLink, Title},
    sx::{Sx, sx},
};

use crate::Route;
use crate::icons::ChevronIcon;

// Pinned to the top of the scrolling row (AppShell, not the document, is
// what scrolls) instead of scrolling away with the content - `height: 100%`
// matches the row's own height, so it stays capped and its content still
// scrolls internally (Drawer's own `Static`-variant overflow:auto) rather
// than growing the sidebar past the row.
fn sidebar_sticky_sx() -> Sx {
    sx().position("sticky").top("0").height("100%")
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
pub fn Sidebar() -> Element {
    rsx! {
        Drawer {
            variant: "static",
            anchor: "left",
            size: "sm",
            role: "navigation",
            sx: sidebar_sticky_sx(),
            Flex {
                direction: "column",
                gap: "20px",
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
