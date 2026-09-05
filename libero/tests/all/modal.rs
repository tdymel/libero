use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Dialog,
    hooks::{ModalScope, use_modal},
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
