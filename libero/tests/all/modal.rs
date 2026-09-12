use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common::{body, render};

use dioxus::{core::NoOpMutations, prelude::*};
use libero::{
    LiberoProvider,
    components::Dialog,
    hooks::{ModalHandle, ModalScope, use_modal},
};

#[test]
fn a_modal_renders_through_the_portal_outlet() {
    #[component]
    fn Opener() -> Element {
        let modal = use_modal(|s: ModalScope<String>| {
            rsx! {
                Dialog { title: "{s.args()}", "modal content" }
            }
        });
        use_hook(move || modal.open_with("Titled"));

        rsx! {
            div { id: "in-place" }
        }
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { Opener {} }
        }
    }

    let html = render(app);
    let body = body(&html);
    let in_place = body.find("in-place").expect("the in-place wrapper");
    let content = body.find("modal content").expect("the modal content");

    assert!(
        content > in_place,
        "portalled content should render at the outlet, after where it was written"
    );
    assert!(body.contains("Titled"), "the args reach the render closure");
}

#[test]
fn a_superseded_opening_no_longer_closes_the_live_modal() {
    #[component]
    fn Opener() -> Element {
        let modal = use_modal(|s: ModalScope<String>| rsx! { Dialog { "{s.args()}" } });
        use_hook(move || {
            let first = modal.open_with("first");
            modal.open_with("second");
            first.close();
        });

        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { Opener {} }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("second"), "got {body}");
    assert!(!body.contains("first"), "got {body}");
}

#[test]
fn dismissing_a_modal_settles_its_opening_with_no_result() {
    #[component]
    fn Opener(outcome: Signal<::std::option::Option<::std::option::Option<bool>>>) -> Element {
        let modal = use_modal(|_: ModalScope<(), bool>| rsx! { Dialog { "asking" } });
        use_hook(move || {
            let mut outcome = outcome;
            modal
                .open()
                .onresult(move |result| outcome.set(Some(result)));
            modal.close();
        });

        rsx! {}
    }

    fn app() -> Element {
        let outcome = use_signal(|| ::std::option::Option::None);

        rsx! {
            LiberoProvider { Opener { outcome } }
            "{outcome:?}"
        }
    }

    let body = body(&render(app));

    assert!(body.contains("Some(None)"), "got {body}");
    assert!(!body.contains("asking"), "the modal should be gone: {body}");
}

thread_local! {
    static CALLER: std::cell::Cell<Option<(ModalHandle<()>, Signal<u32>)>> =
        const { std::cell::Cell::new(None) };
}
static CALLER_RENDERS: AtomicUsize = AtomicUsize::new(0);

/// Opening draws the modal in its own scope: the caller is not redrawn, and a
/// caller redraw while open still reaches what `render` captured.
#[test]
fn opening_leaves_the_caller_alone_and_its_redraw_reaches_the_content() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Caller {} }
        }
    }

    #[component]
    fn Caller() -> Element {
        CALLER_RENDERS.fetch_add(1, Ordering::Relaxed);
        let count = use_signal(|| 0_u32);
        let shown = count();
        let modal = use_modal(move |_: ModalScope<()>| rsx! { Dialog { "count {shown}" } });
        CALLER.set(Some((modal, count)));
        rsx! {}
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let (modal, mut count) = CALLER.get().expect("the caller rendered");
    let before = CALLER_RENDERS.load(Ordering::Relaxed);

    dom.in_runtime(|| {
        let _ = modal.open();
    });
    dom.render_immediate(&mut NoOpMutations);
    assert_eq!(
        CALLER_RENDERS.load(Ordering::Relaxed),
        before,
        "opening redrew the caller"
    );
    assert!(dioxus_ssr::render(&dom).contains("count 0"));

    dom.in_runtime(|| count.set(7));
    dom.render_immediate(&mut NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("count 7"), "stale content: {html}");

    dom.in_runtime(|| modal.close());
    dom.render_immediate(&mut NoOpMutations);
    assert!(!dioxus_ssr::render(&dom).contains("count"));
}
