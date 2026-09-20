use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, Variables, base_props, variables},
        layout::use_box,
    },
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{OVERLAY_BLUR, OVERLAY_OPACITY, Z_INDEX_OVERLAY},
};

static OVERLAY_BASE_SX: StaticSx = StaticSx::new(|| {
    sx().position("fixed")
        .inset("0")
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .z_index(Z_INDEX_OVERLAY.overridable())
        .background(format!("rgba(0, 0, 0, {})", OVERLAY_OPACITY.overridable()))
        .backdrop_filter(OVERLAY_BLUR.overridable())
});

base_props! {
    pub struct OverlayProps {
        #[props(default, into)]
        z_index: Input<ThemeAwareValue>,
        /// Opacity of the black backdrop.
        #[props(default, into)]
        opacity: Input<ThemeAwareValue>,
        /// Backdrop blur; a bare number is `px`.
        #[props(default, into)]
        blur: Input<ThemeAwareValue>,
        #[props(default)]
        onclick: Option<EventHandler<MouseEvent>>,
        children: Option<Element>,
    }
}

/// Bare numbers become `px`; strings and vars pass through.
fn px_value(value: &ThemeAwareValue) -> Option<String> {
    match value {
        ThemeAwareValue::Number(number) => Some(format!("{number}px")),
        _ => value.resolve(None),
    }
}

/// Dims and blurs what's behind it. Render it conditionally: there is no `open`.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::Overlay;
/// # fn app() -> Element {
/// # let mut shown = use_signal(|| true);
/// # rsx! {
/// if shown() {
///     Overlay { opacity: 0.6, blur: 2, onclick: move |_| shown.set(false) }
/// }
/// # } }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/overlay>
#[component]
pub fn Overlay(props: OverlayProps) -> Element {
    let variables: Input<Variables> = variables()
        .with(OVERLAY_OPACITY.override_var(), props.opacity.resolve(None))
        .with(Z_INDEX_OVERLAY.override_var(), props.z_index.resolve(None))
        .with(
            OVERLAY_BLUR.override_var(),
            props
                .blur
                .as_ref()
                .and_then(px_value)
                .map(|blur| format!("blur({blur})")),
        )
        .into();

    use_box()
        .framework_sx(&OVERLAY_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .variables(&variables)
        .prepare()
        .event("onclick", props.onclick)
        .render(HtmlTag::Div, props.attributes, props.children)
}
