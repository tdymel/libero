//! Responsive overlay props (todo 5657): popover sides, a Dialog full screen under 62rem and a
//! Drawer docked at the bottom under 62rem, at the end from it.

use dioxus::prelude::*;
use libero::components::{Button, Dialog, DrawerAnchor, HoverCard, Tooltip};
use libero::hooks::{Align, DrawerOptions, ModalScope, Side, use_drawer, use_modal};
use libero::theme::responsive;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/responsive-overlay", || rsx! { PopoverPage {} }),
    ("/responsive-overlay/dialog", || rsx! { DialogPage {} }),
    ("/responsive-overlay/drawer", || rsx! { DrawerPage {} }),
];

/// Below `md` under the trigger, from `md` at its end.
const SIDE: libero::theme::Responsive<Side> = responsive(Side::Bottom).md(Side::End);

#[component]
fn PopoverPage() -> Element {
    rsx! {
        div { style: "padding: 40px 120px; display: flex; gap: 300px",
            Tooltip { label: rsx! { "Saves" }, label_id: "rsv-tip-label", id: "rsv-tip", open: true, side: SIDE,
                Button { id: "rsv-tip-trigger", "aria-describedby": "rsv-tip-label", "Save" }
            }
            HoverCard { aria_label: "Profile", id: "rsv-card", open: true, side: SIDE,
                align: responsive(Align::Start).md(Align::Center), content: rsx! { "Ada" },
                Button { id: "rsv-card-trigger", "Ada" }
            }
        }
    }
}

#[component]
fn DialogPage() -> Element {
    let modal = use_modal(|_: ModalScope<()>| {
        rsx! {
            Dialog { id: "rsv-dialog", title: "Filters", full_screen: responsive(true).md(false), "Pick a colour." }
        }
    });
    use_hook(move || modal.open());
    rsx! {}
}

#[component]
fn DrawerPage() -> Element {
    let drawer = use_drawer(
        DrawerOptions {
            anchor: responsive(DrawerAnchor::Bottom).md(DrawerAnchor::End).into(),
            aria_label: Some("Panel".into()),
            ..Default::default()
        },
        |_: ModalScope<()>| rsx! { div { id: "rsv-drawer-content", "Drawer" } },
    );
    use_hook(move || drawer.open());
    rsx! {}
}
