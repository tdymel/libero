//! `use_presence`'s first render, the one a server sends. A single pass: a
//! second one would let the mount effect hide a wrong initial value.

use std::cell::Cell;
use std::thread;
use std::time::{Duration, Instant};

use super::polling_tests::{elapse, started};
use crate::hooks::use_presence;
use crate::platform::manual_timer;
use dioxus::prelude::*;

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

    // Both: mounted-but-not-visible is the closed frame the server used to send.
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

thread_local! {
    static RENDERS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Renders of `app` once its mount effects ran.
fn renders_after_mount(app: fn() -> Element) -> u32 {
    RENDERS.with(|r| r.set(0));
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    RENDERS.with(|r| r.get())
}

/// The mount effect writes no value already there: an equal write still re-rendered,
/// so every closed `Collapse` drew twice on mount (todo 2031).
#[test]
fn a_settled_presence_renders_once_on_mount() {
    fn closed() -> Element {
        RENDERS.with(|r| r.set(r.get() + 1));
        let presence = use_presence(false, "grid-template-rows", None);
        rsx! { div { "data-visible": presence.visible() } }
    }
    fn open() -> Element {
        RENDERS.with(|r| r.set(r.get() + 1));
        let presence = use_presence(true, "grid-template-rows", None);
        rsx! { div { "data-visible": presence.visible() } }
    }

    assert_eq!(renders_after_mount(closed), 1);
    assert_eq!(renders_after_mount(open), 1);
}

/// A caller with a duration gets the exit fallback (todo 321). No `transitionend`
/// fires here, as on a suppressed exit.
#[test]
fn a_caller_with_a_duration_unmounts_on_the_fallback() {
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
    assert!(
        start.elapsed() >= Duration::from_millis(30 + 150),
        "{:?}",
        start.elapsed()
    );
}

thread_local! {
    static MOUNTS: Cell<usize> = const { Cell::new(0) };
}

#[component]
fn Counted() -> Element {
    use_hook(|| MOUNTS.with(|mounts| mounts.set(mounts.get() + 1)));
    rsx! { "faded body" }
}

/// Reopening mid-exit drops the fallback. A stale one unmounts the open panel and the
/// effect mounts it again at once, so only the mount count shows the content was rebuilt.
#[test]
fn reopening_before_the_fallback_keeps_the_content() {
    fn app() -> Element {
        let open = use_context_provider(|| Signal::new(true));
        let presence = use_presence(open(), "opacity", Some(Duration::from_millis(30)));
        rsx! {
            if presence.mounted() {
                Counted {}
            }
        }
    }

    let _clock = manual_timer::install();
    MOUNTS.with(|mounts| mounts.set(0));
    let mut dom = started(app);
    let mut open = dom.in_scope(ScopeId::APP, consume_context::<Signal<bool>>);

    dom.in_runtime(|| open.set(false));
    elapse(&mut dom, 100);
    assert!(dioxus_ssr::render(&dom).contains("faded body"));
    dom.in_runtime(|| open.set(true));
    elapse(&mut dom, 200);

    assert!(dioxus_ssr::render(&dom).contains("faded body"));
    assert_eq!(MOUNTS.with(Cell::get), 1, "the content was remounted");
}
