//! Responsive axis props under Blitz (todo 5643): every copy is a `<style>` here, the one
//! added after the first paint too, and media queries follow the window.

use dioxus::prelude::*;
use e2e::native::{Key, Page, mount};
use libero::{
    components::{Badge, Button, Flex, Options, Tabs},
    theme::{Size, TabsPlacement, Variant, responsive},
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

#[derive(Clone, Copy, PartialEq, Options)]
enum Section {
    One,
    Two,
    Three,
}

fn tabs_app() -> Element {
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
    }
}

/// The strip's `aria-orientation`, its flex direction and the selected tab.
fn strip(page: &Page) -> (String, String, String) {
    let selected = (0..3)
        .map(|index| format!("#rsp-tabs-tab-{index}"))
        .find(|tab| page.attr(tab, "aria-selected").as_deref() == Some("true"))
        .unwrap_or_default();
    (
        page.attr("#rsp-tabs-tablist", "aria-orientation")
            .unwrap_or_default(),
        page.computed("#rsp-tabs-tablist", "flex-direction"),
        selected,
    )
}

fn strip_is(page: &mut Page, want: (&str, &str, &str)) {
    let reached = page.wait_for(|page| {
        let now = strip(page);
        (now.0.as_str(), now.1.as_str(), now.2.as_str()) == want
    });
    assert!(reached, "{:?}, want {want:?}", strip(page));
}

/// Todo 5653: the breakpoint arrives after the first layout here; keys follow the column.
#[test]
fn a_responsive_tabs_placement_follows_the_window() {
    let mut page = mount(tabs_app);
    page.resize(1280, 800);
    strip_is(&mut page, ("vertical", "column", "#rsp-tabs-tab-0"));
    page.click("#rsp-tabs-tab-0");
    page.press(Key::Named(NamedKey::ArrowDown));
    strip_is(&mut page, ("vertical", "column", "#rsp-tabs-tab-1"));

    page.resize(600, 800);
    strip_is(&mut page, ("horizontal", "row", "#rsp-tabs-tab-1"));
    page.press(Key::Named(NamedKey::ArrowRight));
    strip_is(&mut page, ("horizontal", "row", "#rsp-tabs-tab-2"));
}
