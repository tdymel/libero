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
    static CALLER_RENDERS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

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
        CALLER_RENDERS.set(CALLER_RENDERS.get() + 1);
        let count = use_signal(|| 0_u32);
        let shown = count();
        let modal = use_modal(move |_: ModalScope<()>| rsx! { Dialog { "count {shown}" } });
        CALLER.set(Some((modal, count)));
        rsx! {}
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let (modal, mut count) = CALLER.get().expect("the caller rendered");
    let before = CALLER_RENDERS.get();

    dom.in_runtime(|| {
        let _ = modal.open();
    });
    dom.render_immediate(&mut NoOpMutations);
    assert_eq!(CALLER_RENDERS.get(), before, "opening redrew the caller");
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

thread_local! {
    static HANDLE: std::cell::Cell<Option<(ModalHandle<(), bool>, Outcome)>> =
        const { std::cell::Cell::new(None) };
}

/// A task awaiting an opening that a second `open` replaces ends with `None`,
/// as the docs say, instead of waiting forever.
#[test]
fn a_replaced_opening_ends_its_awaiting_task_with_no_result() {
    fn app() -> Element {
        let outcome: Outcome = use_signal(Vec::new);
        rsx! {
            LiberoProvider { Owner { outcome } }
        }
    }

    #[component]
    fn Owner(outcome: Outcome) -> Element {
        let modal = use_modal(|_: ModalScope<(), bool>| rsx! { Dialog { "asking" } });
        use_hook(move || {
            HANDLE.set(Some((modal, outcome)));
            let mut outcome = outcome;
            let first = modal.open();
            dioxus::core::spawn_forever(async move {
                let result = first.await;
                assert_eq!(result, None);
                outcome.write().push("first awaited");
            });
        });
        rsx! {}
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.process_events();
    let (modal, outcome) = HANDLE.get().expect("the app rendered");
    assert!(dom.in_runtime(|| outcome.peek().is_empty()));

    dom.in_runtime(|| {
        modal.open();
    });
    dom.render_immediate(&mut NoOpMutations);
    dom.process_events();

    assert_eq!(dom.in_runtime(|| outcome.peek().clone()), ["first awaited"]);
}

type Kept = CopyValue<Option<ModalScope<String>>>;

thread_local! {
    static KEPT: std::cell::Cell<Option<(ModalHandle<String>, Kept)>> =
        const { std::cell::Cell::new(None) };
}

/// Todo 1963: a scope kept past its opening reads `None`, not a panic or the
/// superseding opening's arguments.
#[test]
fn a_stale_scope_reads_no_args() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Owner {} }
        }
    }

    #[component]
    fn Owner() -> Element {
        let kept: Kept = use_hook(|| CopyValue::new(None));
        let modal = use_modal(move |s: ModalScope<String>| {
            let mut kept = kept;
            if kept.peek().is_none() {
                kept.set(Some(s));
            }
            rsx! { Dialog { "{s.args()}" } }
        });
        use_hook(move || {
            KEPT.set(Some((modal, kept)));
            modal.open_with("first");
        });
        rsx! {}
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.process_events();
    let (modal, kept) = KEPT.get().expect("the app rendered");
    let first = dom.in_runtime(|| kept.peek().expect("the modal drew"));
    assert_eq!(dom.in_runtime(|| first.try_args()), Some("first".into()));

    dom.in_runtime(|| {
        modal.open_with("second");
    });
    dom.render_immediate(&mut NoOpMutations);
    assert!(dioxus_ssr::render(&dom).contains("second"));
    assert_eq!(dom.in_runtime(|| first.try_args()), None, "superseded");

    dom.in_runtime(|| modal.close());
    dom.render_immediate(&mut NoOpMutations);
    assert_eq!(dom.in_runtime(|| first.try_args()), None, "closed");
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {

    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Button, Dialog},
        hooks::{ModalScope, use_modal},
    };

    #[test]
    fn resolving_from_inside_a_modal_settles_its_opening() {
        #[component]
        fn Opener(outcome: Signal<Option<Option<bool>>>) -> Element {
            let modal = use_modal(|s: ModalScope<(), bool>| {
                rsx! {
                    Dialog { title: "Delete?",
                        Button { onclick: move |_| s.resolve(true), "Yes" }
                    }
                }
            });
            use_hook(move || {
                let mut outcome = outcome;
                modal
                    .open()
                    .onresult(move |result| outcome.set(Some(result)));
            });

            rsx! {}
        }

        fn app() -> Element {
            let outcome = use_signal(|| None);

            rsx! {
                LiberoProvider { Opener { outcome } }
                "{outcome:?}"
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        // `use_hook` opens after `use_modal` has already read the empty slot, so
        // the dialog only exists from the second render on.
        dom.render_immediate(&mut find);
        // The last click listener in the dialog, i.e. the confirm button - the
        // close button in the header registered before it.
        let confirm = find.click.expect("registered no click listener");

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), confirm);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Some(Some(true))"), "got {html}");
        assert!(
            !html.contains("Delete?"),
            "the modal should be gone: {html}"
        );
    }

    /// The `.await` path of the same opening: the task parks on its waker before
    /// the click, and only the resolve wakes it.
    #[test]
    fn awaiting_an_opening_yields_what_the_modal_resolved() {
        #[component]
        fn Opener(outcome: Signal<Option<Option<bool>>>) -> Element {
            let modal = use_modal(|s: ModalScope<(), bool>| {
                rsx! {
                    Dialog { title: "Delete?",
                        Button { onclick: move |_| s.resolve(true), "Yes" }
                    }
                }
            });
            use_hook(move || {
                let mut outcome = outcome;
                let opening = modal.open();
                spawn(async move { outcome.set(Some(opening.await)) });
            });

            rsx! {}
        }

        fn app() -> Element {
            let outcome = use_signal(|| None);

            rsx! {
                LiberoProvider { Opener { outcome } }
                "{outcome:?}"
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        dom.render_immediate(&mut find);
        dom.process_events();
        let confirm = find.click.expect("registered no click listener");

        let html = dioxus_ssr::render(&dom);
        assert!(!html.contains("Some("), "settled before the click: {html}");

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), confirm);
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Some(Some(true))"), "got {html}");
        assert!(
            !html.contains("Delete?"),
            "the modal should be gone: {html}"
        );
    }
}
