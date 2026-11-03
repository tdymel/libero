//! Responsive axis props and `hidden_from` / `visible_from` (todo 5643), and a responsive
//! behaviour prop, Tabs `placement` (todo 5653).

use dioxus::prelude::*;
use libero::components::{Badge, Button, Flex, Options, Tabs};
use libero::theme::{Size, TabsPlacement, Variant, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive", || rsx! { ResponsivePage {} })];

/// Small and outlined under 62rem, large and filled from it; a late button registers its
/// copies after mount.
#[derive(Clone, Copy, PartialEq, Options)]
enum Section {
    One,
    Two,
    Three,
}

/// A row above the panel under 62rem, a column beside it from there.
#[component]
fn ResponsivePage() -> Element {
    let mut late = use_signal(|| false);
    let mut section = use_signal(|| Section::One);
    rsx! {
        Tabs {
            id: "rsp-tabs",
            aria_label: "Sections",
            placement: responsive(TabsPlacement::Top).md(TabsPlacement::Start),
            value: section(),
            onchange: move |next| section.set(next),
            panel: |section: Section| rsx! { "{section.label()}" },
        }
        Flex { gap: "md", align: "center", wrap: "wrap",
            Button {
                id: "rsp-button",
                size: responsive(Size::Sm).md(Size::Xl),
                variant: responsive(Variant::Outlined).md(Variant::Filled),
                "Responsive"
            }
            Button { id: "rsp-hidden", hidden_from: "md", "Narrow only" }
            Badge { id: "rsp-badge", visible_from: "md", "Wide only" }
            Button { id: "rsp-add", onclick: move |_| late.set(true), "Add" }
            if late() {
                Button { id: "rsp-late", size: responsive(Size::Xs).md(Size::Lg), "Late" }
            }
        }
    }
}
