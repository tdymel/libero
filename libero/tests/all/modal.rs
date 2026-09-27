use std::sync::atomic::{AtomicUsize, Ordering};

use crate::common::{body, render};

use dioxus::{core::NoOpMutations, prelude::*};
use libero::{
    LiberoProvider,
    components::{Dialog, FloatingWindowOptions},
    hooks::{
        ModalHandle, ModalScope, PopoverOptions, use_element, use_floating_window, use_modal,
        use_popover,
    },
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

/// Todo 527: a Dialog in a popover or a FloatingWindow opened inside a modal
/// is not modal. Guards the portal: content takes context from the outlet.
#[test]
fn a_dialog_in_a_non_modal_surface_inside_a_modal_is_not_modal() {
    #[component]
    fn InPopover() -> Element {
        let anchor = use_element();
        let popover = use_popover(anchor, true, PopoverOptions::new(4.0, 8.0));
        popover.show(Some(rsx! {
            Dialog { title: "In popover", "popover content" }
        }));
        rsx! {}
    }

    #[component]
    fn InWindow() -> Element {
        let window = use_floating_window(
            FloatingWindowOptions {
                title: Some("Window".into()),
                ..Default::default()
            },
            |_| rsx! { Dialog { title: "In window", "window content" } },
        );
        use_hook(move || window.open());
        rsx! {}
    }

    #[component]
    fn Opener() -> Element {
        let modal = use_modal(|_: ModalScope<()>| {
            rsx! {
                Dialog { title: "Outer",
                    InPopover {}
                    InWindow {}
                }
            }
        });
        use_hook(move || modal.open());
        rsx! {}
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider { Opener {} }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert!(body.contains("popover content"), "{body}");
    assert!(body.contains("window content"), "{body}");
    let modal_dialogs = body.matches("aria-modal=\"true\"").count();
    assert_eq!(modal_dialogs, 1, "only the outer dialog is modal: {body}");
    assert_eq!(
        body.matches("aria-label=\"Close\"").count(),
        2,
        "the outer dialog and the window close, the inner dialogs do not: {body}"
    );
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

type Outcome = Signal<Vec<&'static str>>;

thread_local! {
    static OWNER: std::cell::Cell<Option<(Signal<bool>, Outcome)>> =
        const { std::cell::Cell::new(None) };
}

/// Todo 1306: the owner unmounting while open settles the opening as a
/// dismissal, for a handler and an awaiting task alike.
#[test]
fn unmounting_the_owner_settles_an_open_modal() {
    fn app() -> Element {
        let shown = use_signal(|| true);
        let outcome: Outcome = use_signal(Vec::new);
        OWNER.set(Some((shown, outcome)));
        rsx! {
            LiberoProvider {
                if shown() {
                    Owner { outcome }
                }
            }
        }
    }

    #[component]
    fn Owner(outcome: Outcome) -> Element {
        let modal = use_modal(|_: ModalScope<(), bool>| rsx! { Dialog { "asking" } });
        use_hook(move || {
            let mut outcome = outcome;
            let opening = modal.open().onresult(move |result| {
                assert_eq!(result, None);
                outcome.write().push("handler");
            });
            // Outlives this scope, as a task awaiting a confirm usually does.
            dioxus::core::spawn_forever(async move {
                let result = opening.await;
                assert_eq!(result, None);
                outcome.write().push("awaited");
            });
        });
        rsx! {}
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.process_events();
    let (mut shown, outcome) = OWNER.get().expect("the app rendered");
    assert!(dioxus_ssr::render(&dom).contains("asking"));

    dom.in_runtime(|| shown.set(false));
    dom.render_immediate(&mut NoOpMutations);
    dom.process_events();

    let html = dioxus_ssr::render(&dom);
    assert!(!html.contains("asking"), "the modal should be gone: {html}");
    let mut settled = dom.in_runtime(|| outcome.peek().clone());
    settled.sort_unstable();
    assert_eq!(settled, ["awaited", "handler"]);
}
