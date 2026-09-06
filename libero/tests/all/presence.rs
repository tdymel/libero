//! `use_presence`'s first render, which is the one a server sends.
//!
//! Deliberately a *single* pass - `common::render` runs two, and the second is
//! exactly where the mount effect would paper over a wrong initial value. What
//! is asserted here is the markup a hydrating client is handed before any
//! effect of its own has run.

use std::thread;
use std::time::{Duration, Instant};

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
        let presence = use_presence(true, "grid-template-rows", None);
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
        let presence = use_presence(false, "grid-template-rows", None);
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

/// A caller outside the crate gets the exit fallback too (todo 321). Nothing
/// here fires `transitionend`, which is the position of a suppressed exit:
/// with `None` the element stayed mounted for good.
#[test]
fn a_public_caller_with_a_duration_unmounts_on_the_fallback() {
    fn app() -> Element {
        let open = use_context_provider(|| Signal::new(true));
        let presence = use_presence(open(), "opacity", Some(Duration::from_millis(30)));
        rsx! {
            if presence.mounted() {
                div { "faded body" }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let mut open = dom.in_scope(ScopeId::APP, consume_context::<Signal<bool>>);
    dom.in_runtime(|| open.set(false));
    let start = Instant::now();

    let html = loop {
        // `process_events` drains the task the timer delivers through.
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        if !html.contains("faded body") || start.elapsed() > Duration::from_secs(2) {
            break html;
        }
        thread::sleep(Duration::from_millis(2));
    };

    assert!(!html.contains("faded body"), "never unmounted: {html}");
    // Not before the duration plus the slack.
    assert!(
        start.elapsed() >= Duration::from_millis(30 + 150),
        "{:?}",
        start.elapsed()
    );
}
