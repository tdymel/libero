//! `use_presence`'s first render, which is the one a server sends.
//!
//! Deliberately a *single* pass - `common::render` runs two, and the second is
//! exactly where the mount effect would paper over a wrong initial value. What
//! is asserted here is the markup a hydrating client is handed before any
//! effect of its own has run.

use dioxus::prelude::*;
use libero::hooks::use_presence;

fn render_once(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn an_initially_open_presence_renders_open() {
    fn app() -> Element {
        let presence = use_presence(true, "grid-template-rows");
        rsx! {
            div {
                "data-mounted": if presence.mounted() { "yes" } else { "no" },
                "data-visible": if presence.visible() { "yes" } else { "no" },
            }
        }
    }

    let html = render_once(app);

    // Both, not just `mounted`: mounted-but-not-visible is the closed frame,
    // and it is what the server used to send for an open element.
    assert!(html.contains(r#"data-mounted="yes""#), "{html}");
    assert!(html.contains(r#"data-visible="yes""#), "{html}");
}

#[test]
fn an_initially_closed_presence_renders_neither_mounted_nor_visible() {
    fn app() -> Element {
        let presence = use_presence(false, "grid-template-rows");
        rsx! {
            div {
                "data-mounted": if presence.mounted() { "yes" } else { "no" },
                "data-visible": if presence.visible() { "yes" } else { "no" },
            }
        }
    }

    let html = render_once(app);

    assert!(html.contains(r#"data-mounted="no""#), "{html}");
    assert!(html.contains(r#"data-visible="no""#), "{html}");
}
