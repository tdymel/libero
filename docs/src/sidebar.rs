use dioxus::prelude::*;
use libero::{
    components::{Anchor, Drawer, Flex, Icon, List, ListItem, Title},
    sx::sx,
};

use crate::Route;
use crate::icons::ChevronIcon;

#[component]
fn NavLink(to: NavigationTarget, children: Element) -> Element {
    rsx! {
        ListItem {
            Anchor {
                to,
                underline: "never",
                sx: sx().display("flex").align_items("center").gap("6px").font_weight("600"),
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
            Title { variant: "h3", component: "p", sx: sx().color("grey.6"), {title} }
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
            Flex {
                direction: "column",
                gap: "20px",
                List {
                    NavLink { to: NavigationTarget::from(Route::GettingStarted {}), "Getting Started" }
                }
                NavGroup { title: "A11y",
                    NavLink { to: NavigationTarget::from(Route::FocusTrapPage {}), "Focus Trap" }
                    NavLink { to: NavigationTarget::from(Route::VisuallyHiddenPage {}), "Visually Hidden" }
                }
                NavGroup { title: "Data Display",
                    NavLink { to: NavigationTarget::from(Route::IconPage {}), "Icon" }
                    NavLink { to: NavigationTarget::from(Route::ImagePage {}), "Image" }
                    NavLink { to: NavigationTarget::from(Route::ListPage {}), "List" }
                    NavLink { to: NavigationTarget::from(Route::QrCodePage {}), "QrCode" }
                }
                NavGroup { title: "Inputs",
                    NavLink { to: NavigationTarget::from(Route::ActionIconPage {}), "ActionIcon" }
                    NavLink { to: NavigationTarget::from(Route::ButtonPage {}), "Button" }
                    NavLink { to: NavigationTarget::from(Route::SelectPage {}), "Select" }
                }
                NavGroup { title: "Layout",
                    NavLink { to: NavigationTarget::from(Route::BoxPage {}), "Box" }
                    NavLink { to: NavigationTarget::from(Route::ContainerPage {}), "Container" }
                    NavLink { to: NavigationTarget::from(Route::DividerPage {}), "Divider" }
                    NavLink { to: NavigationTarget::from(Route::FlexPage {}), "Flex" }
                    NavLink { to: NavigationTarget::from(Route::HeaderPage {}), "Header" }
                }
                NavGroup { title: "Navigation",
                    NavLink { to: NavigationTarget::from(Route::AnchorPage {}), "Anchor" }
                }
                NavGroup { title: "Overlay",
                    NavLink { to: NavigationTarget::from(Route::DialogPage {}), "Dialog" }
                    NavLink { to: NavigationTarget::from(Route::DrawerPage {}), "Drawer" }
                    NavLink { to: NavigationTarget::from(Route::ModalPage {}), "Modal" }
                    NavLink { to: NavigationTarget::from(Route::OverlayPage {}), "Overlay" }
                }
                NavGroup { title: "Typography",
                    NavLink { to: NavigationTarget::from(Route::CodePage {}), "Code" }
                    NavLink { to: NavigationTarget::from(Route::MarkPage {}), "Mark" }
                    NavLink { to: NavigationTarget::from(Route::TextPage {}), "Text" }
                    NavLink { to: NavigationTarget::from(Route::TitlePage {}), "Title" }
                }
            }
        }
    }
}
