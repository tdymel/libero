use dioxus::prelude::*;
#[cfg(feature = "wasm-split")]
use dioxus::wasm_split;
use fast_qr::convert::{Builder, svg::SvgBuilder};
use fast_qr::{ECL, QRBuilder};

use crate::{
    components::{
        Box, Input, States,
        common::{base_props, input_from_str},
    },
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

input_from_str!(QrRobustness);

// ISO/IEC 18004 quiet zone. Not a style choice - scanners need it.
const QR_CODE_MARGIN: usize = 4;

// `fast_qr`'s svg is viewBox-only, so it already scales to fill. `& svg` is
// there to drop inline-svg's baseline gap underneath.
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
        /// Required: a QR code says nothing to a screen reader without one.
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

// `#[wasm_split]` rebuilds the fn from its signature alone and so drops
// visibility on wasm32; `generate_svg_lazy` re-exposes it.
#[cfg(feature = "wasm-split")]
#[wasm_split::wasm_split(qr_code)]
async fn generate_svg_split(data: String, robustness: QrRobustness) -> Option<String> {
    generate_svg(data, robustness)
}

/// [`generate_svg`], but under the `wasm-split` feature `fast_qr` lives in a
/// separate chunk fetched on first call. Without it this is a plain async
/// call, so no `--wasm-split` build is required.
async fn generate_svg_lazy(data: String, robustness: QrRobustness) -> Option<String> {
    #[cfg(feature = "wasm-split")]
    return generate_svg_split(data, robustness).await;

    #[cfg(not(feature = "wasm-split"))]
    return generate_svg(data, robustness);
}

/// `data` as a scalable QR code SVG, colored from `Theme::qr_code`. Renders
/// nothing if `data` is too long for the chosen `robustness` to encode.
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
