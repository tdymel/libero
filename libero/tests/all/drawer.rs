use crate::common::render;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    hooks::{DrawerOptions, ModalScope, use_drawer},
};

#[test]
fn a_drawer_renders_through_the_portal_outlet() {
    #[component]
    fn Opener() -> Element {
        let options = DrawerOptions {
            anchor: "right".into(),
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
        html.contains(r#"data-state="size-md anchor-right""#),
        "got {html}"
    );
    assert!(html.contains(r#"aria-label="Navigation""#), "got {html}");
}
