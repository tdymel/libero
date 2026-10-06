use dioxus::prelude::*;
use fast_qr::convert::{Builder, svg::SvgBuilder};
use fast_qr::{ECL, QRBuilder};

use crate::{
    components::{
        common::{HtmlTag, Input, base_props, input_from_str, names_itself, use_name_warning},
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

// ISO/IEC 18004 quiet zone: scanners need it.
const QR_CODE_MARGIN: usize = 4;

// `fast_qr`'s svg is viewBox-only; `& svg` drops the inline baseline gap.
static QR_CODE_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().display("block")
        .width("100%")
        .color(QR_CODE_FOREGROUND.value())
        .background(QR_CODE_BACKGROUND.value())
        // Forced colours would invert it (light on dark), which many scanners reject.
        .forced_color_adjust("none")
        .selector("& svg", sx().display("block").width("100%").height("100%"))
});

base_props! {
    pub struct QrCodeProps {
        /// The payload encoded into the code.
        data: String,
        #[props(default, into)]
        robustness: Input<QrRobustness>,
        /// Required: the code's accessible name.
        aria_label: String,
    }
}

fn generate_svg(data: String, robustness: QrRobustness) -> Option<String> {
    QRBuilder::new(data)
        .ecl(robustness.into())
        .build()
        .ok()
        .map(|qrcode| {
            // Colours come from the root: Blitz's svg renderer resolves no `var()`.
            SvgBuilder::default()
                .margin(QR_CODE_MARGIN)
                .module_color("currentColor")
                .background_color("none")
                .to_str(&qrcode)
        })
}

/// `data` as a scalable QR code SVG. Renders nothing if `data` is too long to encode.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::QrCode;
/// # fn app() -> Element {
/// rsx! {
///     QrCode { data: "https://libero-ui.dev", aria_label: "Link to the Libero docs" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/qr-code>
#[component]
pub fn QrCode(props: QrCodeProps) -> Element {
    let theme = use_theme();
    let robustness = props.robustness.copied_or(theme.qr_code.robustness);
    let data = props.data.clone();
    use_name_warning(
        !props.aria_label.trim().is_empty() || names_itself(&props.attributes),
        "QrCode: no `aria_label` - name the code, e.g. what it links to; it is the only way \
         to the payload for a screen reader.",
    );

    // Synchronous, so the server and the first client frame render the code.
    let svg = use_memo(use_reactive!(|data, robustness| generate_svg(
        data, robustness
    )));

    let boxed = use_box()
        .framework_sx(&QR_CODE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare();

    let Some(svg) = svg.read().clone() else {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Todo 2530: the code is in the first render, so the server's HTML has it.
    #[test]
    fn the_first_render_has_the_named_code() {
        let mut dom = VirtualDom::new(|| {
            rsx! { crate::LiberoProvider { QrCode { data: "a", aria_label: "A link" } } }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("role=\"img\""), "{html}");
        assert!(html.contains("<svg"), "{html}");
    }
}
