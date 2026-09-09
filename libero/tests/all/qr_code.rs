use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::QrCode};

#[test]
fn qr_code_renders_an_svg_labelled_for_assistive_technology() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                QrCode { data: "https://example.com", aria_label: "Scan me" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "div")["aria-label"], "Scan me");
}

/// 2000 bytes fits version 40 at `Low` (2953) but not at `High` (1273).
#[test]
fn qr_code_renders_nothing_for_data_its_robustness_cannot_encode() {
    fn app_at(robustness: &'static str) -> Element {
        let data = "a".repeat(2000);
        rsx! {
            LiberoProvider {
                QrCode { data, robustness, aria_label: "Scan me" }
            }
        }
    }
    fn low() -> Element {
        app_at("low")
    }
    fn high() -> Element {
        app_at("high")
    }

    let fits = body(&render(low));
    assert!(fits.contains("<svg"), "positive control: {fits}");

    let overflows = body(&render(high));
    assert!(!overflows.contains("<svg"), "{overflows}");
    assert!(!overflows.contains("role=\"img\""), "{overflows}");
}
