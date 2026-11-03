//! Responsive axis props under Blitz (todo 5643): every copy is a `<style>` here, the one
//! added after the first paint too, and media queries follow the window.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{Badge, Button, Flex},
    theme::{Size, Variant, responsive},
};

fn app() -> Element {
    let mut late = use_signal(|| false);
    rsx! {
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

/// The buttons' heights, then the hidden button's and the badge's `display`, blockified by the `Flex`.
fn look(page: &Page) -> (f64, f64, String, String) {
    (
        page.rect("#rsp-button").3,
        page.rect("#rsp-late").3,
        page.computed("#rsp-hidden", "display"),
        page.computed("#rsp-badge", "display"),
    )
}

#[test]
fn responsive_props_follow_the_window_width() {
    let mut page = mount(app);
    page.resize(1280, 800);
    page.click("#rsp-add");
    assert!(page.wait_for(|page| page.exists("#rsp-late")));
    assert_eq!(look(&page), (60.0, 50.0, "none".into(), "flex".into()));

    page.resize(600, 800);
    assert_eq!(look(&page), (30.0, 24.0, "flex".into(), "none".into()));
}
