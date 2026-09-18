//! The shadow scale on the surfaces that rest on it (813): a card at the
//! `Paper` default, every step, an open `Menu` and a `Dialog`.

use dioxus::prelude::*;
use libero::{
    components::{Button, Dialog, Flex, Menu, MenuItem, Paper, Text, Title, use_menu},
    hooks::{ModalScope, use_modal},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/elevation", || rsx! { ElevationPage {} })];

#[component]
fn ElevationPage() -> Element {
    let menu = use_menu();
    use_hook(|| menu.open());
    let items = vec![
        MenuItem::new("Save").onselect(|_| {}).into(),
        MenuItem::new("Share").onselect(|_| {}).into(),
    ];
    let dialog = use_modal(|s: ModalScope<()>| {
        rsx! {
            Dialog { title: "Unsaved changes", size: "sm",
                Text { "notes.md has changes you have not saved." }
                Button { variant: "filled", onclick: move |_| s.close(), "Discard" }
            }
        }
    });

    rsx! {
        Flex { direction: "column", gap: "xl", sx: sx().padding("24px"),
            Paper { id: "card", sx: sx().padding("lg").max_width("320px"),
                Title { size: "md", component: "h2", "Card" }
                Text { "A Paper at the theme's default elevation." }
            }
            Flex { direction: "row", gap: "xl", wrap: "wrap",
                for size in ["xs", "sm", "md", "lg", "xl", "xxl"] {
                    Paper {
                        key: "{size}",
                        id: "step-{size}",
                        shadow: size,
                        sx: sx().padding("lg").width("96px"),
                        Text { "{size}" }
                    }
                }
            }
            Flex { direction: "row", gap: "xl",
                Menu {
                    state: menu,
                    items,
                    Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
                }
                Button {
                    id: "open-dialog",
                    onclick: move |_| {
                        dialog.open();
                    },
                    "Open dialog"
                }
            }
        }
    }
}
