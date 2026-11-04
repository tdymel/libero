//! `DialogBody` and `DialogActions` under Blitz (todo 5634): full screen, the actions sit at the foot.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{Button, Dialog, DialogActions, DialogBody},
    hooks::{ModalScope, use_modal},
};

fn app() -> Element {
    let modal = use_modal(|_: ModalScope<()>| {
        rsx! {
            Dialog { title: "Terms", full_screen: "always",
                DialogBody { "Short." }
                DialogActions { Button { "Accept" } }
            }
        }
    });
    use_hook(move || modal.open());
    rsx! {}
}

fn bottom(page: &Page, selector: &str) -> f64 {
    page.rects(selector)
        .first()
        .map_or(0.0, |rect| rect.1 + rect.3)
}

#[test]
fn the_actions_of_a_full_screen_dialog_sit_at_the_foot() {
    let mut page = mount(app);
    page.resize(375, 700);
    let at_foot = page.wait_for(|page| bottom(page, "[data-slot=actions]") >= 650.0);
    assert!(
        at_foot,
        "actions end at {} of 700",
        bottom(&page, "[data-slot=actions]")
    );
}
