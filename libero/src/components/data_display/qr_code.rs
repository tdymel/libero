use dioxus::prelude::*;
#[cfg(feature = "wasm-split")]
use dioxus::wasm_split;
use fast_qr::convert::{Builder, svg::SvgBuilder};
use fast_qr::{ECL, QRBuilder};

use crate::{
    components::{Box, Input, States, common::base_props},
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::{QR_CODE_BACKGROUND, QR_CODE_FOREGROUND, QrRobustness},
};

impl From<QrRobustness> for ECL {
    fn from(value: QrRobustness) -> Self {
        match value {
            QrRobustness::Low => Self::L,
            QrRobustness::Medium => Self::M,
            QrRobustness::Quartile => Self::Q,
            QrRobustness::High => Self::H,
        }
    }
}

impl From<&str> for Input<QrRobustness> {
    fn from(value: &str) -> Self {
        Input::Value(QrRobustness::from(value))
    }
}

impl From<String> for Input<QrRobustness> {
    fn from(value: String) -> Self {
        Input::Value(QrRobustness::from(value))
    }
}

// Standard 4-module quiet zone (ISO/IEC 18004) - not a style choice, most
// scanners won't reliably read a code without it.
const QR_CODE_MARGIN: usize = 4;

// `fast_qr`'s svg has no width/height attribute (just a square viewBox), so
// it already scales to fill this - `& svg` just makes that fill explicit and
// keeps it block-level instead of leaving inline-svg's baseline gap under it.
static QR_CODE_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .selector("& svg", sx().display("block").width("100%").height("100%"))
});

base_props! {
    pub struct QrCodeProps {
        /// The payload encoded into the code.
        data: String,
        #[props(default, into)]
        robustness: Input<QrRobustness>,
        /// Required - a QR code conveys real information to a sighted/scanning
        /// user, but nothing to a screen reader without one.
        aria_label: String,
    }
}

fn generate_svg(data: String, robustness: QrRobustness) -> Option<String> {
    QRBuilder::new(data)
        .ecl(robustness.into())
        .build()
        .ok()
        .map(|qrcode| {
            SvgBuilder::default()
                .margin(QR_CODE_MARGIN)
                .module_color(QR_CODE_FOREGROUND.value())
                .background_color(QR_CODE_BACKGROUND.value())
                .to_str(&qrcode)
        })
}

// `#[wasm_split]` drops the item's visibility on wasm32 (it rebuilds the fn
// from just `item_fn.sig`, which doesn't carry `vis`), so this stays private
// and `generate_svg_lazy` below re-exposes it at the visibility callers need.
#[cfg(feature = "wasm-split")]
#[wasm_split::wasm_split(qr_code)]
async fn generate_svg_split(data: String, robustness: QrRobustness) -> Option<String> {
    generate_svg(data, robustness)
}

/// Same as [`generate_svg`], but with the `wasm-split` feature on, `fast_qr`
/// itself lives in a separate wasm chunk fetched on first call instead of
/// the main bundle - see `dioxus::wasm_split`. Without that feature (or on
/// non-wasm32 targets) this is just a plain async call with nothing split
/// out, so it works with a plain `dx serve`/`dx build`, no `--wasm-split`
/// cooperation required.
async fn generate_svg_lazy(data: String, robustness: QrRobustness) -> Option<String> {
    #[cfg(feature = "wasm-split")]
    return generate_svg_split(data, robustness).await;

    #[cfg(not(feature = "wasm-split"))]
    return generate_svg(data, robustness);
}

/// Renders `data` as a scalable QR code SVG. Colors come from the theme
/// (`Theme::qr_code`); `robustness` picks the error-correction level,
/// falling back to the theme's default when unset. Renders nothing if
/// `data` is too long for the chosen `robustness` level to encode.
#[component]
pub fn QrCode(props: QrCodeProps) -> Element {
    let theme = use_theme();
    let robustness = props.robustness.copied_or(theme.qr_code.robustness);
    let data = props.data.clone();

    let svg = use_resource(use_reactive!(|data, robustness| async move {
        generate_svg_lazy(data, robustness).await
    }));

    rsx! {
        if let Some(svg) = svg.read().clone().flatten() {
            Box {
                class: props.class,
                sx: props.sx,
                states: props.states,
                framework_sx: &QR_CODE_BASE_SX,
                role: "img",
                "aria-label": props.aria_label,
                attributes: props.attributes,
                div { dangerous_inner_html: "{svg}" }
            }
        }
    }
}
