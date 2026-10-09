//! `Alert`'s `parts` on Blitz: the part selectors match natively, the instance `sx`
//! wins a tie, and a nested `Alert` is not reached.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Alert, AlertPart, Part, Parts};
use libero::sx::sx;

const TITLE: &str = "#styled > [data-slot=body] > [data-slot=title]";

fn app() -> Element {
    rsx! {
        Alert { id: "styled",
            title: "Styled parts",
            parts: Parts::new()
                .part(AlertPart::Title, sx().font_style("italic").color("#c80000")),
            sx: sx().selector(AlertPart::Title.selector(), sx().color("#0000c8")),
            "Parts styled."
            Alert { id: "nested", title: "Nested", "Not reached." }
        }
    }
}

fn actions_alert(width: u32) -> Element {
    rsx! {
        div { style: "width: {width}px",
            Alert {
                id: "wrap",
                title: "Card expiring",
                onclose: move |_| {},
                actions: rsx! { button { "Update card" } },
                "Your card ends 09/26. Update it before the next invoice."
            }
        }
    }
}

fn narrow_app() -> Element {
    actions_alert(238)
}

fn wide_app() -> Element {
    actions_alert(600)
}

/// Without `flex-wrap` the 94px row left the message about 100px wide (browser, 320px).
#[test]
fn a_narrow_alert_drops_its_actions_under_the_message() {
    let page = mount(narrow_app);
    let (_, body_y, body_w, body_h) = page.rect("#wrap > [data-slot=body]");
    let (_, actions_y, _, _) = page.rect("#wrap > [data-slot=actions]");
    let (close_x, _, close_w, _) = page.rect("#wrap > [data-slot=close]");
    let (alert_x, _, alert_w, _) = page.rect("#wrap");

    assert!(body_w > 0.6 * alert_w, "message {body_w}px of {alert_w}px");
    assert!(
        actions_y >= body_y + body_h,
        "{actions_y} under {body_y}+{body_h}"
    );
    assert!(
        close_x + close_w > alert_x + alert_w - 20.0,
        "close at the end edge"
    );
}

#[test]
fn a_wide_alert_keeps_its_actions_beside_the_message() {
    let page = mount(wide_app);
    let (_, body_y, _, body_h) = page.rect("#wrap > [data-slot=body]");
    let (_, actions_y, _, _) = page.rect("#wrap > [data-slot=actions]");

    assert!(
        actions_y < body_y + body_h,
        "{actions_y} beside {body_y}+{body_h}"
    );
}

#[test]
fn parts_style_the_inner_parts_natively() {
    let page = mount(app);
    assert_eq!(page.computed(TITLE, "font-style"), "italic");
    assert_eq!(
        page.computed(TITLE, "color"),
        "rgb(0, 0, 200)",
        "the instance sx should win the tie"
    );
    assert_eq!(
        page.computed("#nested [data-slot=title]", "font-style"),
        "normal"
    );
}
