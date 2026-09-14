use dioxus::prelude::*;
use fast_qr::convert::{Builder, svg::SvgBuilder};
use fast_qr::{ECL, QRBuilder};

use crate::{
    components::{
        HtmlTag, Input,
        common::{base_props, input_from_str},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, sx},
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

/// `data` as a scalable QR code SVG, colored from `Theme::qr_code`. Renders
/// nothing if `data` is too long for the chosen `robustness` to encode.
#[component]
pub fn QrCode(props: QrCodeProps) -> Element {
    let theme = use_theme();
    let robustness = props.robustness.copied_or(theme.qr_code.robustness);
    let data = props.data.clone();

    let svg = use_resource(use_reactive!(|data, robustness| async move {
        generate_svg(data, robustness)
    }));

    let boxed = use_box()
        .framework_sx(&QR_CODE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare();

    let Some(svg) = svg.read().clone().flatten() else {
        return rsx! {};
    };

    boxed
        .attr("role", "img")
        .attr("aria-label", props.aria_label)
        .render(
            HtmlTag::Div,
            props.attributes,
            // The root is the image; the bare `<svg>` would be a second, nameless one.
            rsx! { div { "aria-hidden": "true", dangerous_inner_html: "{svg}" } },
        )
}
