use crate::common::render;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    hooks::{DrawerOptions, ModalScope, use_drawer},
    theme::{DrawerDefaults, Size, Theme},
};

#[test]
fn a_drawer_renders_through_the_portal_outlet() {
    #[component]
    fn Opener() -> Element {
        let options = DrawerOptions {
            anchor: "end".into(),
            aria_label: Some("Navigation".into()),
            ..Default::default()
        };
        let nav = use_drawer(options, |_: ModalScope<()>| rsx! { "drawer content" });
        use_hook(move || nav.open());

        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { Opener {} }
        }
    }

    let html = render(app);

    assert_eq!(html.matches("drawer content").count(), 1);
    assert!(
        html.contains(r#"data-state="size-md anchor-end""#),
        "got {html}"
    );
    assert!(html.contains(r#"aria-label="Navigation""#), "got {html}");
}

static WIDE_DRAWERS: Theme = Theme {
    drawer: DrawerDefaults {
        size: Size::Lg,
        ..Theme::DEFAULT.drawer
    },
    ..Theme::DEFAULT
};

#[test]
fn an_unsized_drawer_takes_the_themes_default_size() {
    #[component]
    fn Opener() -> Element {
        let options = DrawerOptions {
            anchor: "end".into(),
            aria_label: Some("Navigation".into()),
            ..Default::default()
        };
        let nav = use_drawer(options, |_: ModalScope<()>| rsx! { "drawer content" });
        use_hook(move || nav.open());

        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { themes: &WIDE_DRAWERS, Opener {} }
        }
    }

    let html = render(app);

    assert!(
        html.contains(r#"data-state="size-lg anchor-end""#),
        "got {html}"
    );
}
