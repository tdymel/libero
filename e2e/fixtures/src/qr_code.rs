//! `QrCode`.

use dioxus::prelude::*;
use libero::components::QrCode;

use crate::Routes;

pub const ROUTES: Routes = &[("/qr-code", || rsx! { QrCodePage {} })];

#[component]
fn QrCodePage() -> Element {
    rsx! {
        div { width: "160px",
            QrCode {
                id: "qr",
                data: "https://example.com",
                aria_label: "QR code linking to example.com",
            }
        }
    }
}
